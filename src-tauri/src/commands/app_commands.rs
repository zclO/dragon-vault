use tauri::State;
use crate::error::AppError;
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
