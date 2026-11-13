use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::agent::model::AgentStatus;
use crate::config::store::AppConfig;
use crate::error::{AppError, AppResult};
use crate::tasks::TaskDb;

/// 全局应用状态。
pub struct AppState {
    /// 最近一次探测结果缓存（agent_id -> status）。
    pub detect_cache: Mutex<HashMap<String, AgentStatus>>,
    /// 探测缓存时间戳（毫秒）。
    pub detect_cached_at: Mutex<Option<i64>>,
    /// 配置缓存。
    pub config: Mutex<AppConfig>,
    /// 历史任务数据库。
    pub db: Mutex<Option<TaskDb>>,
    /// 安装任务运行句柄（task_id -> 取消标记）。
    pub install_handles: Mutex<HashMap<String, InstallHandle>>,
    /// WorkBuddy 派发任务句柄（task_id -> 取消标记）。
    pub workbuddy_handles: Mutex<HashMap<String, WorkBuddyHandle>>,
    /// 应用句柄（用于事件推送）。
    pub handle: Mutex<Option<AppHandle>>,
}

/// 安装任务的取消句柄。
#[derive(Clone)]
pub struct InstallHandle {
    pub agent_id: String,
    pub label: String,
    pub cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// WorkBuddy 派发任务的取消句柄。
#[derive(Clone)]
pub struct WorkBuddyHandle {
    /// 展示标签。
    pub label: String,
    /// 工作目录。
    pub cwd: String,
    /// 使用的模型。
    pub model: String,
    pub cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            detect_cache: Mutex::new(HashMap::new()),
            detect_cached_at: Mutex::new(None),
            config: Mutex::new(config),
            db: Mutex::new(None),
            install_handles: Mutex::new(HashMap::new()),
            workbuddy_handles: Mutex::new(HashMap::new()),
            handle: Mutex::new(None),
        }
    }

    pub fn set_handle(&self, handle: AppHandle) {
        if let Ok(mut h) = self.handle.lock() {
            *h = Some(handle);
        }
    }

    pub fn app_handle(&self) -> Option<AppHandle> {
        self.handle.lock().ok().and_then(|h| h.clone())
    }

    pub fn emit(&self, event: &str, payload: serde_json::Value) {
        if let Some(handle) = self.app_handle() {
            let _ = handle.emit(event, payload);
        }
    }

    /// 初始化数据库（延迟到 setup 阶段，避免构造期失败）。
    pub fn init_db(&self) -> AppResult<()> {
        let db = TaskDb::open()?;
        if let Ok(mut guard) = self.db.lock() {
            *guard = Some(db);
        }
        Ok(())
    }
}

/// 从 Tauri 状态中取出 `AppState`。
pub fn state(app: &tauri::App) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

/// 数据目录：`%APPDATA%\AgentControl`
pub fn data_dir() -> AppResult<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| AppError::msg("无法定位用户配置目录"))?
        .join("AgentControl");
    std::fs::create_dir_all(&base)?;
    Ok(base)
}
