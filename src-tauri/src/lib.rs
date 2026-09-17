mod commands;
mod error;
mod models;
mod services;
mod storage;

/// 应用全局状态
pub struct AppState {
    // TODO: 初始化加密服务和存储
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_commands::get_app_version,
            commands::app_commands::health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
