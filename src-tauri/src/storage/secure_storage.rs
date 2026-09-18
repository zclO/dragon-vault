use std::fs;
use std::path::PathBuf;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;

use crate::error::AppError;
use crate::services::storage::SecureKeyStore;

/// 安全密钥存储 — 桌面端实现
///
/// 密钥以 Base64 文件形式存储于 `app_data_dir/keystore/` 目录下，
/// 每个 key_id 对应一个文件。Windows 上可后续结合 DPAPI 加密落盘，
/// 当前主密钥仅在内存中派生使用、不持久化，此存储用于非派生类凭据。
// 预留实现：接入 SecureKeyStore 调用方前不报 dead_code
#[allow(dead_code)]
pub struct DesktopSecureKeyStore {
    dir: PathBuf,
}

#[allow(dead_code)]
impl DesktopSecureKeyStore {
    /// 以 `app_data_dir` 为根目录创建密钥存储
    pub fn in_app_data(app_data_dir: PathBuf) -> Result<Self, AppError> {
        let dir = app_data_dir.join("keystore");
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::StorageError(format!("创建密钥目录失败: {e}")))?;
        Ok(Self { dir })
    }

    /// key_id 直接作为文件名，需防止路径穿越
    fn key_path(&self, key_id: &str) -> Result<PathBuf, AppError> {
        if key_id.is_empty()
            || !key_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AppError::ValidationError(format!(
                "非法的密钥标识: {key_id}"
            )));
        }
        Ok(self.dir.join(format!("{key_id}.key")))
    }
}

#[allow(dead_code)]
impl SecureKeyStore for DesktopSecureKeyStore {
    fn store_key(&self, key_id: &str, data: &[u8]) -> Result<(), AppError> {
        let path = self.key_path(key_id)?;
        fs::write(&path, BASE64.encode(data))
            .map_err(|e| AppError::StorageError(format!("写入密钥失败: {e}")))
    }

    fn retrieve_key(&self, key_id: &str) -> Result<Vec<u8>, AppError> {
        let path = self.key_path(key_id)?;
        if !path.exists() {
            return Err(AppError::NotFound(format!("密钥不存在: {key_id}")));
        }
        let encoded = fs::read_to_string(&path)
            .map_err(|e| AppError::StorageError(format!("读取密钥失败: {e}")))?;
        BASE64
            .decode(encoded.trim())
            .map_err(|e| AppError::StorageError(format!("密钥解码失败: {e}")))
    }

    fn remove_key(&self, key_id: &str) -> Result<(), AppError> {
        let path = self.key_path(key_id)?;
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| AppError::StorageError(format!("删除密钥失败: {e}")))?;
        }
        Ok(())
    }

    fn has_key(&self, key_id: &str) -> bool {
        self.key_path(key_id).map(|p| p.exists()).unwrap_or(false)
    }
}

/// 移动端安全密钥存储占位
///
/// 编译条件：仅在 Android / iOS 目标下编译
/// - Android: 通过 JNI 调用 Android Keystore
/// - iOS: 通过 FFI 调用 Keychain / Secure Enclave
#[cfg(any(target_os = "android", target_os = "ios"))]
pub struct MobileSecureKeyStore {
    // TODO: 平台安全存储句柄
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl MobileSecureKeyStore {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl SecureKeyStore for MobileSecureKeyStore {
    fn store_key(&self, _key_id: &str, _data: &[u8]) -> Result<(), AppError> {
        // TODO: 调用平台原生安全存储
        Err(AppError::Internal(
            "MobileSecureKeyStore not yet implemented".into(),
        ))
    }

    fn retrieve_key(&self, _key_id: &str) -> Result<Vec<u8>, AppError> {
        Err(AppError::Internal(
            "MobileSecureKeyStore not yet implemented".into(),
        ))
    }

    fn remove_key(&self, _key_id: &str) -> Result<(), AppError> {
        Err(AppError::Internal(
            "MobileSecureKeyStore not yet implemented".into(),
        ))
    }

    fn has_key(&self, _key_id: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> DesktopSecureKeyStore {
        let dir = std::env::temp_dir().join(format!("dragon-vault-keystore-{name}"));
        DesktopSecureKeyStore::in_app_data(dir).unwrap()
    }

    #[test]
    fn test_store_retrieve_remove() {
        let store = temp_store("roundtrip");
        let _ = store.remove_key("test-key");
        store.store_key("test-key", b"secret-bytes").unwrap();
        assert!(store.has_key("test-key"));
        assert_eq!(store.retrieve_key("test-key").unwrap(), b"secret-bytes");
        store.remove_key("test-key").unwrap();
        assert!(!store.has_key("test-key"));
    }

    #[test]
    fn test_reject_path_traversal_id() {
        let store = temp_store("traversal");
        assert!(matches!(
            store.store_key("../evil", b"x"),
            Err(AppError::ValidationError(_))
        ));
    }
}
