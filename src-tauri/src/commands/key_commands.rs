use tauri::State;

use crate::error::AppError;
use crate::models::api_key::ApiKeySummary;
use crate::models::vault::{CreateApiKeyRequest, UpdateApiKeyRequest};
use crate::AppState;

/// 创建 API Key（值经 AES-256-GCM 加密后落盘）
#[tauri::command]
pub fn create_api_key(
    state: State<'_, AppState>,
    request: CreateApiKeyRequest,
) -> Result<ApiKeySummary, AppError> {
    crate::commands::vault_guard(&state)?.create_api_key(&request)
}

/// 列出全部 API Key 摘要（不含明文值）
#[tauri::command]
pub fn list_api_keys(state: State<'_, AppState>) -> Result<Vec<ApiKeySummary>, AppError> {
    crate::commands::vault_guard(&state)?.list_api_keys()
}

/// 更新 API Key 的名称 / 值 / 标签
#[tauri::command]
pub fn update_api_key(
    state: State<'_, AppState>,
    request: UpdateApiKeyRequest,
) -> Result<ApiKeySummary, AppError> {
    crate::commands::vault_guard(&state)?.update_api_key(&request)
}

/// 删除 API Key
#[tauri::command]
pub fn delete_api_key(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    crate::commands::vault_guard(&state)?.delete_api_key(&id)
}

/// 解密并返回 API Key 明文值（用于复制，同时记录使用时间）
#[tauri::command]
pub fn reveal_api_key(state: State<'_, AppState>, id: String) -> Result<String, AppError> {
    crate::commands::vault_guard(&state)?.reveal_api_key(&id)
}

/// 测试 API Key 连通性：向服务商 Models 接口发起轻量请求
#[tauri::command]
pub async fn test_key_connection(state: State<'_, AppState>, id: String) -> Result<bool, AppError> {
    // 同步段：取出密钥值与服务商配置（不能跨 await 持有锁）
    let (value, provider) = {
        let svc = crate::commands::vault_guard(&state)?;
        svc.key_for_test(&id)?
    };

    let url = format!("{}/models", provider.base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let request = match provider.id.as_str() {
        "anthropic" => client
            .get(&url)
            .header("x-api-key", value)
            .header("anthropic-version", "2023-06-01"),
        "google" => client.get(&url).header("x-goog-api-key", value),
        _ => client.get(&url).bearer_auth(value),
    };

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("网络请求失败: {}", e)))?;
    Ok(response.status().is_success())
}
