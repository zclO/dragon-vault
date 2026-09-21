use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use zeroize::Zeroizing;

use crate::error::AppError;
use crate::models::api_key::ApiKeySummary;
use crate::models::provider::ProviderConfig;
use crate::models::settings::AppSettings;
use crate::models::vault::{
    CreateApiKeyRequest, DashboardStats, UpdateApiKeyRequest, VaultContents, VaultEnvelope,
    VaultStatus,
};
use crate::services::storage::Storage;
use crate::services::{crypto::CryptoService, key_service, provider_service};

/// 保险库信封格式版本
const VAULT_VERSION: u32 = 1;
/// 首次设置主密码时的最短长度
const MIN_PASSWORD_LEN: usize = 8;

/// 解锁后的会话，持有派生密钥与解密后的保险库内容
struct VaultSession {
    key: Zeroizing<Vec<u8>>,
    salt: Vec<u8>,
    contents: VaultContents,
}

/// 保险库核心服务：主密码初始化/解锁/锁定，以及各业务操作的统一入口
///
/// 所有变更操作在成功后立即加密落盘（零明文存储）。
pub struct VaultService {
    storage: Box<dyn Storage + Send>,
    session: Option<VaultSession>,
}

impl VaultService {
    /// 注入具体存储实现
    pub fn new(storage: Box<dyn Storage + Send>) -> Self {
        Self {
            storage,
            session: None,
        }
    }

    /// 查询保险库状态
    pub fn status(&self) -> VaultStatus {
        VaultStatus {
            initialized: self.storage.exists(),
            unlocked: self.session.is_some(),
        }
    }

    /// 首次初始化：设置主密码并创建空保险库
    pub fn initialize(&mut self, password: &str) -> Result<(), AppError> {
        if self.storage.exists() {
            return Err(AppError::ValidationError(
                "保险库已存在，请勿重复初始化".into(),
            ));
        }
        validate_password_strength(password)?;
        let salt = CryptoService::random_salt();
        let key = CryptoService::derive_key(password, &salt)?;
        let contents = VaultContents::default();
        self.session = Some(VaultSession {
            key,
            salt,
            contents,
        });
        self.persist()
    }

    /// 主密码解锁：校验 verifier 后解密保险库内容到内存
    pub fn unlock(&mut self, password: &str) -> Result<(), AppError> {
        let envelope = self.load_envelope()?;
        let salt = BASE64
            .decode(&envelope.salt)
            .map_err(|e| AppError::StorageError(format!("盐值解码失败: {e}")))?;
        let key = CryptoService::derive_key(password, &salt)?;
        self.finish_unlock(envelope, key)
    }

    /// 使用已派生的主密钥解锁（指纹解锁取回封存密钥后走此入口）
    pub fn unlock_with_key(&mut self, key: Zeroizing<Vec<u8>>) -> Result<(), AppError> {
        let envelope = self.load_envelope()?;
        self.finish_unlock(envelope, key)
    }

    /// 解锁收尾：验证校验串 → 解密 payload → 建立会话
    fn finish_unlock(
        &mut self,
        envelope: VaultEnvelope,
        key: Zeroizing<Vec<u8>>,
    ) -> Result<(), AppError> {
        let salt = BASE64
            .decode(&envelope.salt)
            .map_err(|e| AppError::StorageError(format!("盐值解码失败: {e}")))?;
        let verifier = BASE64
            .decode(&envelope.verifier)
            .map_err(|e| AppError::StorageError(format!("校验串解码失败: {e}")))?;
        if !CryptoService::verify_key(&key, &verifier) {
            return Err(AppError::AuthError);
        }
        let payload = BASE64
            .decode(&envelope.payload)
            .map_err(|e| AppError::StorageError(format!("数据解码失败: {e}")))?;
        let plaintext = CryptoService::decrypt(&key, &payload)?;
        let contents: VaultContents = serde_json::from_slice(&plaintext)
            .map_err(|e| AppError::StorageError(format!("保险库数据损坏: {e}")))?;
        self.session = Some(VaultSession {
            key,
            salt,
            contents,
        });
        Ok(())
    }

    /// 当前会话派生密钥的只读访问（供生物解锁封存用）
    pub fn session_key(&self) -> Result<Zeroizing<Vec<u8>>, AppError> {
        Ok(self.require_session()?.key.clone())
    }

    /// 立即锁定并清除内存中的密钥与数据
    pub fn lock(&mut self) {
        self.session = None;
    }

    // ---------- API Key 操作 ----------

    /// 列出全部 Key 摘要（需已解锁）
    pub fn list_api_keys(&self) -> Result<Vec<ApiKeySummary>, AppError> {
        let session = self.require_session()?;
        Ok(key_service::list_summaries(&session.contents))
    }

