//! 厂商额度/用量查询适配器
//!
//! 各家 OpenAI 兼容/自有计费接口差异大，统一归一化为 `BalanceInfo` + `ModelUsage`。
//! 查询地址一律由适配器按 host 重建（而非拼接用户提供的 base_url 路径），
//! 并强制 https + host 白名单，防止密钥被发往任意地址。

use serde_json::Value;

use crate::error::AppError;
use crate::models::usage::{BalanceInfo, ExtraLine, ModelUsage};

/// 请求超时（秒）
const HTTP_TIMEOUT_SECS: u64 = 10;
/// 厂商错误响应体透传的最大长度
const MAX_ERROR_BODY_LEN: usize = 200;

/// 已适配额度查询的厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    DeepSeek,
    Moonshot,
    Zhipu,
    OpenRouter,
    SiliconFlow,
}

impl AdapterKind {
    /// 相对 host 根部的查询路径（重建 URL 用）
    fn endpoint_path(self) -> &'static str {
        match self {
            Self::DeepSeek => "/user/balance",
            Self::Moonshot => "/v1/users/me/balance",
            Self::Zhipu => "/api/monitor/usage/quota/limit",
            Self::OpenRouter => "/api/v1/credits",
            Self::SiliconFlow => "/v1/user/info",
        }
    }
}

/// 从 base_url 提取 host（仅接受 https），并匹配适配器
pub fn find_adapter(base_url: &str) -> Option<AdapterKind> {
    let rest = base_url.strip_prefix("https://")?;
    let host = rest.split('/').next()?.to_ascii_lowercase();
    match host.as_str() {
        "api.deepseek.com" => Some(AdapterKind::DeepSeek),
        // Kimi 国内/海外/新域名均指向同一套账户接口
        "api.moonshot.cn" | "api.moonshot.ai" | "api.kimi.com" => Some(AdapterKind::Moonshot),
        "open.bigmodel.cn" => Some(AdapterKind::Zhipu),
        "openrouter.ai" => Some(AdapterKind::OpenRouter),
        "api.siliconflow.cn" | "api.siliconflow.com" => Some(AdapterKind::SiliconFlow),
        _ => None,
    }
}

/// 该厂商无公开接口时的可读说明
pub fn unsupported_reason(provider_id: &str, provider_name: &str) -> String {
    match provider_id {
        "openai" => "OpenAI 未对普通 API Key 开放额度接口（org usage 需 Admin Key），请前往 platform.openai.com/usage 查看".into(),
        "anthropic" => "Anthropic 未提供公开额度查询接口，请前往 console.anthropic.com 查看".into(),
        "google" => "Google AI 未提供公开额度查询接口，请前往 AI Studio 计费页查看".into(),
        "qwen" => "通义千问（百炼）余额需通过阿里云账号查询，API Key 不支持额度接口，请前往 bailian.console.aliyun.com 查看".into(),
        _ => format!("该服务商（{provider_name}）未适配公开的 Key 级额度查询接口"),
    }
}

/// 发起查询：返回归一化余额与（厂商支持时的）按模型 token 用量
pub async fn fetch_usage(
    kind: AdapterKind,
    host: &str,
    api_key: &str,
) -> Result<(BalanceInfo, Option<Vec<ModelUsage>>), AppError> {
    let url = format!("https://{host}{}", kind.endpoint_path());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let response = client
        .get(&url)
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("网络请求失败: {e}")))?;

    let status = response.status();
    let body: Value = if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Internal(
            "认证失败：API Key 无效或该接口需要管理类 Key".into(),
        ));
    } else {
        let text = response
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("读取响应失败: {e}")))?;
        if !status.is_success() {
            let brief: String = text.chars().take(MAX_ERROR_BODY_LEN).collect();
            return Err(AppError::Internal(format!(
                "查询失败: HTTP {} {brief}",
                status.as_u16()
            )));
        }
        serde_json::from_str(&text)
            .map_err(|_| AppError::Internal("厂商返回了非 JSON 响应".into()))?
    };

    match kind {
        AdapterKind::DeepSeek => Ok((parse_deepseek(&body)?, None)),
        AdapterKind::Moonshot => Ok((parse_moonshot(&body, host)?, None)),
        AdapterKind::Zhipu => Ok(parse_zhipu(&body)?),
        AdapterKind::OpenRouter => Ok((parse_openrouter(&body)?, None)),
        AdapterKind::SiliconFlow => Ok((parse_siliconflow(&body, host)?, None)),
    }
}

