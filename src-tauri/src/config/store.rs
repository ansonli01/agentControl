use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::agent::model::AgentOverride;
use crate::error::{AppError, AppResult};
use crate::state::data_dir;

/// 项目条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    /// 工作区根目录
    pub workspace: String,
    /// 绑定的智能体 id
    pub agents: Vec<String>,
    pub default_model: Option<String>,
    /// 附加启动参数
    pub extra_args: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
}
/// 凭据条目（密钥本体存于系统凭据库，此处仅保留引用与掩码）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub id: String,
    pub provider: String,
    pub label: String,
    /// 掩码后的密钥，例如 sk-****abcd
    pub masked: String,
    pub created_at: i64,
}

/// 全局配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// npm 镜像源
    pub npm_registry: String,
    /// HTTP 代理
    pub proxy: Option<String>,
    /// 下载目录
    pub download_dir: Option<String>,
    /// 单目标探测超时（秒）
    pub detect_timeout_secs: u64,
    /// 安装任务超时（分钟）
    pub install_timeout_minutes: u64,
    pub projects: Vec<Project>,
    pub agent_overrides: Vec<AgentOverride>,
    pub credentials: Vec<Credential>,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            npm_registry: "https://registry.npmmirror.com".into(),
            proxy: None,
            download_dir: None,
            detect_timeout_secs: 8,
            install_timeout_minutes: 30,
            projects: Vec::new(),
            agent_overrides: Vec::new(),
            credentials: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn find_project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }
}
/// 配置文件路径：用户配置目录下的 AgentControl/config.json
pub fn config_path() -> AppResult<PathBuf> {
    Ok(data_dir()?.join("config.json"))
}

/// 读取配置（不存在时返回默认配置）。
pub fn load() -> AppResult<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    match serde_json::from_str::<AppConfig>(&raw) {
        Ok(cfg) => Ok(cfg),
        Err(e) => {
            tracing::warn!("config parse failed, fallback to default: {e}");
            Ok(AppConfig::default())
        }
    }
}
/// 原子写入配置（先写临时文件再替换）。
pub fn save(cfg: &AppConfig) -> AppResult<()> {
    let path = config_path()?;
    write_atomic(&path, serde_json::to_string_pretty(cfg)?.as_bytes())
}

/// 原子写入任意内容。
pub fn write_atomic(path: &Path, content: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let stamp = chrono::Utc::now().timestamp_millis();
    let tmp = path.with_extension(format!("tmp{stamp}"));
    std::fs::write(&tmp, content)?;
    match std::fs::rename(&tmp, path) {
        Ok(_) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(AppError::msg(e.to_string()))
        }
    }
}