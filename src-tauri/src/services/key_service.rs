use chrono::{Duration, Utc};

use crate::error::AppError;
use crate::models::api_key::{ApiKey, ApiKeySummary};
use crate::models::vault::{CreateApiKeyRequest, DashboardStats, VaultContents};
use crate::services::{crypto::CryptoService, provider_service};

/// 创建一条 API Key（校验参数与格式后加密存储）
pub fn create_key(
    contents: &mut VaultContents,
    key_format_pattern: Option<&str>,
    req: &CreateApiKeyRequest,
    vault_key: &[u8],
) -> Result<ApiKeySummary, AppError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::ValidationError("Key 名称不能为空".into()));
    }
    let value = req.value.trim();
    if value.is_empty() {
        return Err(AppError::ValidationError("API Key 值不能为空".into()));
    }
    provider_service::validate_key_format(key_format_pattern, value)?;

    let now = Utc::now().to_rfc3339();
    let encrypted_value = CryptoService::encrypt(vault_key, value.as_bytes())?;
    let api_key = ApiKey {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        provider_id: req.provider_id.clone(),
        encrypted_value,
        tags: normalize_tags(&req.tags),
        created_at: now.clone(),
        updated_at: now,
        last_used_at: None,
    };
    let summary = to_summary(
        &api_key,
        contents_provider_name(contents, &api_key.provider_id),
    );
    contents.api_keys.push(api_key);
    Ok(summary)
}

/// 更新 API Key 的名称 / 标签 / 值（None 字段保持不变）
pub fn update_key(
    contents: &mut VaultContents,
    key_format_pattern: Option<&str>,
    id: &str,
    name: Option<&str>,
    value: Option<&str>,
    tags: Option<Vec<String>>,
    vault_key: &[u8],
) -> Result<ApiKeySummary, AppError> {
    let index = contents
        .api_keys
        .iter()
        .position(|k| k.id == id)
        .ok_or_else(|| AppError::NotFound(format!("API Key 不存在: {id}")))?;

    if let Some(name) = name.map(str::trim).filter(|n| !n.is_empty()) {
        contents.api_keys[index].name = name.to_string();
    }
    if let Some(tags) = tags {
        contents.api_keys[index].tags = normalize_tags(&Some(tags));
    }
    if let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) {
        provider_service::validate_key_format(key_format_pattern, value)?;
        contents.api_keys[index].encrypted_value =
            CryptoService::encrypt(vault_key, value.as_bytes())?;
    }
    contents.api_keys[index].updated_at = Utc::now().to_rfc3339();

    let key = &contents.api_keys[index];
    Ok(to_summary(
        key,
        contents_provider_name(contents, &key.provider_id),
    ))
}

/// 删除 API Key
pub fn delete_key(contents: &mut VaultContents, id: &str) -> Result<(), AppError> {
    let before = contents.api_keys.len();
    contents.api_keys.retain(|k| k.id != id);
    if contents.api_keys.len() == before {
        return Err(AppError::NotFound(format!("API Key 不存在: {id}")));
    }
    Ok(())
}

/// 解密单条 API Key 的明文值，并记录最后使用时间
pub fn reveal_value(
    contents: &mut VaultContents,
    id: &str,
    vault_key: &[u8],
) -> Result<String, AppError> {
    let index = contents
        .api_keys
        .iter()
        .position(|k| k.id == id)
        .ok_or_else(|| AppError::NotFound(format!("API Key 不存在: {id}")))?;
    let plaintext = CryptoService::decrypt(vault_key, &contents.api_keys[index].encrypted_value)?;
    let value = String::from_utf8(plaintext)
        .map_err(|_| AppError::CryptoError("密钥内容不是合法 UTF-8".into()))?;
    contents.api_keys[index].last_used_at = Some(Utc::now().to_rfc3339());
    Ok(value)
}

/// 列出全部 Key 摘要（含服务商名称）
pub fn list_summaries(contents: &VaultContents) -> Vec<ApiKeySummary> {
    contents
        .api_keys
        .iter()
        .map(|k| to_summary(k, contents_provider_name(contents, &k.provider_id)))
        .collect()
}

/// 计算仪表盘统计：近 7 天内使用过的 Key 数计入 recent_usage
pub fn dashboard_stats(contents: &VaultContents, total_providers: usize) -> DashboardStats {
    let threshold = Utc::now() - Duration::days(7);
    let recent_usage = contents
        .api_keys
        .iter()
        .filter(|k| {
            k.last_used_at
                .as_deref()
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .is_some_and(|t| t >= threshold)
        })
        .count();
    DashboardStats {
        total_keys: contents.api_keys.len(),
        total_providers,
        recent_usage,
    }
}

fn to_summary(key: &ApiKey, provider_name: String) -> ApiKeySummary {
    ApiKeySummary {
        id: key.id.clone(),
        name: key.name.clone(),
        provider_id: key.provider_id.clone(),
        provider_name,
        tags: key.tags.clone(),
        created_at: key.created_at.clone(),
        updated_at: key.updated_at.clone(),
        last_used_at: key.last_used_at.clone(),
    }
}

/// 查找服务商展示名：自定义优先，其次内置，最后回退为 id
fn contents_provider_name(contents: &VaultContents, provider_id: &str) -> String {
    let all = provider_service::merged_providers(&contents.custom_providers);
    provider_service::find_provider(&all, provider_id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| provider_id.to_string())
}

fn normalize_tags(tags: &Option<Vec<String>>) -> Vec<String> {
    tags.clone()
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key_value() -> CreateApiKeyRequest {
        CreateApiKeyRequest {
            name: "测试 Key".into(),
            provider_id: "openai".into(),
            value: "sk-abc123".into(),
            tags: Some(vec![" 开发 ".into(), "".into()]),
        }
    }

    #[test]
    fn test_create_and_list_roundtrip() {
        let mut contents = VaultContents::default();
        let key = [1u8; 32];
        let summary = create_key(
            &mut contents,
            Some("^sk-[a-zA-Z0-9]+$"),
            &test_key_value(),
            &key,
        )
        .unwrap();
        assert_eq!(summary.tags, vec!["开发"]);
        assert_eq!(summary.provider_name, "OpenAI");
        assert_eq!(list_summaries(&contents).len(), 1);
    }

    #[test]
    fn test_create_rejects_bad_format() {
        let mut contents = VaultContents::default();
        let key = [1u8; 32];
        let req = CreateApiKeyRequest {
            value: "bad-value".into(),
            ..test_key_value()
        };
        assert!(matches!(
            create_key(&mut contents, Some("^sk-[a-zA-Z0-9]+$"), &req, &key),
            Err(AppError::ValidationError(_))
        ));
    }

    #[test]
    fn test_reveal_value_and_track_usage() {
        let mut contents = VaultContents::default();
        let key = [1u8; 32];
        let summary = create_key(&mut contents, None, &test_key_value(), &key).unwrap();
        assert!(contents.api_keys[0].last_used_at.is_none());
        let value = reveal_value(&mut contents, &summary.id, &key).unwrap();
        assert_eq!(value, "sk-abc123");
        assert!(contents.api_keys[0].last_used_at.is_some());
    }

    #[test]
    fn test_delete_missing_returns_not_found() {
        let mut contents = VaultContents::default();
        assert!(matches!(
            delete_key(&mut contents, "nope"),
            Err(AppError::NotFound(_))
        ));
    }
}
