use tauri::State;

use crate::error::AppError;
use crate::models::settings::AppSettings;
use crate::models::vault::{DashboardStats, VaultStatus};
use crate::AppState;

/// 获取应用版本信息
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 健康检查
#[tauri::command]
pub fn health_check(_state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(true)
}

/// 查询保险库状态（是否已初始化 / 已解锁）
#[tauri::command]
pub fn get_vault_status(state: State<'_, AppState>) -> Result<VaultStatus, AppError> {
    Ok(crate::commands::vault_guard(&state)?.status())
}

/// 首次初始化保险库并设置主密码
#[tauri::command]
pub fn initialize_vault(state: State<'_, AppState>, password: String) -> Result<(), AppError> {
    crate::commands::vault_guard(&state)?.initialize(&password)
}

/// 主密码解锁保险库
#[tauri::command]
pub fn unlock_vault(state: State<'_, AppState>, password: String) -> Result<(), AppError> {
    crate::commands::vault_guard(&state)?.unlock(&password)
}

/// 锁定保险库并清除内存中的密钥
#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), AppError> {
    crate::commands::vault_guard(&state)?.lock();
    Ok(())
}

/// 获取应用设置
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    crate::commands::vault_guard(&state)?.get_settings()
}

/// 保存应用设置
#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, AppError> {
    crate::commands::vault_guard(&state)?.update_settings(&settings)
}

/// 获取仪表盘统计数据
#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, AppError> {
    Ok(crate::commands::vault_guard(&state)?.dashboard_stats())
}
