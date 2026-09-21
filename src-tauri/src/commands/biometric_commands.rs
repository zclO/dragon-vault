use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_biometry::{BiometryExt, DataOptions, GetDataOptions, SetDataOptions};
use zeroize::Zeroizing;

use crate::commands::vault_guard;
use crate::error::AppError;
use crate::AppState;

/// 生物封存记录坐标 — domain 同时用作 Android Keystore 密钥别名
const BIOMETRIC_DOMAIN: &str = "com.iss.dragon-vault";
const BIOMETRIC_NAME: &str = "master-key";
/// Argon2id 派生密钥长度（AES-256），解封后校验防错用
const KEY_LEN: usize = 32;

/// 当前平台是否启用生物解锁
///
/// 审计结论（tauri-plugin-biometry 0.2.8）：安卓侧 Keystore RSA 包裹 +
/// AES-256-GCM + 指纹变更失效的实现合格；Windows 侧为静态 IV 的
/// KeyCredential/CBC 方案、与官方 README 宣称的 WebAuthn PRF 不符，
/// 待其 0.3 正式发布并复审通过后再启用。
fn platform_enabled() -> bool {
    cfg!(target_os = "android")
}

fn ensure_platform() -> Result<(), AppError> {
    if platform_enabled() {
        Ok(())
    } else {
        Err(AppError::ValidationError(
            "当前版本仅 Android 支持生物解锁".into(),
        ))
    }
}

/// 指纹解锁可用性（供前端决定是否展示入口 / 开关）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiometricStatus {
    /// 设备生物硬件与系统录入是否可用
    pub available: bool,
    /// 本应用是否已封存主密钥（可走指纹解锁）
    pub enrolled: bool,
    /// 不可用原因（硬件缺失 / 未录入 / 平台未启用等）
    pub reason: Option<String>,
}

fn map_plugin_err(e: tauri_plugin_biometry::Error) -> AppError {
    AppError::Internal(format!("生物识别插件调用失败: {e}"))
}

/// 查询指纹解锁状态（含平台门控）
#[tauri::command]
pub async fn biometric_status(app: AppHandle) -> Result<BiometricStatus, AppError> {
    if !platform_enabled() {
        return Ok(BiometricStatus {
            available: false,
            enrolled: false,
            reason: Some("当前版本仅 Android 支持生物解锁".into()),
        });
    }
    let status = app.biometry().status().map_err(map_plugin_err)?;
    let enrolled = status.is_available
        && app
            .biometry()
            .has_data(DataOptions {
                domain: BIOMETRIC_DOMAIN.into(),
                name: BIOMETRIC_NAME.into(),
            })
            .map_err(map_plugin_err)?;
    Ok(BiometricStatus {
        available: status.is_available,
        enrolled,
        reason: status.error,
    })
}

/// 封存当前会话主密钥，启用指纹解锁（要求保险库已解锁）
#[tauri::command]
pub async fn enable_biometric(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    ensure_platform()?;
    let key = vault_guard(&state)?.session_key()?;
    app.biometry()
        .set_data(SetDataOptions {
            domain: BIOMETRIC_DOMAIN.into(),
            name: BIOMETRIC_NAME.into(),
            data: BASE64.encode(key.as_slice()),
        })
        .map_err(map_plugin_err)
}

/// 移除封存密钥，关闭指纹解锁
#[tauri::command]
pub async fn disable_biometric(app: AppHandle) -> Result<(), AppError> {
    ensure_platform()?;
    app.biometry()
        .remove_data(DataOptions {
            domain: BIOMETRIC_DOMAIN.into(),
            name: BIOMETRIC_NAME.into(),
        })
        .map_err(map_plugin_err)
}

/// 指纹解锁：系统生物验证通过后取回封存主密钥并解锁保险库
#[tauri::command]
pub async fn unlock_with_biometric(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    ensure_platform()?;
    let resp = app
        .biometry()
        .get_data(GetDataOptions {
            domain: BIOMETRIC_DOMAIN.into(),
            name: BIOMETRIC_NAME.into(),
            reason: "解锁 Dragon Vault".into(),
            cancel_title: Some("改用主密码".into()),
        })
        .map_err(map_plugin_err)?;
    let key = Zeroizing::new(
        BASE64
            .decode(resp.data.trim())
            .map_err(|_| AppError::CryptoError("封存密钥解码失败".into()))?,
    );
    if key.len() != KEY_LEN {
        return Err(AppError::CryptoError("封存密钥长度异常".into()));
    }
    vault_guard(&state)?.unlock_with_key(key)
}