    /// 创建 Key
    pub fn create_api_key(&mut self, req: &CreateApiKeyRequest) -> Result<ApiKeySummary, AppError> {
        let session = self.require_session_mut()?;
        let providers = provider_service::merged_providers(&session.contents.custom_providers);
        let provider = provider_service::find_provider(&providers, &req.provider_id)
            .ok_or_else(|| AppError::NotFound(format!("服务商不存在: {}", req.provider_id)))?;
        let pattern = provider.key_format_pattern.clone();
        let summary =
            key_service::create_key(&mut session.contents, pattern.as_deref(), req, &session.key)?;
        self.persist()?;
        Ok(summary)
    }

    /// 更新 Key
    pub fn update_api_key(&mut self, req: &UpdateApiKeyRequest) -> Result<ApiKeySummary, AppError> {
        let session = self.require_session_mut()?;
        let provider_id = session
            .contents
            .api_keys
            .iter()
            .find(|k| k.id == req.id)
            .map(|k| k.provider_id.clone())
            .ok_or_else(|| AppError::NotFound(format!("API Key 不存在: {}", req.id)))?;
        let providers = provider_service::merged_providers(&session.contents.custom_providers);
        let pattern = provider_service::find_provider(&providers, &provider_id)
            .and_then(|p| p.key_format_pattern.clone());
        let summary = key_service::update_key(
            &mut session.contents,
            pattern.as_deref(),
            &req.id,
            req.name.as_deref(),
            req.value.as_deref(),
            req.tags.clone(),
            &session.key,
        )?;
        self.persist()?;
        Ok(summary)
    }

    /// 删除 Key
    pub fn delete_api_key(&mut self, id: &str) -> Result<(), AppError> {
        let session = self.require_session_mut()?;
        key_service::delete_key(&mut session.contents, id)?;
        self.persist()
    }

    /// 解密并返回 Key 明文值（同时记录使用时间）
    pub fn reveal_api_key(&mut self, id: &str) -> Result<String, AppError> {
        let session = self.require_session_mut()?;
        let value = key_service::reveal_value(&mut session.contents, id, &session.key)?;
        self.persist()?;
        Ok(value)
    }

    /// 只读地取出 Key 值与所属服务商（用于连通性测试，不更新使用时间）
    pub fn key_for_test(&self, id: &str) -> Result<(String, ProviderConfig), AppError> {
        let session = self.require_session()?;
        let api_key = session
            .contents
            .api_keys
            .iter()
            .find(|k| k.id == id)
            .ok_or_else(|| AppError::NotFound(format!("API Key 不存在: {id}")))?;
        let providers = provider_service::merged_providers(&session.contents.custom_providers);
        let provider = provider_service::find_provider(&providers, &api_key.provider_id)
            .ok_or_else(|| AppError::NotFound(format!("服务商不存在: {}", api_key.provider_id)))?
            .clone();
        let plaintext = CryptoService::decrypt(&session.key, &api_key.encrypted_value)?;
        let value = String::from_utf8(plaintext)
            .map_err(|_| AppError::CryptoError("密钥内容不是合法 UTF-8".into()))?;
        Ok((value, provider))
    }

    // ---------- 服务商操作 ----------

    /// 内置 + 自定义服务商列表
    pub fn list_providers(&self) -> Result<Vec<ProviderConfig>, AppError> {
        let session = self.require_session()?;
        Ok(provider_service::merged_providers(
            &session.contents.custom_providers,
        ))
    }

    /// 添加自定义服务商
    pub fn add_custom_provider(
        &mut self,
        provider: &ProviderConfig,
    ) -> Result<ProviderConfig, AppError> {
        let session = self.require_session_mut()?;
        if provider.is_built_in {
            return Err(AppError::ValidationError("不能添加内置标记的服务商".into()));
        }
        provider_service::validate_custom_id(&provider.id)?;
        provider_service::validate_provider(provider)?;
        let providers = provider_service::merged_providers(&session.contents.custom_providers);
        if provider_service::find_provider(&providers, &provider.id).is_some() {
            return Err(AppError::ValidationError(format!(
                "服务商 ID 已存在: {}",
                provider.id
            )));
        }
        session.contents.custom_providers.push(provider.clone());
        self.persist()?;
        Ok(provider.clone())
    }

    /// 更新自定义服务商（内置服务商只读）
    pub fn update_provider(
        &mut self,
        provider: &ProviderConfig,
    ) -> Result<ProviderConfig, AppError> {
        let session = self.require_session_mut()?;
        provider_service::validate_provider(provider)?;
        let existing = session
            .contents
            .custom_providers
            .iter_mut()
            .find(|p| p.id == provider.id)
            .ok_or_else(|| {
                AppError::ValidationError("仅支持修改自定义服务商，内置服务商不可修改".into())
            })?;
        let mut updated = provider.clone();
        updated.is_built_in = false;
        *existing = updated.clone();
        self.persist()?;
        Ok(updated)
    }

