//! 各智能体的登录态探测。
//!
//! 每个智能体属于**不同的账号体系**，凭据存储位置与格式互不相同：
//! - 腾讯 CodeBuddy 系（WorkBuddy / CodeBuddy / CodeBuddy CLI）：共用同一份会话文件
//!   `%LOCALAPPDATA%\CodeBuddyExtension\Data\Public\auth\<id>.info`；
//! - OpenAI（Codex CLI）：`~/.codex/auth.json`；
//! - Anthropic（Claude Code）：`~/.claude/.credentials.json`；
//! - Google（Gemini CLI）：`~/.gemini/oauth_creds.json`；
//! - 阿里云百炼（Qwen Code）：`~/.qwen/oauth_creds.json`；
//! - OpenCode：`~/.local/share/opencode/auth.json`。
//!
//! 所有取值一律脱敏；令牌只报存在性、长度与有效期，不导出明文。

use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;

use crate::agent::model::AgentDefinition;
use crate::commands::workbuddy::workbuddy_account;
use crate::commands::workbuddy_data::read_auth_session;
use crate::error::AppResult;

use super::app_state;

/// 凭据来源文件。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSource {
    /// 来源名称（如「认证会话」）。
    pub label: String,
    /// 文件或目录路径。
    pub path: String,
    /// 是否存在。
    pub exists: bool,
    /// 文件大小（字节）。
    pub size: u64,
    /// 目录内的条目数（目录来源用）。
    pub entries: usize,
    /// 类型：认证会话 / OAuth 凭据 / 配置 / 索引。
    pub kind: String,
}

/// 脱敏后的凭据字段。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CredentialField {
    /// 字段名（点分路径）。
    pub label: String,
    /// 展示值（敏感字段已脱敏）。
    pub value: String,
    /// 是否为敏感字段。
    pub secret: bool,
}

/// 智能体登录态报告。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentCredentialReport {
    pub agent_id: String,
    pub agent_name: String,
    pub vendor: String,
    /// 所属账号体系。
    pub account_system: String,
    /// 与哪些智能体共用同一份登录态。
    pub shared_with: Vec<String>,
    /// 是否检出有效登录态。
    pub logged_in: bool,
    /// 登录方式说明。
    pub method: String,
    /// 建议的登录命令。
    pub login_command: Option<String>,
    /// 凭据来源。
    pub sources: Vec<CredentialSource>,
    /// 脱敏后的关键字段。
    pub fields: Vec<CredentialField>,
    /// 补充说明。
    pub note: Option<String>,
}

/// 用户主目录。
fn home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// 展开 `~` 与正斜杠。
fn expand(path: &str) -> Option<PathBuf> {
    let normalized = path.replace('/', "\\");
    if normalized == "~" {
        return home();
    }
    if let Some(rest) = normalized.strip_prefix("~\\") {
        return home().map(|h| h.join(rest));
    }
    Some(PathBuf::from(normalized))
}

/// 脱敏：只保留前 4 位与后 4 位。
fn mask_secret(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return format!("••••（{} 字符）", chars.len());
    }
    format!(
        "{}…{}（{} 字符）",
        chars[..4].iter().collect::<String>(),
        chars[chars.len() - 4..].iter().collect::<String>(),
        chars.len()
    )
}

/// 字段名是否敏感。
fn is_secret_key(key: &str) -> bool {
    let k = key.to_lowercase();
    ["token", "key", "secret", "password", "credential", "jwt", "cookie"]
        .iter()
        .any(|s| k.contains(s))
}

/// 字段名是否值得展示。
fn is_interesting_key(key: &str) -> bool {
    let k = key.to_lowercase();
    [
        "token", "key", "secret", "email", "account", "uid", "uin", "id", "expire", "refresh", "scope",
        "subscription", "plan", "edition", "type", "nickname", "user",
    ]
    .iter()
    .any(|s| k.contains(s))
}

/// 递归收集 JSON 中的关键字段（值一律脱敏）。
fn collect_fields(node: &Value, prefix: &str, out: &mut Vec<CredentialField>) {
    match node {
        Value::Object(map) => {
            for (key, value) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_fields(value, &path, out);
            }
        }
        Value::Array(items) => {
            for (idx, item) in items.iter().enumerate() {
                collect_fields(item, &format!("{prefix}[{idx}]"), out);
            }
        }
        Value::String(text) => {
            let leaf = prefix
                .rsplit('.')
                .next()
                .unwrap_or(prefix)
                .to_lowercase();
            if is_interesting_key(&leaf) {
                let secret = is_secret_key(&leaf);
                out.push(CredentialField {
                    label: prefix.to_string(),
                    value: if secret { mask_secret(text) } else { text.clone() },
                    secret,
                });
            }
        }
        Value::Number(num) => {
            let leaf = prefix
                .rsplit('.')
                .next()
                .unwrap_or(prefix)
                .to_lowercase();
            if is_interesting_key(&leaf) {
                out.push(CredentialField {
                    label: prefix.to_string(),
                    value: num.to_string(),
                    secret: is_secret_key(&leaf),
                });
            }
        }
        _ => {}
    }
}

