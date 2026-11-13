use serde::{Deserialize, Serialize};

/// 智能体形态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentKind {
    /// 命令行 CLI
    Cli,
    /// 桌面客户端 / IDE 插件
    Desktop,
}

/// 安装方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallMethod {
    /// npm 全局包（走国内镜像）
    Npm,
    /// 官方安装包下载
    Official,
    /// 仅检测，不提供自动安装
    Manual,
}

/// MCP 配置格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigFormat {
    Json,
    Toml,
}

/// 智能体定义（清单条目）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefinition {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub kind: AgentKind,
    pub description: String,
    /// 标签，用于前端筛选
    pub tags: Vec<String>,
    /// 命令行可执行文件名（探测用）
    pub cli_names: Vec<String>,
    /// 版本探测参数
    pub version_args: Vec<String>,
    /// npm 包名（InstallMethod::Npm）
    pub npm_package: Option<String>,
    /// 官方安装包直链（可被用户覆盖）
    pub official_download: Option<String>,
    /// 官方下载页 / 官网（用户可点开手动下载）
    pub official_page: Option<String>,
    /// 开放平台 / API 接入地址
    pub open_platform: Option<String>,
    /// 静默安装参数
    pub silent_args: Vec<String>,
    /// MCP 配置路径（支持 ~ 与 %VAR% 展开）
    pub mcp_config_path: Option<String>,
    pub mcp_config_format: Option<ConfigFormat>,
    /// 主配置路径
    pub config_path: Option<String>,
    /// 需要的 API Key 类型
    pub credential_provider: Option<String>,
    /// 图标字母（前端绘制标识）
    pub icon_letter: String,
    /// 主题色
    pub accent: String,
    pub install_method: InstallMethod,
    /// 是否内置条目（自定义条目为 false）
    pub builtin: bool,
}

/// 用户自定义覆盖（下载源、镜像、参数等）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentOverride {
    pub agent_id: String,
    pub official_download: Option<String>,
    pub official_page: Option<String>,
    pub npm_package: Option<String>,
    pub npm_registry: Option<String>,
    pub silent_args: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

/// 探测来源。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectSource {
    /// path / npm / registry / install-dir / shortcut
    pub kind: String,
    pub detail: Option<String>,
}

/// 探测结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub agent_id: String,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub sources: Vec<DetectSource>,
    pub note: Option<String>,
    pub checked_at: i64,
    /// 已安装但版本未知（CLI 存在但 --version 失败）
    pub version_unknown: bool,
}

/// 凭据服务商标识。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub key_label: String,
    pub base_url: Option<String>,
    pub docs: Option<String>,
}