    /// 删除自定义服务商（仍有 Key 引用时拒绝）
    pub fn delete_provider(&mut self, id: &str) -> Result<(), AppError> {
        let session = self.require_session_mut()?;
        if provider_service::find_provider(&provider_service::builtin_providers(), id).is_some() {
            return Err(AppError::ValidationError("内置服务商不可删除".into()));
        }
        let in_use = session
            .contents
            .api_keys
            .iter()
            .any(|k| k.provider_id == id);
        if in_use {
            return Err(AppError::ValidationError(
                "该服务商下仍有 API Key，无法删除".into(),
            ));
        }
        let before = session.contents.custom_providers.len();
        session.contents.custom_providers.retain(|p| p.id != id);
        if session.contents.custom_providers.len() == before {
            return Err(AppError::NotFound(format!("服务商不存在: {id}")));
        }
        self.persist()
    }

    // ---------- 设置与统计 ----------

    /// 获取应用设置
    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        Ok(self.require_session()?.contents.settings.clone())
    }

    /// 保存应用设置
    pub fn update_settings(&mut self, settings: &AppSettings) -> Result<AppSettings, AppError> {
        let session = self.require_session_mut()?;
        session.contents.settings = settings.clone();
        self.persist()?;
        Ok(settings.clone())
    }

    /// 仪表盘统计（未解锁时返回全零，便于前端展示登录前界面）
    pub fn dashboard_stats(&self) -> DashboardStats {
        match &self.session {
            Some(session) => {
                let providers =
                    provider_service::merged_providers(&session.contents.custom_providers).len();
                key_service::dashboard_stats(&session.contents, providers)
            }
            None => DashboardStats {
                total_keys: 0,
                total_providers: 0,
                recent_usage: 0,
            },
        }
    }

    // ---------- 内部辅助 ----------

    fn require_session(&self) -> Result<&VaultSession, AppError> {
        self.session.as_ref().ok_or(AppError::Locked)
    }

    fn require_session_mut(&mut self) -> Result<&mut VaultSession, AppError> {
        match self.session.as_mut() {
            Some(session) => Ok(session),
            None => Err(AppError::Locked),
        }
    }

    fn load_envelope(&self) -> Result<VaultEnvelope, AppError> {
        let bytes = self.storage.load()?;
        serde_json::from_slice(&bytes)
            .map_err(|e| AppError::StorageError(format!("保险库文件格式错误: {e}")))
    }

    /// 将当前会话内容加密写入存储（沿用原始盐值，保持派生密钥稳定）
    fn persist(&self) -> Result<(), AppError> {
        let session = self.require_session()?;
        let plaintext = serde_json::to_vec(&session.contents)
            .map_err(|e| AppError::Internal(format!("序列化失败: {e}")))?;
        let payload = CryptoService::encrypt(&session.key, &plaintext)?;
        let verifier = CryptoService::create_verifier(&session.key)?;
        let envelope = VaultEnvelope {
            version: VAULT_VERSION,
            salt: BASE64.encode(&session.salt),
            verifier: BASE64.encode(verifier),
            payload: BASE64.encode(payload),
        };
        self.storage.save(
            &serde_json::to_vec_pretty(&envelope)
                .map_err(|e| AppError::Internal(format!("信封序列化失败: {e}")))?,
        )
    }
}

