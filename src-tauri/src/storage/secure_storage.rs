use crate::error::AppError;
use crate::services::storage::SecureKeyStore;

/// 安全密钥存储 — 桌面端实现
///
/// 桌面端使用加密文件存储密钥。
/// Windows 上可结合 DPAPI，macOS/Linux 上使用文件级加密。
///
/// 移动端将使用平台原生安全存储：
/// - Android: Android Keystore System（硬件级 TEE/StrongBox）
/// - iOS: Keychain Services（Secure Enclave）
pub struct DesktopSecureKeyStore {
    // TODO: 密钥文件路径或系统凭据句柄
}

impl DesktopSecureKeyStore {
    pub fn new() -> Self {
        Self {}
    }
}

impl SecureKeyStore for DesktopSecureKeyStore {
    fn store_key(&self, _key_id: &str, _data: &[u8]) -> Result<(), AppError> {
        // TODO: 实现桌面端密钥存储
        Err(AppError::Internal("DesktopSecureKeyStore not yet implemented".into()))
    }

    fn retrieve_key(&self, _key_id: &str) -> Result<Vec<u8>, AppError> {
        // TODO: 实现桌面端密钥读取
        Err(AppError::Internal("DesktopSecureKeyStore not yet implemented".into()))
    }

    fn remove_key(&self, _key_id: &str) -> Result<(), AppError> {
        // TODO: 实现桌面端密钥删除
        Err(AppError::Internal("DesktopSecureKeyStore not yet implemented".into()))
    }

    fn has_key(&self, _key_id: &str) -> bool {
        // TODO: 实现密钥存在性检查
        false
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
        Err(AppError::Internal("MobileSecureKeyStore not yet implemented".into()))
    }

    fn retrieve_key(&self, _key_id: &str) -> Result<Vec<u8>, AppError> {
        Err(AppError::Internal("MobileSecureKeyStore not yet implemented".into()))
    }

    fn remove_key(&self, _key_id: &str) -> Result<(), AppError> {
        Err(AppError::Internal("MobileSecureKeyStore not yet implemented".into()))
    }

    fn has_key(&self, _key_id: &str) -> bool {
        false
    }
}
