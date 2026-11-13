pub mod agent;
pub mod agent_credentials;
pub mod config;
pub mod detect;
pub mod install;
pub mod platform_catalogs;
pub mod tasks;
pub mod workbuddy;
pub mod workbuddy_cli;
pub mod workbuddy_catalog;
pub mod workbuddy_data;
pub mod workbuddy_dispatch;

use tauri::{AppHandle, Manager};

use crate::error::AppResult;
use crate::state::AppState;

/// 取全局状态。
pub fn app_state(app: &AppHandle) -> AppResult<tauri::State<'_, AppState>> {
    Ok(app.state::<AppState>())
}