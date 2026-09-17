# Dragon Vault — Rust 编码规范

## 1. 总体原则

- 遵循 [Rust 官方 API Guidelines](https://rust-lang.github.io/api-guidelines/) 和 [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
- 使用 Rust 2021 edition
- 代码必须通过 `cargo clippy` 零警告
- 代码必须通过 `cargo fmt --check` 格式化检查
- 所有公开 API 必须有文档注释 (`///`)

## 2. 项目结构规范

### 2.1 模块组织

```
src-tauri/src/
├── main.rs              # 仅调用 lib::run()，不含业务逻辑
├── lib.rs               # Tauri Builder 配置，注册所有 commands
├── error.rs             # 统一错误类型 AppError
├── commands/            # Tauri command 入口层
│   ├── mod.rs           # 统一导出所有 command
│   ├── key_commands.rs
│   ├── provider_commands.rs
│   └── app_commands.rs
├── models/              # 数据结构定义（纯数据，无业务逻辑）
│   ├── mod.rs
│   ├── api_key.rs
│   ├── provider.rs
│   └── settings.rs
├── services/            # 业务逻辑层
│   ├── mod.rs
│   ├── crypto.rs
│   ├── key_service.rs
│   └── storage.rs       # Storage trait 定义
└── storage/             # 存储层实现
    ├── mod.rs
    └── local_storage.rs
```

### 2.2 模块职责

| 模块 | 职责 | 禁止 |
|------|------|------|
| `commands/` | 参数校验、调用 service、返回结果 | 包含业务逻辑、直接操作文件 |
| `models/` | 数据结构定义、序列化/反序列化 | 包含业务逻辑、I/O 操作 |
| `services/` | 核心业务逻辑 | 直接处理 Tauri IPC |
| `storage/` | 数据持久化 | 包含业务判断逻辑 |

## 3. 命名规范

### 3.1 基本规则

| 类别 | 风格 | 示例 |
|------|------|------|
| 模块/文件 | `snake_case` | `key_commands.rs`, `crypto.rs` |
| 结构体/枚举/Trait | `PascalCase` | `ApiKey`, `ProviderConfig`, `Storage` |
| 函数/方法 | `snake_case` | `encrypt_key()`, `list_providers()` |
| 常量 | `SCREAMING_SNAKE_CASE` | `MAX_KEY_LENGTH`, `DEFAULT_ALGORITHM` |
| 类型参数 | 单个大写字母或描述性 `PascalCase` | `T`, `E`, `S: Storage` |

### 3.2 命名示例

```rust
// ✅ 正确
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub provider_id: String,
    pub encrypted_value: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

pub trait Storage {
    fn save(&self, data: &VaultData) -> Result<(), AppError>;
    fn load(&self) -> Result<VaultData, AppError>;
}

// ❌ 错误
pub struct apiKey { ... }          // 应为 PascalCase
pub fn EncryptKey() -> ...         // 应为 snake_case
const max_length: usize = 1024;    // 应为 SCREAMING_SNAKE_CASE
```

## 4. 错误处理规范

### 4.1 统一错误类型

使用 `thiserror` 定义统一的应用错误类型：

```rust
// error.rs
use thiserror::Error;

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

    #[error("内部错误: {0}")]
    Internal(String),
}

// 实现 Tauri 的 Serialize 以便传递给前端
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string())
    }
}
```

### 4.2 错误处理原则

- **command 层**：捕获 service 层错误，直接返回 `Result<T, AppError>`
- **service 层**：使用 `?` 运算符传播错误，在必要时包装为 `AppError`
- **禁止**：在 service 层使用 `.unwrap()` 或 `.expect()`（测试代码除外）
- **禁止**：吞掉错误（`let _ = ...`），除非有明确注释说明原因

```rust
// ✅ 正确
pub fn decrypt_key(&self, encrypted: &[u8]) -> Result<String, AppError> {
    let key = self.derive_key()?;
    let plaintext = self.cipher.decrypt(&key, encrypted)
        .map_err(|e| AppError::CryptoError(e.to_string()))?;
    String::from_utf8(plaintext)
        .map_err(|e| AppError::CryptoError(e.to_string()))
}

// ❌ 错误
pub fn decrypt_key(&self, encrypted: &[u8]) -> String {
    self.cipher.decrypt(...).unwrap()  // 禁止 unwrap
}
```

## 5. Tauri Command 规范

### 5.1 Command 签名

```rust
// ✅ 标准 command 签名
#[tauri::command]
pub async fn create_api_key(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    name: String,
    provider_id: String,
    value: String,
) -> Result<ApiKeySummary, AppError> {
    // 1. 参数校验
    // 2. 调用 service
    // 3. 返回结果
}
```

### 5.2 Command 命名

| 操作 | 前缀 | 示例 |
|------|------|------|
| 创建 | `create_` | `create_api_key` |
| 查询单个 | `get_` | `get_api_key` |
| 查询列表 | `list_` | `list_api_keys` |
| 更新 | `update_` | `update_api_key` |
| 删除 | `delete_` | `delete_api_key` |
| 操作 | 动词 | `test_key_connection`, `export_vault` |

### 5.3 注册 Command

所有 command 在 `lib.rs` 中统一注册：

```rust
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // commands 模块中的所有 command
            commands::create_api_key,
            commands::list_api_keys,
            commands::delete_api_key,
            // ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 6. 数据模型规范

### 6.1 序列化

所有需要跨 IPC 传输的数据结构必须派生 `Serialize` 和 `Deserialize`：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub encrypted_value: Vec<u8>,
    pub tags: Vec<String>,
    pub created_at: String,    // ISO 8601 格式
    pub updated_at: String,
}
```

### 6.2 模型层原则

- 模型是纯数据结构，不包含业务方法
- 使用 `#[serde(rename_all = "camelCase")]` 确保 JSON 字段名与前端一致
- 时间字段统一使用 ISO 8601 字符串格式传递

## 7. 安全编码规范

### 7.1 敏感数据处理

```rust
// 使用后立即清零敏感缓冲区
use zeroize::Zeroize;

let mut decrypted_key = decrypt(encrypted_value)?;
let result = use_key(&decrypted_key);
decrypted_key.zeroize();  // 立即清零
result
```

### 7.2 安全要求清单

- [ ] 加密算法使用 AES-256-GCM，禁止使用弱算法
- [ ] 密钥派生使用 Argon2id，迭代次数不低于推荐值
- [ ] 每次加密使用随机 Nonce/IV
- [ ] 敏感数据（明文密钥）在内存中停留时间最短化
- [ ] 日志中禁止输出 API Key 明文或完整值（可输出前4位 + `****`）
- [ ] 错误信息中禁止包含敏感数据

## 8. 依赖管理规范

### 8.1 推荐依赖

| 用途 | crate | 说明 |
|------|-------|------|
| 加密 | `aes-gcm` | AES-256-GCM 对称加密 |
| 密钥派生 | `argon2` | Argon2id 密码哈希 |
| 随机数 | `rand` | 安全随机数生成 |
| 错误处理 | `thiserror` | 派生 Error trait |
| UUID | `uuid` | 唯一标识符生成 |
| 时间 | `chrono` | 日期时间处理 |
| 敏感数据清零 | `zeroize` | 内存安全清零 |
| 序列化 | `serde` + `serde_json` | JSON 序列化 |

### 8.2 依赖原则

- 新增依赖前检查是否已有等效替代
- 优先选择维护活跃、社区广泛使用的 crate
- 安全相关 crate 必须检查其安全审计状态
- 在 `Cargo.toml` 中为每个依赖添加注释说明用途

## 9. 测试规范

### 9.1 测试组织

```rust
// 在对应模块文件底部添加测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // 测试加密解密往返一致性
    }

    #[test]
    fn test_invalid_password_returns_auth_error() {
        // 测试错误密码返回 AuthError
    }
}
```

### 9.2 测试要求

- 加密模块：必须有往返一致性测试、错误密码测试
- Service 层：必须有核心业务流程测试
- Command 层：使用集成测试验证 IPC 调用
- 测试中可使用固定的测试密钥，禁止使用真实 API Key

## 10. 日志规范

使用 `tracing` 或 `log` 宏记录日志：

```rust
use tracing::{info, warn, error, debug};

// ✅ 正确的日志级别
info!("用户成功解锁保险库");
warn!("API Key 即将过期: {}", key_id);
error!("加密操作失败: {}", err);
debug!("加载服务商配置: {}", provider_name);

// ❌ 禁止
println!("debug info");                    // 使用日志宏代替
info!("API Key: {}", full_key_value);      // 禁止记录完整密钥
```
