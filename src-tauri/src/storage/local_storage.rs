use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::services::storage::{Platform, Storage};

/// 保险库数据文件名
pub const VAULT_FILE: &str = "vault.json";

/// 本地加密文件存储实现
///
/// 桌面端：数据以加密 JSON 信封存储于 Tauri `app_data_dir`。
/// 上层（VaultService）负责在写入前完成加密，本层只存取字节。
pub struct LocalStorage {
    path: PathBuf,
}

impl LocalStorage {
    /// 指定数据文件完整路径创建存储
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// 以 `app_data_dir` 为根目录构建保险库文件存储
    pub fn in_app_data(app_data_dir: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::StorageError(format!("创建数据目录失败: {e}")))?;
        Ok(Self::new(app_data_dir.join(VAULT_FILE)))
    }

    /// 根据平台选择存储策略（当前各平台均落盘于应用数据目录）
    pub fn for_platform(platform: Platform, app_data_dir: PathBuf) -> Result<Self, AppError> {
        match platform {
            Platform::Android | Platform::IOS => {
                // TODO: 移动端接入平台原生安全文件系统 API
                Self::in_app_data(app_data_dir)
            }
            _ => Self::in_app_data(app_data_dir),
        }
    }
}

impl Storage for LocalStorage {
    /// 原子写入：先写临时文件再重命名，避免中途崩溃损坏数据
    fn save(&self, data: &[u8]) -> Result<(), AppError> {
        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, data)
            .map_err(|e| AppError::StorageError(format!("写入数据文件失败: {e}")))?;
        fs::rename(&tmp_path, &self.path)
            .map_err(|e| AppError::StorageError(format!("数据文件替换失败: {e}")))
    }

    fn load(&self) -> Result<Vec<u8>, AppError> {
        if !self.exists() {
            return Err(AppError::NotFound("保险库文件不存在".into()));
        }
        fs::read(&self.path).map_err(|e| AppError::StorageError(format!("读取数据文件失败: {e}")))
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn delete(&self) -> Result<(), AppError> {
        if !self.exists() {
            return Ok(());
        }
        fs::remove_file(&self.path)
            .map_err(|e| AppError::StorageError(format!("删除数据文件失败: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_vault_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("dragon-vault-test-{name}.json"))
    }

    #[test]
    fn test_save_load_roundtrip() {
        let storage = LocalStorage::new(temp_vault_path("roundtrip"));
        let _ = storage.delete();
        storage.save(b"encrypted-bytes").unwrap();
        assert!(storage.exists());
        assert_eq!(storage.load().unwrap(), b"encrypted-bytes");
        storage.delete().unwrap();
        assert!(!storage.exists());
    }

    #[test]
    fn test_load_missing_returns_not_found() {
        let storage = LocalStorage::new(temp_vault_path("missing"));
        let _ = storage.delete();
        assert!(matches!(storage.load(), Err(AppError::NotFound(_))));
    }
}
