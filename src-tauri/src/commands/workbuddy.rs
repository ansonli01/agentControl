//! WorkBuddy 集成：读取本机 WorkBuddy 的运行状态与能力清单。
//!
//! WorkBuddy 桌面端把数据放在 `~/.workbuddy` 下：
//! - `last-launch.json`：版本与最近启动信息
//! - `workspace/sessions/<id>`：会话目录
//! - `mcp.json`：MCP 服务配置
//! - `.skill-list-cache.json`：已安装技能（SkillHub 市场）
//! - `connectors/`：已接入的连接器
//! - `models.json`：自定义模型配置（仅读取元信息，不回传密钥）

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;

use crate::error::AppResult;

/// WorkBuddy 运行状态。
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyStatus {
    /// 是否检测到 WorkBuddy 数据目录。
    pub installed: bool,
    /// 客户端是否正在运行。
    pub running: bool,
    /// 客户端版本。
    pub version: Option<String>,
    /// 安装目录。
    pub install_dir: Option<String>,
    /// 数据目录（`~/.workbuddy`）。
    pub data_dir: Option<String>,
    /// 最近启动时间（ISO8601）。
    pub last_launch: Option<String>,
    /// 会话数量。
    pub session_count: usize,
    /// 技能数量。
    pub skill_count: usize,
    /// MCP 服务数量。
    pub mcp_count: usize,
    /// 连接器数量。
    pub connector_count: usize,
    /// 模型数量。
    pub model_count: usize,
}

/// 会话条目。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddySession {
    pub id: String,
    /// 最后修改时间（毫秒时间戳）。
    pub updated_at: i64,
}

/// MCP 服务条目。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyMcp {
    pub name: String,
    /// `http` / `sse` / `stdio` 等。
    pub kind: Option<String>,
    pub url: Option<String>,
    pub command: Option<String>,
    pub disabled: bool,
}

/// 技能条目。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddySkill {
    pub name: String,
    pub description: Option<String>,
    /// `builtin` / `userSettings` 等来源。
    pub source: Option<String>,
    pub canonical_name: Option<String>,
    pub version: Option<String>,
    /// 安装时间（毫秒时间戳）。
    pub installed_at: Option<i64>,
}

/// 模型条目（不含任何密钥字段）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyModel {
    pub id: String,
    pub name: Option<String>,
    pub vendor: Option<String>,
    pub url: Option<String>,
}

/// WorkBuddy 能力总览。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyOverview {
    pub status: WorkBuddyStatus,
    pub sessions: Vec<WorkBuddySession>,
    pub mcps: Vec<WorkBuddyMcp>,
    pub skills: Vec<WorkBuddySkill>,
    pub connectors: Vec<String>,
    pub models: Vec<WorkBuddyModel>,
}

/// 用户主目录。
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// WorkBuddy 数据目录。
fn data_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".workbuddy"))
}

/// 读取 JSON 文件。
fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 客户端是否在运行。
fn is_running() -> bool {
    let out = match Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq WorkBuddy.exe", "/NH"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };
    String::from_utf8_lossy(&out.stdout).contains("WorkBuddy.exe")
}

/// 运行中进程的可执行文件所在目录。
fn running_install_dir() -> Option<String> {
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Process WorkBuddy -ErrorAction SilentlyContinue | Select-Object -First 1).Path",
        ])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        return None;
    }
    Path::new(&path)
        .parent()
        .map(|p| p.display().to_string())
}

/// 目录最后修改时间（毫秒）。
fn modified_millis(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 统计会话总数（不截断）。
fn count_sessions(root: &Path) -> usize {
    let dir = root.join("workspace").join("sessions");
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .count()
}

/// 统计会话。
fn collect_sessions(root: &Path) -> Vec<WorkBuddySession> {
    let mut out = Vec::new();
    let dir = root.join("workspace").join("sessions");
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if id.starts_with('.') {
            continue;
        }
        out.push(WorkBuddySession {
            updated_at: modified_millis(&entry.path()),
            id,
        });
    }
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    out.truncate(50);
    out
}

/// 解析 MCP 服务。
fn collect_mcps(root: &Path) -> Vec<WorkBuddyMcp> {
    let mut out = Vec::new();
    let doc = match read_json(&root.join("mcp.json")) {
        Some(d) => d,
        None => return out,
    };
    let servers = match doc.get("mcpServers").and_then(|v| v.as_object()) {
        Some(s) => s,
        None => return out,
    };
    for (name, cfg) in servers {
        out.push(WorkBuddyMcp {
            name: name.clone(),
            kind: cfg
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            url: cfg.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            command: cfg
                .get("command")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            disabled: cfg
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        });
    }
    out
}