/// 从 base_url 提取 host（配合 find_adapter 使用）
pub fn host_of(base_url: &str) -> Option<String> {
    let rest = base_url.strip_prefix("https://")?;
    Some(rest.split('/').next()?.to_ascii_lowercase())
}

// ---------- 各厂商响应解析（纯函数，便于单测） ----------

/// 宽容取数：兼容数字与字符串两种 JSON 形态
fn num(val: &Value, key: &str) -> Option<f64> {
    match val.get(key)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn parse_deepseek(body: &Value) -> Result<BalanceInfo, AppError> {
    let infos = body
        .get("balance_infos")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Internal("DeepSeek 响应缺少 balance_infos".into()))?;
    let first = infos
        .first()
        .ok_or_else(|| AppError::Internal("DeepSeek balance_infos 为空".into()))?;
    let mut extra = Vec::new();
    if body.get("is_available").and_then(Value::as_bool) == Some(false) {
        extra.push(ExtraLine {
            label: "账户状态".into(),
            value: "余额不足或已停用".into(),
        });
    }
    Ok(BalanceInfo {
        currency: first
            .get("currency")
            .and_then(Value::as_str)
            .unwrap_or("CNY")
            .to_string(),
        total: num(first, "total_balance"),
        granted: num(first, "granted_balance"),
        topped_up: num(first, "topped_up_balance"),
        extra,
    })
}

fn parse_moonshot(body: &Value, host: &str) -> Result<BalanceInfo, AppError> {
    let data = body
        .get("data")
        .ok_or_else(|| AppError::Internal("Moonshot 响应缺少 data".into()))?;
    // 代金券余额可能为列表，累加各类券
    let voucher = data
        .get("voucher_balance_list")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|v| num(v, "voucher_balance"))
                .sum::<f64>()
        })
        .or_else(|| num(data, "voucher_balance"));
    let mut extra = Vec::new();
    if let Some(overdue) = num(data, "overdue_balance").filter(|v| *v > 0.0) {
        extra.push(ExtraLine {
            label: "欠款".into(),
            value: fmt_amount(overdue),
        });
    }
    Ok(BalanceInfo {
        currency: if host == "api.moonshot.cn" {
            "CNY".into()
        } else {
            "USD".into()
        },
        total: num(data, "available_balance").or_else(|| num(data, "balance")),
        granted: voucher,
        topped_up: num(data, "cash_balance"),
        extra,
    })
}

/// 智谱 monitor 接口返回的是配额而非金额：余额仅取可得金额，配额明细进 extra，
/// TOKENS_LIMIT 的 usageDetails 归一化为按模型 token 用量
fn parse_zhipu(body: &Value) -> Result<(BalanceInfo, Option<Vec<ModelUsage>>), AppError> {
    if body.get("success").and_then(Value::as_bool) == Some(false) {
        let msg = body
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("未知错误");
        return Err(AppError::Internal(format!("智谱配额查询失败: {msg}")));
    }
    let data = body.get("data").unwrap_or(&Value::Null);
    let mut extra = Vec::new();
    let mut model_usage: Vec<ModelUsage> = Vec::new();

    if let Some(limits) = data.get("limits").and_then(Value::as_array) {
        for limit in limits {
            let kind = limit.get("type").and_then(Value::as_str).unwrap_or("LIMIT");
            let current = num(limit, "currentValue");
            let cap = num(limit, "usage");
            let remaining = num(limit, "remaining");
            let percentage = num(limit, "percentage");
            let value = match (current, cap, remaining, percentage) {
                (Some(current), Some(cap), Some(remaining), Some(percentage)) if cap > 0.0 => {
                    format!(
                        "已用 {} / {}（剩余 {}，{percentage:.0}%）",
                        fmt_amount(current),
                        fmt_amount(cap),
                        fmt_amount(remaining)
                    )
                }
                _ => format!(
                    "已用 {}，剩余 {}",
                    current.map(fmt_amount).unwrap_or_else(|| "?".into()),
                    remaining.map(fmt_amount).unwrap_or_else(|| "?".into())
                ),
            };
            extra.push(ExtraLine {
                label: limit_label(kind),
                value,
            });
            if let Some(details) = limit.get("usageDetails").and_then(Value::as_array) {
                for detail in details {
                    if let Some(model) = detail.get("modelCode").and_then(Value::as_str) {
                        model_usage.push(ModelUsage {
                            model: model.to_string(),
                            prompt_tokens: None,
                            completion_tokens: None,
                            total_tokens: num(detail, "usage").map(|v| v as i64),
                        });
                    }
                }
            }
        }
    }

    let balance = BalanceInfo {
        currency: "配额".into(),
        total: num(data, "currentBalance").or_else(|| num(data, "balance")),
        granted: None,
        topped_up: None,
        extra,
    };
    Ok((balance, (!model_usage.is_empty()).then_some(model_usage)))
}

