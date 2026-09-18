use thiserror::Error;

/// 应用统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("加密失败: {0}")]
    CryptoError(String),

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("数据未找到: {0}")]
    NotFound(String),

    #[error("参数无效: {0}")]
    ValidationError(String),

    #[error("认证失败: 主密码错误")]
    AuthError,

    #[error("保险库已锁定，请先解锁")]
    Locked,

    #[error("内部错误: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