/// 解析已安装技能。
fn collect_skills(root: &Path) -> Vec<WorkBuddySkill> {
    let mut out = Vec::new();
    let doc = match read_json(&root.join(".skill-list-cache.json")) {
        Some(d) => d,
        None => return out,
    };
    let results = match doc.get("results").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => return out,
    };
    for item in results {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let description = item
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| {
                let one_line: String = s
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if one_line.chars().count() > 120 {
                    let cut: String = one_line.chars().take(120).collect();
                    format!("{cut}…")
                } else {
                    one_line
                }
            })
            .filter(|s| !s.is_empty());
        out.push(WorkBuddySkill {
            name,
            description,
            source: item
                .get("source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            canonical_name: item
                .get("canonicalName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            version: item
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            installed_at: item.get("installedAt").and_then(|v| v.as_i64()),
        });
    }
    out.truncate(100);
    out
}

/// 列出连接器目录。
fn collect_connectors(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let dir = root.join("connectors");
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "marketplace-meta" {
            continue;
        }
        out.push(name);
    }
    out.sort();
    out
}

/// 解析自定义模型（仅元信息）。
fn collect_models(root: &Path) -> Vec<WorkBuddyModel> {
    let mut out = Vec::new();
    let doc = match read_json(&root.join("models.json")) {
        Some(d) => d,
        None => return out,
    };
    let items = match doc.as_array() {
        Some(a) => a,
        None => return out,
    };
    for item in items {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            continue;
        }
        out.push(WorkBuddyModel {
            name: item.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            vendor: item
                .get("vendor")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            url: item.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            id,
        });
    }
    out
}

