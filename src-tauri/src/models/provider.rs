use serde::{Deserialize, Serialize};

/// 模型服务商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub key_format_pattern: Option<String>,
    pub models: Vec<String>,
    pub is_built_in: bool,
}