/// 读取 JSON 文件。
fn read_json(path: &PathBuf) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 构造一个来源条目。
fn source(label: &str, path: &PathBuf, kind: &str) -> CredentialSource {
    let (exists, size, entries) = match fs::metadata(path) {
        Ok(meta) if meta.is_dir() => (
            true,
            0,
            fs::read_dir(path)
                .map(|d| d.flatten().count())
                .unwrap_or(0),
        ),
        Ok(meta) => (true, meta.len(), 0),
        Err(_) => (false, 0, 0),
    };
    CredentialSource {
        label: label.to_string(),
        path: path.display().to_string(),
        exists,
        size,
        entries,
        kind: kind.to_string(),
    }
}

/// 追加 JSON 文件里的关键字段。
fn push_json_fields(path: &PathBuf, out: &mut Vec<CredentialField>) {
    let Some(doc) = read_json(path) else {
        return;
    };
    collect_fields(&doc, "", out);
}

/// 是否是腾讯 CodeBuddy 系（共用同一份登录态）。
pub fn is_codebuddy_family(agent_id: &str) -> bool {
    matches!(agent_id, "workbuddy" | "codebuddy")
}

/// 腾讯 CodeBuddy 系：WorkBuddy 桌面端 / CodeBuddy IDE 扩展 / CodeBuddy CLI 共用。
fn codebuddy_report(agent: &AgentDefinition) -> AgentCredentialReport {
    let mut report = base_report(agent, "腾讯 CodeBuddy 账号体系", "本地会话文件 + Bearer 令牌");
    report.shared_with = vec![
        "WorkBuddy 桌面端".to_string(),
        "CodeBuddy IDE 扩展".to_string(),
        "CodeBuddy CLI".to_string(),
    ];

    // 认证会话文件（共享产品目录）
    if let Some(session) = read_auth_session() {
        if let Some(file) = &session.file {
            let path = PathBuf::from(file);
            report.sources.push(source("认证会话", &path, "认证会话"));
        }
        report.logged_in = session.logged_in;
        if let Some(account) = &session.account {
            if let Some(uid) = &account.uid {
                report.fields.push(CredentialField {
                    label: "uid".into(),
                    value: uid.clone(),
                    secret: false,
                });
            }
            if let Some(uin) = &account.uin {
                report.fields.push(CredentialField {
                    label: "uin".into(),
                    value: uin.clone(),
                    secret: false,
                });
            }
            if let Some(t) = &account.account_type {
                report.fields.push(CredentialField {
                    label: "账号类型".into(),
                    value: t.clone(),
                    secret: false,
                });
            }
        }
        if let Some(token_type) = &session.token_type {
            report.fields.push(CredentialField {
                label: "令牌类型".into(),
                value: token_type.clone(),
                secret: false,
            });
        }
        if let Some(domain) = &session.domain {
            report.fields.push(CredentialField {
                label: "域".into(),
                value: domain.clone(),
                secret: false,
            });
        }
        if let Some(token) = &session.access_token {
            report.fields.push(CredentialField {
                label: "accessToken".into(),
                value: format!("密文 {}B", token.envelope_len),
                secret: true,
            });
        }
        if let Some(token) = &session.refresh_token {
            report.fields.push(CredentialField {
                label: "refreshToken".into(),
                value: format!("密文 {}B", token.envelope_len),
                secret: true,
            });
        }
        if !session.encrypted_fields.is_empty() {
            report.fields.push(CredentialField {
                label: "字段级加密".into(),
                value: session.encrypted_fields.join("、"),
                secret: false,
            });
        }
    }

    // 账号快照
    if let Some(root) = home().map(|h| h.join(".workbuddy")) {
        let snapshot = root
            .join("storage")
            .join("skeleton")
            .join("account-snapshot.json");
        report
            .sources
            .push(source("账号快照", &snapshot, "配置"));
        let account = workbuddy_account();
        if account.logged_in {
            report.logged_in = true;
        }
        if let Some(nickname) = &account.nickname {
            report.fields.push(CredentialField {
                label: "昵称".into(),
                value: nickname.clone(),
                secret: false,
            });
        }
        if let Some(edition) = &account.edition_type {
            report.fields.push(CredentialField {
                label: "套餐".into(),
                value: edition.clone(),
                secret: false,
            });
        }
        if let Some(mode) = &account.permission_mode {
            report.fields.push(CredentialField {
                label: "权限模式".into(),
                value: mode.clone(),
                secret: false,
            });
        }
        let local_storage = root.join("local_storage");
        report
            .sources
            .push(source("登录态索引", &local_storage, "索引"));
    }

    report.note = Some(
        "WorkBuddy 桌面端、CodeBuddy IDE 扩展与 CodeBuddy CLI 共用同一份登录态".to_string(),
    );
    report
}