/// 汇总状态。
fn build_status(root: &Path) -> WorkBuddyStatus {
    let launch = read_json(&root.join("last-launch.json"));
    let mcps = collect_mcps(root);
    let skills = collect_skills(root);
    let connectors = collect_connectors(root);
    let models = collect_models(root);
    WorkBuddyStatus {
        installed: root.exists(),
        running: is_running(),
        version: launch
            .as_ref()
            .and_then(|d| d.get("version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        install_dir: running_install_dir(),
        data_dir: Some(root.display().to_string()),
        last_launch: launch
            .as_ref()
            .and_then(|d| d.get("timestamp"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        session_count: count_sessions(root),
        skill_count: skills.len(),
        mcp_count: mcps.len(),
        connector_count: connectors.len(),
        model_count: models.len(),
    }
}

/// 查询 WorkBuddy 运行状态。
#[tauri::command]
pub fn workbuddy_status() -> WorkBuddyStatus {
    match data_dir() {
        Some(root) => build_status(&root),
        None => WorkBuddyStatus::default(),
    }
}

/// 查询 WorkBuddy 能力总览（会话 / MCP / 技能 / 连接器 / 模型）。
#[tauri::command]
pub fn workbuddy_overview() -> WorkBuddyOverview {
    let root = data_dir().unwrap_or_else(|| PathBuf::from("."));
    let status = build_status(&root);
    let sessions = collect_sessions(&root);
    let mcps = collect_mcps(&root);
    let skills = collect_skills(&root);
    let connectors = collect_connectors(&root);
    let models = collect_models(&root);
    WorkBuddyOverview {
        status,
        sessions,
        mcps,
        skills,
        connectors,
        models,
    }
}

/// WorkBuddy 登录账号信息。
///
/// 数据来源为数据目录下的账号快照 `storage/skeleton/account-snapshot.json`
/// 与用户偏好 `storage/user-<uid>-personal/**`。
/// 仅读取展示所需的非敏感字段，不涉及任何令牌或密钥。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyAccount {
    /// 是否检测到已登录账号。
    pub logged_in: bool,
    /// 用户 UID。
    pub uid: Option<String>,
    /// 昵称（已清洗掉私有区占位符）。
    pub nickname: Option<String>,
    /// 昵称原文（可能包含 Unicode 私有区占位符）。
    pub nickname_raw: Option<String>,
    /// 账号类型（`personal` / `team`）。
    pub account_type: Option<String>,
    /// 套餐类型（`free` / `pro` 等）。
    pub edition_type: Option<String>,
    /// 是否为付费版。
    pub is_pro: bool,
    /// 是否为管理员。
    pub is_admin: bool,
    /// OneID 账号 ID。
    pub oneid_account_id: Option<String>,
    /// 账号快照写入时间（毫秒时间戳）。
    pub saved_at: Option<i64>,
    /// 界面语言。
    pub language: Option<String>,
    /// 界面主题。
    pub theme: Option<String>,
    /// 本地任务权限模式。
    pub permission_mode: Option<String>,
    /// 数据目录。
    pub data_dir: Option<String>,
    /// 未读取到时的说明。
    pub reason: Option<String>,
}

/// 清洗昵称：去掉 Unicode 私有区占位符与零宽字符，压缩空白。
///
/// WorkBuddy 的昵称常带有 U+E000–U+F8FF 的私有区 emoji 占位符，
/// 直接渲染会显示成方块。
fn sanitize_nickname(raw: &str) -> String {
    let filtered: String = raw
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            let private = (0xE000..=0xF8FF).contains(&cp);
            let zero_width = cp == 0x200B
                || cp == 0x200C
                || cp == 0x200D
                || cp == 0xFEFF
                || (0x200E..=0x200F).contains(&cp);
            !private && !zero_width
        })
        .collect();
    filtered
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 从用户目录的 scoped 分区里找语言设置。
fn scoped_language(user_dir: &Path) -> Option<String> {
    let entries = fs::read_dir(user_dir.join("scoped")).ok()?;
    for entry in entries.flatten() {
        let path = entry.path().join("preferences.json");
        if let Some(doc) = read_json(&path) {
            if let Some(lang) = doc.get("language").and_then(|v| v.as_str()) {
                return Some(lang.to_string());
            }
        }
    }
    None
}

/// 读取 WorkBuddy 登录账号信息。
#[tauri::command]
pub fn workbuddy_account() -> WorkBuddyAccount {
    let Some(root) = data_dir() else {
        return WorkBuddyAccount {
            reason: Some("无法定位用户目录".to_string()),
            ..Default::default()
        };
    };
    let mut account = WorkBuddyAccount {
        data_dir: Some(root.display().to_string()),
        ..Default::default()
    };

    let Some(doc) = read_json(
        &root.join("storage")
            .join("skeleton")
            .join("account-snapshot.json"),
    ) else {
        account.reason = Some("未找到账号快照，WorkBuddy 可能尚未登录".to_string());
        return account;
    };
    let primary = doc.get("primary").cloned().unwrap_or_else(|| doc.clone());

    account.uid = primary
        .get("uid")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    account.nickname = primary
        .get("nickname")
        .and_then(|v| v.as_str())
        .map(sanitize_nickname)
        .filter(|s| !s.is_empty());
    account.nickname_raw = primary
        .get("nickname")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    account.account_type = primary
        .get("type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    account.edition_type = primary
        .get("editionType")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    account.is_pro = primary
        .get("isPro")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    account.is_admin = primary
        .get("isAdmin")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    account.oneid_account_id = primary
        .get("oneidAccountId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    account.saved_at = primary.get("savedAt").and_then(|v| v.as_i64());
    account.logged_in = account.uid.is_some() || account.nickname.is_some();

    // 用户偏好：主题 / 权限模式 / 语言。
    if let Some(uid) = account.uid.clone() {
        let user_dir = root.join("storage").join(format!("user-{uid}-personal"));
        if let Some(pref) = read_json(&user_dir.join("global").join("preferences.json")) {
            account.theme = pref
                .get("appearanceTheme")
                .or_else(|| pref.get("theme"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            account.permission_mode = pref
                .get("localTaskPermissionMode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
        account.language = scoped_language(&user_dir);
    }
    account
}

/// 打开 WorkBuddy 相关路径或链接。
#[tauri::command]
pub fn workbuddy_open(app: tauri::AppHandle, target: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;

    let target = target.trim().to_string();
    if target.is_empty() {
        return Err(crate::error::AppError::msg("目标为空"));
    }
    if target.starts_with("http://") || target.starts_with("https://") {
        app.opener()
            .open_url(target, None::<&str>)
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        return Ok(());
    }
    let path = PathBuf::from(&target);
    if path.is_dir() {
        // 目录交给资源管理器打开。
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        return Ok(());
    }
    app.opener()
        .open_path(target, None::<&str>)
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    Ok(())
}

/// WorkBuddy 关键路径集合（便于前端展示与打开）。
#[tauri::command]
pub fn workbuddy_paths() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(root) = data_dir() {
        map.insert("data".to_string(), root.display().to_string());
        map.insert(
            "sessions".to_string(),
            root.join("workspace").join("sessions").display().to_string(),
        );
        map.insert(
            "skills".to_string(),
            root.join("skills").display().to_string(),
        );
        map.insert(
            "connectors".to_string(),
            root.join("connectors").display().to_string(),
        );
    }
    map
}
