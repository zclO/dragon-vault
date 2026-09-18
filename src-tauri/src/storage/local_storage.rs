use crate::error::AppError;
use crate::services::storage::Storage;
use crate::services::storage::Platform;

/// 本地加密文件存储实现
///
/// 桌面端：数据以加密 JSON 文件存储于 Tauri app_data_dir
/// 移动端：可扩展为使用平台原生文件 API 或 SQLite + SQLCipher
pub struct LocalStorage {
    // TODO: 文件路径、加密服务引用
}

impl LocalStorage {
    pub fn new() -> Self {
        Self {}
    }

    /// 根据平台选择存储策略
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::Android | Platform::IOS => {
                // TODO: 移动端使用平台原生文件 API
                Self::new()
            }
            _ => {
                // TODO: 桌面端使用 Tauri app_data_dir
                Self::new()
            }
        }
    }
}

impl Storage for LocalStorage {
    fn save(&self, _data: &[u8]) -> Result<(), AppError> {
        // TODO: 实现加密文件写入
        Err(AppError::StorageError("LocalStorage not yet implemented".into()))
    }

    fn load(&self) -> Result<Vec<u8>, AppError> {
        // TODO: 实现加密文件读取
        Err(AppError::StorageError("LocalStorage not yet implemented".into()))
    }

    fn exists(&self) -> bool {
        // TODO: 检查数据文件是否存在
        false
    }

    fn delete(&self) -> Result<(), AppError> {
        // TODO: 安全删除数据文件
        Err(AppError::StorageError("LocalStorage not yet implemented".into()))
    }
}
