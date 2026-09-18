use std::sync::Mutex;

use tauri::Manager;

mod commands;
mod error;
mod models;
mod services;
mod storage;

use services::storage::Platform;
use services::vault_service::VaultService;
use storage::local_storage::LocalStorage;

/// 应用全局状态：保险库服务置于 Mutex 下供 command 层共享
pub struct AppState {
    vault_service: Mutex<VaultService>,
}

impl AppState {
    /// 基于具体存储实现构建全局状态
    pub fn new(storage: LocalStorage) -> Self {
        Self {
            vault_service: Mutex::new(VaultService::new(Box::new(storage))),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            // 按当前平台选择存储策略，桌面端落盘于 app_data_dir
            let storage = LocalStorage::for_platform(Platform::current(), data_dir)?;
            app.manage(AppState::new(storage));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 应用级
            commands::app_commands::get_app_version,
            commands::app_commands::health_check,
            commands::app_commands::get_vault_status,
            commands::app_commands::initialize_vault,
            commands::app_commands::unlock_vault,
            commands::app_commands::lock_vault,
            commands::app_commands::get_settings,
            commands::app_commands::update_settings,
            commands::app_commands::get_dashboard_stats,
            // API Key
            commands::key_commands::create_api_key,
            commands::key_commands::list_api_keys,
            commands::key_commands::update_api_key,
            commands::key_commands::delete_api_key,
            commands::key_commands::reveal_api_key,
            commands::key_commands::test_key_connection,
            // 服务商
            commands::provider_commands::list_providers,
            commands::provider_commands::add_custom_provider,
            commands::provider_commands::update_provider,
            commands::provider_commands::delete_provider,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
