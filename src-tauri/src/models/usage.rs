use serde::{Deserialize, Serialize};

/// 余额附加行（各厂商归一化后仍无法对齐的字段，如代金券/欠款/配额行）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraLine {
    pub label: String,
    pub value: String,
}

/// 归一化后的余额/额度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceInfo {
    /// 币种或额度类别（CNY / USD / 配额 等）
    pub currency: String,
    /// 总余额（可用），部分厂商仅有配额信息时为空
    pub total: Option<f64>,
    /// 赠送余额
    pub granted: Option<f64>,
    /// 充值余额
    pub topped_up: Option<f64>,
    /// 厂商特有明细
    pub extra: Vec<ExtraLine>,
}

/// 单模型 token 用量（仅支持按模型查询的厂商提供）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
}

/// 一次额度/用量查询的结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub key_id: String,
    pub provider_id: String,
    pub fetched_at: String,
    pub balance: Option<BalanceInfo>,
    pub token_usage: Option<Vec<ModelUsage>>,
    /// 厂商无公开接口时的说明（此时 balance/token_usage 为空）
    pub unsupported_reason: Option<String>,
}

/// 持久化进保险库的用量快照（每 Key 保留最近若干条，用于趋势展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshotRecord {
    pub key_id: String,
    pub ts: String,
    pub total_balance: Option<f64>,
    pub total_tokens: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_serde_camel_case() {
        let report = UsageReport {
            key_id: "k1".into(),
            provider_id: "deepseek".into(),
            fetched_at: "2026-09-21T00:00:00Z".into(),
            balance: Some(BalanceInfo {
                currency: "CNY".into(),
                total: Some(9.5),
                granted: None,
                topped_up: None,
                extra: vec![ExtraLine {
                    label: "账户状态".into(),
                    value: "正常".into(),
                }],
            }),
            token_usage: None,
            unsupported_reason: None,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"fetchedAt\""));
        assert!(json.contains("\"toppedUp\""));
        assert!(json.contains("\"unsupportedReason\""));
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let record = UsageSnapshotRecord {
            key_id: "k1".into(),
            ts: "2026-09-21T00:00:00Z".into(),
            total_balance: Some(1.0),
            total_tokens: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        let back: UsageSnapshotRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_balance, Some(1.0));
        assert_eq!(back.total_tokens, None);
    }
}
