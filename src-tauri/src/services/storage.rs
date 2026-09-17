use crate::error::AppError;

/// 存储层抽象 trait
pub trait Storage {
    fn save(&self, data: &[u8]) -> Result<(), AppError>;
    fn load(&self) -> Result<Vec<u8>, AppError>;
    fn exists(&self) -> bool;
}
