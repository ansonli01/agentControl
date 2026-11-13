use std::time::Duration;

use tauri::AppHandle;

use crate::agent::model::AgentStatus;
use crate::error::AppResult;

use super::app_state;

/// 全量探测本机已安装的智能体。
#[tauri::command]
pub async fn detect_all(app: AppHandle) -> AppResult<Vec<AgentStatus>> {
    let state = app_state(&app)?;
    let (cfg, timeout) = {
        let cfg = state
            .config
            .lock()
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        let t = Duration::from_secs(cfg.detect_timeout_secs.clamp(3, 60));
        (cfg.clone(), t)
    };
    let defs = crate::agent::definitions(&cfg);
    let statuses = crate::detect::detect_all(&defs, timeout, Some(app.clone())).await;

    {
        let mut cache = state
            .detect_cache
            .lock()
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        cache.clear();
        for s in &statuses {
            cache.insert(s.agent_id.clone(), s.clone());
        }
    }
    if let Ok(mut ts) = state.detect_cached_at.lock() {
        *ts = Some(chrono::Utc::now().timestamp_millis());
    }
    Ok(statuses)
}