fn limit_label(kind: &str) -> String {
    match kind {
        "TOKENS_LIMIT" => "Token 配额".into(),
        "TIME_LIMIT" => "时长配额".into(),
        other => format!("配额（{other}）"),
    }
}

fn parse_openrouter(body: &Value) -> Result<BalanceInfo, AppError> {
    let data = body
        .get("data")
        .ok_or_else(|| AppError::Internal("OpenRouter 响应缺少 data".into()))?;
    let credits = num(data, "total_credits").ok_or_else(|| {
        AppError::Internal("OpenRouter 响应缺少 total_credits（需 Management Key）".into())
    })?;
    let usage = num(data, "total_usage").unwrap_or(0.0);
    Ok(BalanceInfo {
        currency: "USD".into(),
        total: Some(credits - usage),
        granted: None,
        topped_up: Some(credits),
        extra: vec![ExtraLine {
            label: "累计消费".into(),
            value: fmt_amount(usage),
        }],
    })
}

fn parse_siliconflow(body: &Value, host: &str) -> Result<BalanceInfo, AppError> {
    let data = body
        .get("data")
        .ok_or_else(|| AppError::Internal("SiliconFlow 响应缺少 data".into()))?;
    let balance = num(data, "balance");
    let charge = num(data, "chargeBalance");
    let total = num(data, "totalBalance").or(match (balance, charge) {
        (Some(b), Some(c)) => Some(b + c),
        (Some(b), None) => Some(b),
        (None, Some(c)) => Some(c),
        _ => None,
    });
    Ok(BalanceInfo {
        currency: if host == "api.siliconflow.cn" {
            "CNY".into()
        } else {
            "USD".into()
        },
        total,
        granted: balance,
        topped_up: charge,
        extra: Vec::new(),
    })
}

