use serde::{Deserialize, Serialize};

/// 探测进度事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectProgress {
    pub agent_id: String,
    pub agent_name: String,
    pub phase: String,
    pub done: usize,
    pub total: usize,
    pub finished: bool,
}

/// 安装任务阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallPhase {
    Queued,
    Resolving,
    Downloading,
    Verifying,
    Installing,
    Uninstalling,
    Finished,
    Failed,
    Cancelled,
}

impl InstallPhase {
    pub fn label(&self) -> &'static str {
        match self {
            InstallPhase::Queued => "排队中",
            InstallPhase::Resolving => "解析源",
            InstallPhase::Downloading => "下载中",
            InstallPhase::Verifying => "校验中",
            InstallPhase::Installing => "安装中",
            InstallPhase::Uninstalling => "卸载中",
            InstallPhase::Finished => "已完成",
            InstallPhase::Failed => "失败",
            InstallPhase::Cancelled => "已取消",
        }
    }
}

/// 安装日志行。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallLogLine {
    pub task_id: String,
    pub agent_id: String,
    pub stream: String,
    pub text: String,
    pub ts: i64,
}

/// 安装任务状态变化。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallStatus {
    pub task_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub action: String,
    pub phase: InstallPhase,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub ts: i64,
}

pub mod events {
    pub const DETECT_PROGRESS: &str = "detect://progress";
    pub const INSTALL_LOG: &str = "install://log";
    pub const INSTALL_STATUS: &str = "install://status";
    pub const WORKBUDDY_LOG: &str = "workbuddy://log";
    pub const WORKBUDDY_STATUS: &str = "workbuddy://status";
}

/// WorkBuddy 派发任务的日志行。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyLogLine {
    pub task_id: String,
    /// `text` / `tool` / `error`。
    pub stream: String,
    pub text: String,
    pub ts: i64,
}

/// WorkBuddy 派发任务的状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyTaskStatus {
    pub task_id: String,
    /// `running` / `finished` / `failed` / `cancelled`。
    pub phase: String,
    pub message: Option<String>,
    pub duration_ms: Option<i64>,
    pub ts: i64,
}