/// 主密码强度校验
fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::ValidationError(format!(
            "主密码至少需要 {MIN_PASSWORD_LEN} 个字符"
        )));
    }
    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_letter || !has_digit {
        return Err(AppError::ValidationError(
            "主密码需同时包含字母和数字".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::local_storage::LocalStorage;

    struct MemStorage {
        data: std::sync::Mutex<Option<Vec<u8>>>,
    }

    impl MemStorage {
        fn new() -> Self {
            Self {
                data: std::sync::Mutex::new(None),
            }
        }
    }

    impl Storage for MemStorage {
        fn save(&self, data: &[u8]) -> Result<(), AppError> {
            *self.data.lock().unwrap() = Some(data.to_vec());
            Ok(())
        }
        fn load(&self) -> Result<Vec<u8>, AppError> {
            self.data
                .lock()
                .unwrap()
                .clone()
                .ok_or(AppError::NotFound("空".into()))
        }
        fn exists(&self) -> bool {
            self.data.lock().unwrap().is_some()
        }
        fn delete(&self) -> Result<(), AppError> {
            *self.data.lock().unwrap() = None;
            Ok(())
        }
    }

    fn service() -> VaultService {
        VaultService::new(Box::new(MemStorage::new()))
    }

    fn disk_storage(name: &str) -> LocalStorage {
        let path = std::env::temp_dir().join(format!("dragon-vault-vsvc-{name}.json"));
        LocalStorage::new(path)
    }

    #[test]
    fn test_initialize_unlock_and_crud() {
        let mut svc = service();
        assert!(!svc.status().initialized);
        svc.initialize("password123").unwrap();
        assert!(svc.status().initialized);
        assert!(svc.status().unlocked);

        let summary = svc
            .create_api_key(&CreateApiKeyRequest {
                name: "主 Key".into(),
                provider_id: "openai".into(),
                value: "sk-abc123".into(),
                tags: None,
            })
            .unwrap();
        assert_eq!(svc.list_api_keys().unwrap().len(), 1);
        assert_eq!(svc.reveal_api_key(&summary.id).unwrap(), "sk-abc123");

        svc.lock();
        assert!(!svc.status().unlocked);
        svc.unlock("password123").unwrap();
        assert_eq!(svc.list_api_keys().unwrap().len(), 1);
    }

    #[test]
    fn test_wrong_password_returns_auth_error() {
        let mut svc = service();
        svc.initialize("password123").unwrap();
        svc.lock();
        assert!(matches!(svc.unlock("wrong12345"), Err(AppError::AuthError)));
        assert!(!svc.status().unlocked);
    }

    #[test]
    fn test_unlock_with_key_roundtrip() {
        let mut svc = service();
        svc.initialize("password123").unwrap();
        let key = svc.session_key().unwrap();
        svc.lock();
        assert!(!svc.status().unlocked);
        svc.unlock_with_key(Zeroizing::new(key.to_vec())).unwrap();
        assert!(svc.status().unlocked);
    }

    #[test]
    fn test_unlock_with_wrong_key_rejected() {
        let mut svc = service();
        svc.initialize("password123").unwrap();
        svc.lock();
        let bogus = Zeroizing::new(vec![0u8; 32]);
        assert!(matches!(
            svc.unlock_with_key(bogus),
            Err(AppError::AuthError)
        ));
        assert!(!svc.status().unlocked);
    }

    #[test]
    fn test_session_key_requires_unlocked() {
        let mut svc = service();
        svc.initialize("password123").unwrap();
        svc.lock();
        assert!(matches!(svc.session_key(), Err(AppError::Locked)));
    }

    #[test]
    fn test_locked_operations_rejected() {
        let mut svc = service();
        svc.initialize("password123").unwrap();
        svc.lock();
        assert!(matches!(svc.list_api_keys(), Err(AppError::Locked)));
    }

    #[test]
    fn test_weak_password_rejected() {
        let mut svc = service();
        assert!(matches!(
            svc.initialize("abc"),
            Err(AppError::ValidationError(_))
        ));
    }

    #[test]
    fn test_custom_provider_lifecycle() {
        let mut svc = service();
        svc.initialize("password123").unwrap();
        let provider = ProviderConfig {
            id: "my-llm".into(),
            name: "自建服务".into(),
            base_url: "https://llm.example.com/v1".into(),
            key_format_pattern: None,
            models: vec!["local-model".into()],
            is_built_in: false,
        };
        svc.add_custom_provider(&provider).unwrap();
        assert!(svc
            .list_providers()
            .unwrap()
            .iter()
            .any(|p| p.id == "my-llm"));
        // 重复添加被拒
        assert!(matches!(
            svc.add_custom_provider(&provider),
            Err(AppError::ValidationError(_))
        ));
        svc.delete_provider("my-llm").unwrap();
        assert!(!svc
            .list_providers()
            .unwrap()
            .iter()
            .any(|p| p.id == "my-llm"));
        // 内置服务商不可删
        assert!(matches!(
            svc.delete_provider("openai"),
            Err(AppError::ValidationError(_))
        ));
    }

    #[test]
    fn test_disk_persistence_across_instances() {
        let mut svc = VaultService::new(Box::new(disk_storage("persist")));
        let _ = svc.storage.delete();
        svc.initialize("password123").unwrap();
        svc.create_api_key(&CreateApiKeyRequest {
            name: "落盘".into(),
            provider_id: "qwen".into(),
            value: "sk-abcdef0123".into(),
            tags: None,
        })
        .unwrap();
        drop(svc);

        let mut svc2 = VaultService::new(Box::new(disk_storage("persist")));
        assert!(svc2.status().initialized);
        svc2.unlock("password123").unwrap();
        assert_eq!(svc2.list_api_keys().unwrap().len(), 1);
        svc2.storage.delete().unwrap();
    }
}
