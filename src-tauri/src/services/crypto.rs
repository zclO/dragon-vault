use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::error::AppError;

/// Argon2id 盐值长度（字节）
pub const SALT_LEN: usize = 16;
/// AES-GCM Nonce 长度（字节）
const NONCE_LEN: usize = 12;
/// 派生密钥长度（AES-256）
const KEY_LEN: usize = 32;
/// 主密码校验串的明文，加密后存于信封中用于验证密码
const VERIFIER_TOKEN: &[u8] = b"dragon-vault-master-password-check";

/// AES-256-GCM 加密 / Argon2id 密钥派生服务（无状态，全部为关联函数）
pub struct CryptoService;

impl CryptoService {
    /// 生成随机盐值
    pub fn random_salt() -> Vec<u8> {
        let mut salt = vec![0u8; SALT_LEN];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        salt
    }

    /// 使用 Argon2id 从主密码派生 256 位加密密钥
    pub fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<Vec<u8>>, AppError> {
        let mut output = vec![0u8; KEY_LEN];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut output)
            .map_err(|e| AppError::CryptoError(e.to_string()))?;
        Ok(Zeroizing::new(output))
    }

    /// 加密明文，输出格式为 `nonce || ciphertext`
    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
        let cipher = Self::cipher(key)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| AppError::CryptoError("加密操作失败".into()))?;
        let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// 解密 `nonce || ciphertext` 格式的密文
    pub fn decrypt(key: &[u8], data: &[u8]) -> Result<Vec<u8>, AppError> {
        if data.len() <= NONCE_LEN {
            return Err(AppError::CryptoError("密文数据不完整".into()));
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let cipher = Self::cipher(key)?;
        cipher
            .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|_| AppError::CryptoError("解密失败，数据可能已损坏".into()))
    }

    /// 生成主密码校验串（用派生密钥加密固定明文）
    pub fn create_verifier(key: &[u8]) -> Result<Vec<u8>, AppError> {
        Self::encrypt(key, VERIFIER_TOKEN)
    }

    /// 校验主密码：解密校验串并与已知明文比对
    pub fn verify_key(key: &[u8], verifier: &[u8]) -> bool {
        match Self::decrypt(key, verifier) {
            Ok(plaintext) => plaintext.as_slice() == VERIFIER_TOKEN,
            Err(_) => false,
        }
    }

    /// 构造 AES-256-GCM 实例
    fn cipher(key: &[u8]) -> Result<Aes256Gcm, AppError> {
        Aes256Gcm::new_from_slice(key).map_err(|_| AppError::CryptoError("无效的密钥长度".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let salt = CryptoService::random_salt();
        let k1 = CryptoService::derive_key("correct horse", &salt).unwrap();
        let k2 = CryptoService::derive_key("correct horse", &salt).unwrap();
        assert_eq!(k1.as_slice(), k2.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [7u8; KEY_LEN];
        let plaintext = b"sk-test-1234567890";
        let encrypted = CryptoService::encrypt(&key, plaintext).unwrap();
        assert_ne!(&encrypted[NONCE_LEN..], plaintext.as_slice());
        let decrypted = CryptoService::decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext.to_vec());
    }

    #[test]
    fn test_same_plaintext_produces_different_ciphertext() {
        let key = [7u8; KEY_LEN];
        let a = CryptoService::encrypt(&key, b"same").unwrap();
        let b = CryptoService::encrypt(&key, b"same").unwrap();
        assert_ne!(a, b, "每次加密必须使用随机 Nonce");
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let encrypted = CryptoService::encrypt(&[1u8; KEY_LEN], b"secret").unwrap();
        let result = CryptoService::decrypt(&[2u8; KEY_LEN], &encrypted);
        assert!(matches!(result, Err(AppError::CryptoError(_))));
    }

    #[test]
    fn test_verify_key_accepts_only_correct_key() {
        let salt = CryptoService::random_salt();
        let key = CryptoService::derive_key("pa55w0rd", &salt).unwrap();
        let verifier = CryptoService::create_verifier(&key).unwrap();
        assert!(CryptoService::verify_key(&key, &verifier));

        let wrong = CryptoService::derive_key("wrong-pwd", &salt).unwrap();
        assert!(!CryptoService::verify_key(&wrong, &verifier));
    }
}