/// 通用 JSON 凭据文件型智能体。
fn json_credential_report(
    agent: &AgentDefinition,
    account_system: &str,
    method: &str,
    login_command: Option<&str>,
    files: &[(&str, &str, &str)],
) -> AgentCredentialReport {
    let mut report = base_report(agent, account_system, method);
    report.login_command = login_command.map(|s| s.to_string());
    let mut fields = Vec::new();
    for (label, raw_path, kind) in files {
        let Some(path) = expand(raw_path) else {
            continue;
        };
        let src = source(label, &path, kind);
        if src.exists && src.kind == "OAuth 凭据" {
            push_json_fields(&path, &mut fields);
        }
        report.sources.push(src);
    }
    // 有凭据文件且含 token 字段 → 视为已登录
    report.logged_in = fields.iter().any(|f| {
        f.secret && f.label.to_lowercase().contains("token")
    });
    report.fields = fields.into_iter().take(14).collect();
    report
}

/// OpenCode：本地凭据 + 配置文件。
fn opencode_report(agent: &AgentDefinition) -> AgentCredentialReport {
    let mut report = base_report(
        agent,
        "各模型厂商 API Key（OpenCode 不统一账号）",
        "本地凭据文件 / 配置文件",
    );
    report.login_command = Some("opencode auth login".to_string());
    let mut fields = Vec::new();
    for (label, raw, kind) in [
        ("本地凭据", "~/.local/share/opencode/auth.json", "凭据"),
        ("配置文件", "~/.config/opencode/opencode.json", "配置"),
    ] {
        let Some(path) = expand(raw) else { continue };
        let src = source(label, &path, kind);
        if src.exists && kind == "凭据" {
            push_json_fields(&path, &mut fields);
        }
        report.sources.push(src);
    }
    report.logged_in = !fields.is_empty();
    report.fields = fields.into_iter().take(14).collect();
    report
}

/// 火山方舟（豆包）：靠环境变量，不落盘。
fn ark_report(agent: &AgentDefinition) -> AgentCredentialReport {
    let mut report = base_report(agent, "火山方舟 API Key", "环境变量");
    report.login_command = Some("设置环境变量 ARK_API_KEY 后启动".to_string());
    report.note = Some("豆包 CLI 不保存独立登录态，凭据由火山方舟 API Key 提供".to_string());
    report
}

/// 应用内登录型（TRAE / Cursor 等）。
fn app_login_report(
    agent: &AgentDefinition,
    account_system: &str,
    dir: &str,
    note: &str,
) -> AgentCredentialReport {
    let mut report = base_report(agent, account_system, "应用内登录（凭据由应用自行管理）");
    if let Some(path) = expand(dir) {
        report.sources.push(source("数据目录", &path, "配置"));
    }
    report.note = Some(note.to_string());
    report
}

/// 兜底。
fn generic_report(agent: &AgentDefinition) -> AgentCredentialReport {
    let mut report = base_report(agent, "未登记的账号体系", "未知");
    report.note = Some("该智能体未登记登录态来源，可在设置中补充".to_string());
    if let Some(provider) = &agent.credential_provider {
        report.fields.push(CredentialField {
            label: "凭据服务商标识".into(),
            value: provider.clone(),
            secret: false,
        });
    }
    report
}

/// 构造基础报告。
fn base_report(agent: &AgentDefinition, account_system: &str, method: &str) -> AgentCredentialReport {
    AgentCredentialReport {
        agent_id: agent.id.clone(),
        agent_name: agent.name.clone(),
        vendor: agent.vendor.clone(),
        account_system: account_system.to_string(),
        shared_with: Vec::new(),
        logged_in: false,
        method: method.to_string(),
        login_command: None,
        sources: Vec::new(),
        fields: Vec::new(),
        note: None,
    }
}

