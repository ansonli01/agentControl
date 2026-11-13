use tauri::AppHandle;

use crate::agent;
use crate::agent::model::{AgentDefinition, Provider};
use crate::error::AppResult;

use super::app_state;

/// 获取智能体清单。
#[tauri::command]
pub fn list_agents(app: AppHandle) -> AppResult<Vec<AgentDefinition>> {
    let state = app_state(&app)?;
    let cfg = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    Ok(agent::definitions(&cfg))
}

/// 获取凭据服务商标识。
#[tauri::command]
pub fn list_providers() -> Vec<Provider> {
    agent::registry::providers()
}