/// 金额格式化：保留两位小数并去掉多余的 .00
fn fmt_amount(v: f64) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{:.0}", v)
    } else {
        format!("{v:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(raw: &str) -> Value {
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn test_find_adapter_matches_hosts() {
        assert_eq!(
            find_adapter("https://api.deepseek.com/v1"),
            Some(AdapterKind::DeepSeek)
        );
        assert_eq!(
            find_adapter("https://open.bigmodel.cn/api/paas/v4"),
            Some(AdapterKind::Zhipu)
        );
        assert_eq!(
            find_adapter("https://api.moonshot.ai/v1"),
            Some(AdapterKind::Moonshot)
        );
        // 自定义服务商指向同域也可复用
        assert_eq!(
            find_adapter("https://api.siliconflow.cn/v1"),
            Some(AdapterKind::SiliconFlow)
        );
        // http / 未知域 / OpenAI 等未适配厂商一律拒绝
        assert_eq!(find_adapter("http://api.deepseek.com/v1"), None);
        assert_eq!(find_adapter("https://api.openai.com/v1"), None);
        assert_eq!(find_adapter("https://evil.example"), None);
    }

    #[test]
    fn test_parse_deepseek() {
        // 官方文档示例：金额字段为字符串
        let body = json(
            r#"{"is_available":false,"balance_infos":[{"currency":"CNY","total_balance":" 12.34","granted_balance":"0.00","topped_up_balance":"12.34"}]}"#,
        );
        let balance = parse_deepseek(&body).unwrap();
        assert_eq!(balance.currency, "CNY");
        assert_eq!(balance.total, Some(12.34));
        assert_eq!(balance.granted, Some(0.0));
        assert_eq!(balance.extra.len(), 1);
        assert_eq!(balance.extra[0].value, "余额不足或已停用");
    }

    #[test]
    fn test_parse_deepseek_rejects_missing_field() {
        assert!(parse_deepseek(&json(r#"{"is_available":true}"#)).is_err());
    }

    #[test]
    fn test_parse_moonshot() {
        let body = json(
            r#"{"code":0,"data":{"available_balance":98.5,"voucher_balance_list":[{"voucher_balance":2.5,"type":"normal_voucher"}],"cash_balance":96.0,"overdue_balance":1.2}}"#,
        );
        let balance = parse_moonshot(&body, "api.moonshot.cn").unwrap();
        assert_eq!(balance.currency, "CNY");
        assert_eq!(balance.total, Some(98.5));
        assert_eq!(balance.granted, Some(2.5));
        assert_eq!(balance.topped_up, Some(96.0));
        assert_eq!(balance.extra[0].label, "欠款");

        // 字段缺失时宽容降级，海外域名计 USD
        let balance =
            parse_moonshot(&json(r#"{"data":{"balance":5.0}}"#), "api.moonshot.ai").unwrap();
        assert_eq!(balance.total, Some(5.0));
        assert_eq!(balance.currency, "USD");
        assert!(balance.extra.is_empty());
        // 缺 data 时报错
        assert!(parse_moonshot(&json(r#"{"code":-1}"#), "api.moonshot.cn").is_err());
    }

    #[test]
    fn test_parse_zhipu_limits_and_model_usage() {
        // 智谱 monitor/usage/quota/limit 实测响应结构（节选）
        let body = json(
            r#"{"code":200,"msg":"操作成功","data":{"limits":[{"type":"TIME_LIMIT","unit":5,"number":1,"usage":100,"currentValue":0,"remaining":100,"percentage":0,"usageDetails":[{"modelCode":"search-prime","usage":0},{"modelCode":"web-reader","usage":7}]},{"type":"TOKENS_LIMIT","unit":3,"number":5,"usage":40000000,"currentValue":10261098,"remaining":29738902,"percentage":25}]},"success":true}"#,
        );
        let (balance, usage) = parse_zhipu(&body).unwrap();
        assert_eq!(balance.currency, "配额");
        assert_eq!(balance.extra.len(), 2);
        assert_eq!(balance.extra[1].label, "Token 配额");
        assert!(balance.extra[1].value.starts_with("已用 10261098"));
        let usage = usage.unwrap();
        assert_eq!(usage.len(), 2);
        assert_eq!(usage[1].model, "web-reader");
        assert_eq!(usage[1].total_tokens, Some(7));
    }

    #[test]
    fn test_parse_zhipu_business_error_and_empty() {
        let err = json(r#"{"success":false,"msg":"请求过于频繁，请稍后再试"}"#);
        assert!(parse_zhipu(&err).is_err());
        // 普通账户无 limits 时不报 token 用量
        let (balance, usage) = parse_zhipu(&json(r#"{"success":true,"data":{}}"#)).unwrap();
        assert_eq!(balance.total, None);
        assert!(usage.is_none());
    }

    #[test]
    fn test_parse_openrouter() {
        let body = json(r#"{"data":{"total_credits":112.74,"total_usage":100.24}}"#);
        let balance = parse_openrouter(&body).unwrap();
        assert_eq!(balance.currency, "USD");
        assert!((balance.total.unwrap() - 12.50).abs() < 1e-9);
        assert_eq!(balance.extra[0].value, "100.24");
        // 缺 total_credits（如非 Management Key）应明确报错
        assert!(parse_openrouter(&json(r#"{"data":{}}"#)).is_err());
    }

    #[test]
    fn test_parse_siliconflow_string_amounts() {
        let body = json(
            r#"{"code":20000,"status":true,"data":{"id":"u1","balance":"0.88","chargeBalance":"88.00","totalBalance":"88.88"}}"#,
        );
        let balance = parse_siliconflow(&body, "api.siliconflow.cn").unwrap();
        assert_eq!(balance.total, Some(88.88));
        assert_eq!(balance.granted, Some(0.88));
        assert_eq!(balance.topped_up, Some(88.0));
        // totalBalance 缺失时回退求和
        let body = json(r#"{"data":{"balance":"1.5","chargeBalance":"2.5"}}"#);
        let balance = parse_siliconflow(&body, "api.siliconflow.cn").unwrap();
        assert_eq!(balance.total, Some(4.0));
    }

    #[test]
    fn test_unsupported_reason_mentions_console_hint() {
        let reason = unsupported_reason("openai", "OpenAI");
        assert!(reason.contains("Admin Key"));
        let reason = unsupported_reason("mine", "自建网关");
        assert!(reason.contains("自建网关"));
    }
}