/// 探测指定智能体的登录态。
#[tauri::command]
pub fn agent_credentials(app: AppHandle, agent_id: String) -> AppResult<AgentCredentialReport> {
    let state = app_state(&app)?;
    let cfg = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    let defs = crate::agent::definitions(&cfg);
    let Some(agent) = defs.iter().find(|a| a.id == agent_id) else {
        return Ok(AgentCredentialReport {
            agent_id,
            note: Some("未找到该智能体".to_string()),
            ..Default::default()
        });
    };

    let report = match agent.id.as_str() {
        "workbuddy" | "codebuddy" => codebuddy_report(agent),
        "codex" => json_credential_report(
            agent,
            "OpenAI 账号",
            "OAuth 凭据文件",
            Some("codex login"),
            &[("认证凭据", "~/.codex/auth.json", "OAuth 凭据")],
        ),
        "claude-code" => json_credential_report(
            agent,
            "Anthropic 账号",
            "OAuth 凭据文件",
            Some("claude login"),
            &[
                ("认证凭据", "~/.claude/.credentials.json", "OAuth 凭据"),
                ("配置文件", "~/.claude.json", "配置"),
            ],
        ),
        "gemini-cli" => json_credential_report(
            agent,
            "Google 账号",
            "OAuth 凭据文件",
            Some("gemini auth"),
            &[("认证凭据", "~/.gemini/oauth_creds.json", "OAuth 凭据")],
        ),
        "qwen-code" => json_credential_report(
            agent,
            "阿里云百炼（Qwen Code）",
            "OAuth 凭据文件",
            Some("qwen"),
            &[("认证凭据", "~/.qwen/oauth_creds.json", "OAuth 凭据")],
        ),
        "opencode" | "opencode-desktop" => opencode_report(agent),
        "doubao" => ark_report(agent),
        "trae" => app_login_report(
            agent,
            "TRAE 账号",
            "~/.trae",
            "TRAE 凭据由应用自行管理，未在本地暴露可读格式",
        ),
        "cursor" => app_login_report(
            agent,
            "Cursor 账号",
            "~/.cursor",
            "Cursor 凭据由应用自行管理，未在本地暴露可读格式",
        ),
        _ => generic_report(agent),
    };
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 打印各智能体的登录态报告（人工核对用）。
    #[test]
    fn dump_agent_credentials() {
        for id in [
            "workbuddy",
            "codebuddy",
            "codex",
            "claude-code",
            "gemini-cli",
            "qwen-code",
            "opencode",
            "doubao",
            "trae",
            "cursor",
            "unknown-agent",
        ] {
            let agent = AgentDefinition {
                id: id.to_string(),
                name: id.to_string(),
                vendor: "test".into(),
                kind: crate::agent::model::AgentKind::Cli,
                description: String::new(),
                tags: vec![],
                cli_names: vec![],
                version_args: vec![],
                npm_package: None,
                official_download: None,
                official_page: None,
                open_platform: None,
                silent_args: vec![],
                mcp_config_path: None,
                mcp_config_format: None,
                config_path: None,
                credential_provider: None,
                icon_letter: "T".into(),
                accent: "#000".into(),
                install_method: crate::agent::model::InstallMethod::Manual,
                builtin: true,
            };
            let report = match id {
                "workbuddy" | "codebuddy" => codebuddy_report(&agent),
                "codex" => json_credential_report(
                    &agent,
                    "OpenAI 账号",
                    "OAuth 凭据文件",
                    Some("codex login"),
                    &[("认证凭据", "~/.codex/auth.json", "OAuth 凭据")],
                ),
                "claude-code" => json_credential_report(
                    &agent,
                    "Anthropic 账号",
                    "OAuth 凭据文件",
                    Some("claude login"),
                    &[("认证凭据", "~/.claude/.credentials.json", "OAuth 凭据")],
                ),
                "gemini-cli" => json_credential_report(
                    &agent,
                    "Google 账号",
                    "OAuth 凭据文件",
                    Some("gemini auth"),
                    &[("认证凭据", "~/.gemini/oauth_creds.json", "OAuth 凭据")],
                ),
                "qwen-code" => json_credential_report(
                    &agent,
                    "阿里云百炼（Qwen Code）",
                    "OAuth 凭据文件",
                    Some("qwen"),
                    &[("认证凭据", "~/.qwen/oauth_creds.json", "OAuth 凭据")],
                ),
                "opencode" | "opencode-desktop" => opencode_report(&agent),
                "doubao" => ark_report(&agent),
                "trae" => app_login_report(&agent, "TRAE 账号", "~/.trae", "应用内"),
                "cursor" => app_login_report(&agent, "Cursor 账号", "~/.cursor", "应用内"),
                _ => generic_report(&agent),
            };
            println!("--- {id} ---");
            println!("  system={} logged_in={}", report.account_system, report.logged_in);
            println!("  method={} cmd={:?}", report.method, report.login_command);
            println!("  shared={:?}", report.shared_with);
            for s in &report.sources {
                println!(
                    "  source[{}] kind={} exists={} size={} entries={} path={}",
                    s.label, s.kind, s.exists, s.size, s.entries, s.path
                );
            }
            for f in &report.fields {
                println!("  field[{}] = {} (secret={})", f.label, f.value, f.secret);
            }
            if let Some(n) = &report.note {
                println!("  note={n}");
            }
        }
    }
}
