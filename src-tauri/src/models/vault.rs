use serde::{Deserialize, Serialize};

use crate::models::api_key::ApiKey;
use crate::models::provider::ProviderConfig;
use crate::models::settings::AppSettings;

/// 保险库磁盘信封（JSON 格式，payload/verifier 均为密文 Base64）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultEnvelope {
    /// 格式版本号，为后续数据迁移预留
    pub version: u32,
    /// Argon2id 盐值（Base64）
    pub salt: String,
    /// 用派生密钥加密的固定校验串（Base64），用于验证主密码
    pub verifier: String,
    /// 加密后的 VaultContents（Base64）
    pub payload: String,
}

/// 保险库解密后的内存 contents（仅解锁状态下存在）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultContents {
    pub api_keys: Vec<ApiKey>,
    pub custom_providers: Vec<ProviderConfig>,
    pub settings: AppSettings,
}

/// 保险库状态（前端据此决定展示初始化/解锁界面）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    /// 是否已初始化（磁盘上存在保险库文件）
    pub initialized: bool,
    /// 当前会话是否已解锁
    pub unlocked: bool,
}

/// 创建 API Key 的请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub provider_id: String,
    pub value: String,
    pub tags: Option<Vec<String>>,
}

/// 更新 API Key 的请求参数（字段为 None 时保持不变）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiKeyRequest {
    pub id: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 仪表盘统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_keys: usize,
    pub total_providers: usize,
    pub recent_usage: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_serde_camel_case() {
        let envelope = VaultEnvelope {
            version: 1,
            salt: "abc".into(),
            verifier: "def".into(),
            payload: "ghi".into(),
        };
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"salt\""));
        let back: VaultEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert_eq!(back.payload, "ghi");
    }

    #[test]
    fn test_contents_default_is_empty() {
        let contents = VaultContents::default();
        assert!(contents.api_keys.is_empty());
        assert!(contents.custom_providers.is_empty());
    }
}
