pub mod app_commands;
pub mod biometric_commands;
pub mod key_commands;
pub mod provider_commands;

use std::sync::MutexGuard;

use tauri::State;

use crate::error::AppError;
use crate::services::vault_service::VaultService;
use crate::AppState;

/// 获取 VaultService 的锁守卫；所有 command 统一入口
///
/// 中毒锁映射为 `AppError::Internal`，提示用户重启应用。
pub fn vault_guard<'a>(
    state: &'a State<'a, AppState>,
) -> Result<MutexGuard<'a, VaultService>, AppError> {
    state
        .vault_service
        .lock()
        .map_err(|_| AppError::Internal("应用状态异常，请重启应用".into()))
}
