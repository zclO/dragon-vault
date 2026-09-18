use tauri::State;

use crate::error::AppError;
use crate::models::provider::ProviderConfig;
use crate::AppState;

/// 列出全部服务商（内置 + 自定义）
#[tauri::command]
pub fn list_providers(state: State<'_, AppState>) -> Result<Vec<ProviderConfig>, AppError> {
    crate::commands::vault_guard(&state)?.list_providers()
}

/// 添加自定义服务商
#[tauri::command]
pub fn add_custom_provider(
    state: State<'_, AppState>,
    provider: ProviderConfig,
) -> Result<ProviderConfig, AppError> {
    crate::commands::vault_guard(&state)?.add_custom_provider(&provider)
}

/// 更新自定义服务商（内置服务商只读）
#[tauri::command]
pub fn update_provider(
    state: State<'_, AppState>,
    provider: ProviderConfig,
) -> Result<ProviderConfig, AppError> {
    crate::commands::vault_guard(&state)?.update_provider(&provider)
}

/// 删除自定义服务商
#[tauri::command]
pub fn delete_provider(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    crate::commands::vault_guard(&state)?.delete_provider(&id)
}
