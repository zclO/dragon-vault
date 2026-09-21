use regex::Regex;

use crate::error::AppError;
use crate::models::provider::ProviderConfig;

/// 内置主流大模型服务商配置（与前端 mock-data 保持一致）
pub fn builtin_providers() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig {
            id: "openai".into(),
            name: "OpenAI".into(),
            base_url: "https://api.openai.com/v1".into(),
            key_format_pattern: Some("^sk-[a-zA-Z0-9_-]+$".into()),
            models: vec!["gpt-4o".into(), "gpt-4o-mini".into(), "gpt-4-turbo".into()],
            is_built_in: true,
        },
        ProviderConfig {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            base_url: "https://api.anthropic.com/v1".into(),
            key_format_pattern: Some("^sk-ant-[a-zA-Z0-9-]+$".into()),
            models: vec![
                "claude-sonnet-4-20250514".into(),
                "claude-3-5-haiku-20241022".into(),
            ],
            is_built_in: true,
        },
        ProviderConfig {
            id: "google".into(),
            name: "Google AI".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
            key_format_pattern: Some("^AIza[a-zA-Z0-9_-]+$".into()),
            models: vec![
                "gemini-2.0-flash".into(),
                "gemini-1.5-pro".into(),
                "gemini-1.5-flash".into(),
            ],
            is_built_in: true,
        },
        ProviderConfig {
            id: "zhipu".into(),
            name: "智谱 AI".into(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".into(),
            // 智谱 Key 形如 <id>.<JWT>，JWT 段包含大小写字母、数字与 . _ -
            key_format_pattern: Some("^[a-zA-Z0-9._-]+$".into()),
            models: vec![
                "glm-4-plus".into(),
                "glm-4-air".into(),
                "glm-4-flash".into(),
            ],
            is_built_in: true,
        },
        ProviderConfig {
            id: "qwen".into(),
            name: "通义千问".into(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".into(),
            key_format_pattern: Some("^sk-[a-f0-9]+$".into()),
            models: vec!["qwen-max".into(), "qwen-plus".into(), "qwen-turbo".into()],
            is_built_in: true,
        },
        ProviderConfig {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            key_format_pattern: Some("^sk-[a-zA-Z0-9]+$".into()),
            models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()],
            is_built_in: true,
        },
        ProviderConfig {
            id: "moonshot".into(),
            name: "Kimi（月之暗面）".into(),
            base_url: "https://api.moonshot.cn/v1".into(),
            key_format_pattern: Some("^sk-[a-zA-Z0-9_-]+$".into()),
            models: vec![
                "kimi-latest".into(),
                "moonshot-v1-8k".into(),
                "moonshot-v1-32k".into(),
            ],
            is_built_in: true,
        },
        ProviderConfig {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            key_format_pattern: Some("^sk-or-[a-zA-Z0-9_-]+$".into()),
            models: vec![
                "openai/gpt-4o".into(),
                "anthropic/claude-sonnet-4".into(),
                "deepseek/deepseek-chat".into(),
            ],
            is_built_in: true,
        },
        ProviderConfig {
            id: "siliconflow".into(),
            name: "硅基流动 SiliconFlow".into(),
            base_url: "https://api.siliconflow.cn/v1".into(),
            key_format_pattern: Some("^sk-[a-zA-Z0-9_-]+$".into()),
            models: vec![
                "deepseek-ai/DeepSeek-V3".into(),
                "Qwen/Qwen2.5-72B-Instruct".into(),
            ],
            is_built_in: true,
        },
    ]
}

/// 在内置与自定义服务商中查找指定 id 的配置
pub fn find_provider<'a>(
    providers: impl IntoIterator<Item = &'a ProviderConfig>,
    provider_id: &str,
) -> Option<&'a ProviderConfig> {
    providers.into_iter().find(|p| p.id == provider_id)
}

/// 合并内置服务商与自定义服务商（自定义项不覆盖内置 id）
pub fn merged_providers(custom: &[ProviderConfig]) -> Vec<ProviderConfig> {
    let mut all = builtin_providers();
    let builtin_ids: Vec<String> = all.iter().map(|p| p.id.clone()).collect();
    let extras: Vec<ProviderConfig> = custom
        .iter()
        .filter(|p| !builtin_ids.iter().any(|id| id == &p.id))
        .cloned()
        .collect();
    all.extend(extras);
    all
}

/// 校验自定义服务商配置的合法性
pub fn validate_provider(provider: &ProviderConfig) -> Result<(), AppError> {
    if provider.name.trim().is_empty() {
        return Err(AppError::ValidationError("服务商名称不能为空".into()));
    }
    if !provider.base_url.starts_with("http://") && !provider.base_url.starts_with("https://") {
        return Err(AppError::ValidationError(
            "Base URL 必须以 http(s):// 开头".into(),
        ));
    }
    validate_pattern(provider.key_format_pattern.as_deref())?;
    Ok(())
}

/// 校验自定义服务商 id（仅允许小写字母、数字、连字符）
pub fn validate_custom_id(id: &str) -> Result<(), AppError> {
    let ok = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(AppError::ValidationError(
            "服务商 ID 仅允许小写字母、数字和连字符".into(),
        ))
    }
}

/// 按服务商的 Key 格式规则校验 API Key 值
pub fn validate_key_format(pattern: Option<&str>, value: &str) -> Result<(), AppError> {
    let Some(pattern) = pattern.filter(|p| !p.is_empty()) else {
        return Ok(());
    };
    let re =
        Regex::new(pattern).map_err(|e| AppError::Internal(format!("Key 格式规则无效: {e}")))?;
    if !re.is_match(value.trim()) {
        return Err(AppError::ValidationError(
            "API Key 格式不符合该服务商的规则".into(),
        ));
    }
    Ok(())
}

/// 正则表达式必须能编译
fn validate_pattern(pattern: Option<&str>) -> Result<(), AppError> {
    if let Some(p) = pattern.filter(|p| !p.is_empty()) {
        Regex::new(p).map_err(|e| AppError::ValidationError(format!("Key 格式正则无效: {e}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_ids_are_unique() {
        let providers = builtin_providers();
        let ids: Vec<&str> = providers.iter().map(|p| p.id.as_str()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_validate_key_format_matches_and_rejects() {
        let pattern = Some("^sk-[a-zA-Z0-9]+$");
        assert!(validate_key_format(pattern, "sk-abc123").is_ok());
        assert!(matches!(
            validate_key_format(pattern, "pk-abc123"),
            Err(AppError::ValidationError(_))
        ));
        // 无规则时放行
        assert!(validate_key_format(None, "anything").is_ok());
    }

    #[test]
    fn test_validate_provider_rejects_bad_base_url() {
        let provider = ProviderConfig {
            id: "test".into(),
            name: "Test".into(),
            base_url: "ftp://bad".into(),
            key_format_pattern: None,
            models: vec![],
            is_built_in: false,
        };
        assert!(matches!(
            validate_provider(&provider),
            Err(AppError::ValidationError(_))
        ));
    }

    #[test]
    fn test_merged_providers_ignore_duplicate_custom() {
        let custom = vec![ProviderConfig {
            id: "openai".into(),
            name: "冲突项".into(),
            base_url: "https://evil.example".into(),
            key_format_pattern: None,
            models: vec![],
            is_built_in: false,
        }];
        let merged = merged_providers(&custom);
        let openai = find_provider(&merged, "openai").unwrap();
        assert_eq!(openai.name, "OpenAI");
        assert!(openai.is_built_in);
    }
}
