use tauri::AppHandle;

use crate::error::AppResult;
use crate::tasks::TaskRecord;

use super::app_state;

/// 查询历史任务。
#[tauri::command]
pub fn list_tasks(app: AppHandle, limit: Option<i64>, kind: Option<String>) -> AppResult<Vec<TaskRecord>> {
    let state = app_state(&app)?;
    let db_guard = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    let db = db_guard
        .as_ref()
        .ok_or_else(|| crate::error::AppError::msg("历史数据库尚未初始化"))?;
    db.list(limit.unwrap_or(200).clamp(1, 1000), kind)
}

/// 清空历史任务。
#[tauri::command]
pub fn clear_tasks(app: AppHandle) -> AppResult<()> {
    let state = app_state(&app)?;
    let db_guard = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    let db = db_guard
        .as_ref()
        .ok_or_else(|| crate::error::AppError::msg("历史数据库尚未初始化"))?;
    db.clear()
}