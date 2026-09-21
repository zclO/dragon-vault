use chrono::Utc;
use tauri::State;

use crate::error::AppError;
use crate::models::usage::{UsageReport, UsageSnapshotRecord};
use crate::services::usage_service;
use crate::AppState;

/// 查询单个 API Key 的额度/用量（手动触发，成功后记录快照落盘）
#[tauri::command]
pub async fn fetch_usage(
    state: State<'_, AppState>,
    key_id: String,
) -> Result<UsageReport, AppError> {
    // 同步段：取出密钥明文与服务商配置（不能跨 await 持有锁，且不更新使用时间）
    let (value, provider) = {
        let svc = crate::commands::vault_guard(&state)?;
        svc.key_for_test(&key_id)?
    };

    let fetched_at = Utc::now().to_rfc3339();
    let mut balance = None;
    let mut token_usage = None;
    let mut unsupported_reason = None;

    match usage_service::find_adapter(&provider.base_url)
        .zip(usage_service::host_of(&provider.base_url))
    {
        Some((kind, host)) => {
            let (b, t) = usage_service::fetch_usage(kind, &host, &value).await?;
            balance = Some(b);
            token_usage = t;
        }
        None => {
            unsupported_reason = Some(usage_service::unsupported_reason(
                &provider.id,
                &provider.name,
            ));
        }
    }

    // 有可量化结果时记录快照（不支持的厂商不落库，避免污染趋势）
    let total_balance = balance.as_ref().and_then(|b| b.total);
    let total_tokens = token_usage
        .as_ref()
        .map(|list| list.iter().filter_map(|u| u.total_tokens).sum());
    if total_balance.is_some() || total_tokens.is_some() {
        crate::commands::vault_guard(&state)?.record_usage_snapshot(UsageSnapshotRecord {
            key_id: key_id.clone(),
            ts: fetched_at.clone(),
            total_balance,
            total_tokens: total_tokens.filter(|v| *v > 0),
        })?;
    }

    Ok(UsageReport {
        key_id,
        provider_id: provider.id,
        fetched_at,
        balance,
        token_usage,
        unsupported_reason,
    })
}

/// 指定 Key 的历史用量快照（时间倒序）
#[tauri::command]
pub fn usage_history(
    state: State<'_, AppState>,
    key_id: String,
    limit: Option<usize>,
) -> Result<Vec<UsageSnapshotRecord>, AppError> {
    let limit = limit.unwrap_or(30).clamp(1, 200);
    crate::commands::vault_guard(&state)?.usage_history(&key_id, limit)
}
