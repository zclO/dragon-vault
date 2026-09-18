use crate::error::AppError;

/// 存储层抽象 trait
///
/// 桌面端使用本地加密文件存储（LocalStorage），
/// 移动端使用系统级安全存储（Android Keystore / iOS Keychain）。
pub trait Storage {
    /// 保存加密数据
    fn save(&self, data: &[u8]) -> Result<(), AppError>;

    /// 加载加密数据
    fn load(&self) -> Result<Vec<u8>, AppError>;

    /// 检查数据是否存在
    fn exists(&self) -> bool;

    /// 删除存储数据
    fn delete(&self) -> Result<(), AppError>;
}

/// 安全密钥存储 trait
///
/// 用于存储主密码哈希、加密密钥等敏感凭据。
/// 桌面端使用文件 + 系统 DPAPI/CryptoAPI，
/// 移动端使用硬件级安全存储。
pub trait SecureKeyStore {
    /// 存储密钥（平台安全区域）
    fn store_key(&self, key_id: &str, data: &[u8]) -> Result<(), AppError>;

    /// 读取密钥
    fn retrieve_key(&self, key_id: &str) -> Result<Vec<u8>, AppError>;

    /// 删除密钥
    fn remove_key(&self, key_id: &str) -> Result<(), AppError>;

    /// 检查密钥是否存在
    fn has_key(&self, key_id: &str) -> bool;
}

/// 当前运行平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
}

impl Platform {
    /// 获取当前运行平台
    pub fn current() -> Self {
        #[cfg(target_os = "android")]
        return Platform::Android;
        #[cfg(target_os = "ios")]
        return Platform::IOS;
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        #[cfg(target_os = "linux")]
        return Platform::Linux;
    }

    /// 是否为移动端平台
    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::Android | Platform::IOS)
    }

    /// 是否为桌面端平台
    pub fn is_desktop(&self) -> bool {
        !self.is_mobile()
    }
}
