//! WorkBuddy 会话与凭据数据读取。
//!
//! 会话数据来自桌面端数据库 `~/.workbuddy/workbuddy.db`（只读打开）：
//! - `sessions`：会话主表（标题、工作目录、模型、专家、状态、时间）
//! - `session_usage`：每个会话的 token 用量与额度消耗
//! - `workspaces`：最近打开的工作目录
//!
//! 凭据数据来自：
//! - `~/.workbuddy/local_storage/*.info`：WorkBuddy 自研本地存储（值以 gzip+base64 存储），
//!   其中保存了各域名的登录态索引（uid / 账号类型 / 域名 / 是否启用）。
//! - `~/.workbuddy/app/session/Network/Cookies`：Chromium Cookie 库（值用 Windows DPAPI 加密）。
//! - `~/.workbuddy/keyblob`：本地文件加密的主密钥容器（仅读取元信息，不导出密钥）。
//!
//! 安全约定：所有凭据只以**脱敏**形式返回（前缀 + 长度 + 有效期），
//! 完整令牌不会出现在 IPC 返回值或界面里。

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::Engine;
use flate2::read::GzDecoder;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::Value;

/// 会话条目。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddySessionRow {
    pub id: String,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub expert_id: Option<String>,
    pub source_mode: Option<String>,
    pub permission_mode: Option<String>,
    /// 是否为后台自动化会话。
    pub background: bool,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    /// 已消耗 token。
    pub used: Option<i64>,
    /// 上下文窗口大小。
    pub context_size: Option<i64>,
    /// 额度消耗。
    pub credit: f64,
}

/// 工作区条目。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyWorkspaceRow {
    pub path: String,
    pub last_opened_at: Option<i64>,
    /// 该工作区下的会话数。
    pub sessions: i64,
}

/// 会话数据总览。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddySessionOverview {
    /// 会话总数。
    pub total_sessions: i64,
    /// 未结束会话数。
    pub active_sessions: i64,
    /// 累计 token 消耗。
    pub total_used: i64,
    /// 累计额度消耗。
    pub total_credit: f64,
    /// 最近会话。
    pub sessions: Vec<WorkBuddySessionRow>,
    /// 最近工作区。
    pub workspaces: Vec<WorkBuddyWorkspaceRow>,
    /// 数据库路径。
    pub db_path: Option<String>,
    /// 读取失败时的说明。
    pub note: Option<String>,
}

/// 登录态条目（来自本地存储的登录态索引）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyLoginState {
    /// 用户 UID。
    pub uid: Option<String>,
    /// 账号类型（personal / team）。
    pub account_type: Option<String>,
    /// 域名。
    pub domain: Option<String>,
    /// 接口路径。
    pub api_path: Option<String>,
    /// 主机。
    pub host: Option<String>,
    /// 是否处于启用状态。
    pub enabled: bool,
    /// 记录时间（毫秒时间戳）。
    pub updated_at: Option<i64>,
}

/// Cookie 条目（值已脱敏）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyCookie {
    pub host: String,
    pub name: String,
    pub path: Option<String>,
    /// 是否仅 HTTPS。
    pub secure: bool,
    /// 是否禁止脚本读取。
    pub http_only: bool,
    /// 过期时间（毫秒时间戳）。
    pub expires_at: Option<i64>,
    /// 值长度。
    pub value_len: usize,
    /// 是否为加密存储。
    pub encrypted: bool,
    /// 脱敏预览。
    pub preview: Option<String>,
}

/// 疑似令牌键。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyTokenHint {
    /// 所属存储条目。
    pub entry: String,
    /// 键路径。
    pub path: String,
    /// 值长度。
    pub length: usize,
    /// 脱敏预览。
    pub preview: Option<String>,
}

/// 主密钥容器元信息（不含密钥内容）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyKeyblobInfo {
    pub version: i64,
    pub key_id: Option<String>,
    pub slots: usize,
    pub slot_type: Option<String>,
    pub protector_key_id: Option<String>,
    /// 文件更新时间（毫秒时间戳）。
    pub updated_at: Option<i64>,
}

/// 凭据数据总览。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyCredentialOverview {
    /// 是否存在启用中的登录态。
    pub logged_in: bool,
    /// 登录态条目。
    pub login_states: Vec<WorkBuddyLoginState>,
    /// Cookie 列表。
    pub cookies: Vec<WorkBuddyCookie>,
    /// Cookie 读取失败时的说明。
    pub cookies_error: Option<String>,
    /// 主密钥容器元信息。
    pub keyblob: Option<WorkBuddyKeyblobInfo>,
    /// 疑似令牌键。
    pub token_hints: Vec<WorkBuddyTokenHint>,
    /// CLI（`~/.codebuddy`）目录。
    pub cli_dir: Option<String>,
    /// CLI 是否存在登录凭据。
    pub cli_logged_in: bool,
    /// 认证会话（WorkBuddy 共享产品目录下的 `auth/<id>.info`）。
    pub auth_session: Option<WorkBuddyAuthSession>,
    /// 备注。
    pub note: Option<String>,
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

/// 脱敏预览：保留前 8 位与后 4 位。
fn mask(value: &str) -> Option<String> {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() < 12 {
        return None;
    }
    let head: String = chars.iter().take(8).collect();
    let tail: String = chars.iter().skip(chars.len() - 4).collect();
    Some(format!("{head}…{tail}"))
}

/// 解码本地存储条目：文件内容可能是 JSON 字符串包裹的 base64(gzip(json))。
fn decode_entry(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    let trimmed = text.trim();
    let inner = if trimmed.starts_with('"') {
        serde_json::from_str::<String>(trimmed).ok()?
    } else {
        trimmed.to_string()
    };
    let inner = inner.trim().to_string();
    let bytes = if inner.starts_with("H4sI") {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(inner.as_bytes())
            .ok()?;
        let mut out = Vec::new();
        GzDecoder::new(&raw[..]).read_to_end(&mut out).ok()?;
        out
    } else {
        inner.into_bytes()
    };
    serde_json::from_slice(&bytes).ok()
}

/// 遍历 JSON，找出所有字符串叶子。
fn collect_strings(node: &Value, prefix: &str, out: &mut Vec<(String, String)>) {
    match node {
        Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                collect_strings(v, &path, out);
            }
        }
        Value::Array(items) => {
            for (idx, v) in items.iter().enumerate() {
                collect_strings(v, &format!("{prefix}.{idx}"), out);
            }
        }
        Value::String(s) => out.push((prefix.to_string(), s.clone())),
        _ => {}
    }
}

/// 解析登录态索引的 key 串：`uid|type||||<domain>|<apiPath>||<host>`。
fn parse_login_key(raw: &str) -> Option<WorkBuddyLoginState> {
    let parts: Vec<&str> = raw.split('|').collect();
    if parts.len() < 6 || parts[0].is_empty() {
        return None;
    }
    Some(WorkBuddyLoginState {
        uid: Some(parts[0].to_string()),
        account_type: parts.get(1).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        domain: parts.get(5).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        api_path: parts.get(6).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        host: parts.get(8).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        enabled: false,
        updated_at: None,
    })
}

/// 读取本地存储中的登录态与疑似令牌键。
fn scan_local_storage(root: &Path) -> (Vec<WorkBuddyLoginState>, Vec<WorkBuddyTokenHint>) {
    let mut states = Vec::new();
    let mut hints = Vec::new();
    let dir = root.join("local_storage");
    let Ok(entries) = fs::read_dir(&dir) else {
        return (states, hints);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("info") {
            continue;
        }
        let Some(doc) = decode_entry(&path) else {
            continue;
        };
        let file = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let mut leaves = Vec::new();
        collect_strings(&doc, "", &mut leaves);
        for (path_in_doc, value) in &leaves {
            // 登录态索引：含分隔符的复合键
            if path_in_doc.ends_with(".key") && value.contains('|') {
                if let Some(mut state) = parse_login_key(value) {
                    // 找到同一条目里的 enabled / savedAt
                    if let Value::Array(items) = &doc {
                        for item in items {
                            let Some(obj) = item.as_object() else { continue };
                            let k = obj.get("key").and_then(|v| v.as_str()).unwrap_or("");
                            if k != value {
                                continue;
                            }
                            state.enabled = obj
                                .get("enabled")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            state.updated_at = obj.get("savedAt").and_then(|v| v.as_i64());
                        }
                    }
                    states.push(state);
                }
            }
            // 疑似令牌
            let lower = path_in_doc.to_ascii_lowercase();
            let looks_token = (lower.contains("token") || lower.contains("secret")
                || lower.contains("credential"))
                && !lower.contains("tokenheader")
                && !lower.contains("tokentype");
            if looks_token && value.chars().count() >= 20 {
                hints.push(WorkBuddyTokenHint {
                    entry: file.clone(),
                    path: path_in_doc.clone(),
                    length: value.chars().count(),
                    preview: mask(value),
                });
            }
        }
    }
    states.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    states.dedup_by(|a, b| a.uid == b.uid && a.domain == b.domain);
    hints.truncate(50);
    (states, hints)
}

/// 认证令牌条目（来自 WorkBuddy 的认证会话文件）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyAuthToken {
    /// 字段名（accessToken / refreshToken）。
    pub field: String,
    /// 是否为字段级加密存储。
    pub encrypted: bool,
    /// 密文（envelope）长度。
    pub envelope_len: usize,
    /// 过期时间（毫秒时间戳）。
    pub expires_at: Option<i64>,
    /// 剩余有效期（秒）。
    pub expires_in: Option<i64>,
}

/// 认证会话中的账号信息。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyAuthAccount {
    pub uid: Option<String>,
    pub uin: Option<String>,
    pub account_type: Option<String>,
    pub last_login: bool,
    pub is_admin: bool,
    pub unique_user_id: Option<String>,
    pub mp_open_id: Option<String>,
    pub oneid_account_id: Option<String>,
}

/// 认证会话（`Data/Public/auth/<id>.info`）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyAuthSession {
    /// 认证文件路径。
    pub file: Option<String>,
    /// 访问令牌是否仍在有效期内。
    pub logged_in: bool,
    pub token_type: Option<String>,
    pub domain: Option<String>,
    pub session_state: Option<String>,
    pub access_token: Option<WorkBuddyAuthToken>,
    pub refresh_token: Option<WorkBuddyAuthToken>,
    /// 最近刷新时间（毫秒时间戳）。
    pub last_refresh_at: Option<i64>,
    /// refresh 令牌过期时间（毫秒时间戳）。
    pub refresh_expires_at: Option<i64>,
    pub account: Option<WorkBuddyAuthAccount>,
    /// 采用字段级加密的字段名。
    pub encrypted_fields: Vec<String>,
    /// 观察记录（明文）。
    pub observation: Option<Value>,
    /// 说明。
    pub note: Option<String>,
}

/// 产品数据目录候选（WorkBuddy 桌面端与 IDE 扩展共用产品目录）。
fn auth_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Some(local) = std::env::var_os("LOCALAPPDATA") else {
        return out;
    };
    for product in ["CodeBuddyExtension", "WorkBuddy", "CodeBuddy"] {
        let dir = PathBuf::from(&local)
            .join(product)
            .join("Data")
            .join("Public")
            .join("auth");
        if dir.is_dir() {
            out.push(dir);
        }
    }
    out
}

/// 读取认证会话文件（WorkBuddy 桌面端与 IDE 扩展共享 `Data/Public/auth/<id>.info`）。
///
/// 文件结构：
/// ```json
/// { "auth": { "accessToken": { "$wbEncrypted": 1, "envelope": "..." },
///             "refreshToken": {...}, "tokenType": "Bearer",
///             "expiresAt": 1796951743699, ... },
///   "account": { "uid": "...", "nickname": { "$wbEncrypted": 1, ... } } }
/// ```
/// 令牌字段本身是密文（HKDF-SHA256 + AES-256-GCM），这里只读取元信息与有效期，
/// 不解密、不导出令牌明文。
pub(crate) fn read_auth_session() -> Option<WorkBuddyAuthSession> {
    let dir = auth_dirs().into_iter().next()?;
    let mut info_file = None;
    let mut observation = None;
    for entry in fs::read_dir(&dir).ok()?.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_string_lossy().to_string();
        if name.ends_with(".observation.json") {
            if let Ok(text) = fs::read_to_string(&path) {
                observation = serde_json::from_str::<Value>(&text).ok();
            }
        } else if name.ends_with(".info") {
            info_file = Some(path);
        }
    }
    let path = info_file?;
    let text = fs::read_to_string(&path).ok()?;
    let doc: Value = serde_json::from_str(&text).ok()?;
    let auth = doc.get("auth").cloned().unwrap_or(Value::Null);

    let token_of = |name: &str| -> Option<WorkBuddyAuthToken> {
        let node = auth.get(name)?;
        let envelope = node.get("envelope").and_then(|v| v.as_str());
        let encrypted = node.get("$wbEncrypted").and_then(|v| v.as_i64()) == Some(1);
        Some(WorkBuddyAuthToken {
            field: name.to_string(),
            encrypted: encrypted || envelope.is_some(),
            envelope_len: envelope.map(|s| s.len()).unwrap_or(0),
            expires_at: auth.get("expiresAt").and_then(|v| v.as_i64()),
            expires_in: auth.get("expiresIn").and_then(|v| v.as_i64()),
        })
    };

    let account = doc.get("account").map(|acc| WorkBuddyAuthAccount {
        uid: acc.get("uid").and_then(|v| v.as_str()).map(|s| s.to_string()),
        uin: acc.get("uin").and_then(|v| v.as_str()).map(|s| s.to_string()),
        account_type: acc.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        last_login: acc.get("lastLogin").and_then(|v| v.as_bool()).unwrap_or(false),
        is_admin: acc.get("isAdmin").and_then(|v| v.as_bool()).unwrap_or(false),
        unique_user_id: acc
            .get("uniqueUserId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        mp_open_id: acc
            .get("mpOpenId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        oneid_account_id: acc
            .get("oneidAccountId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
    });

    // 收集所有字段级加密的键名
    let mut encrypted_fields = Vec::new();
    fn walk_encrypted(node: &Value, prefix: &str, out: &mut Vec<String>) {
        match node {
            Value::Object(map) => {
                for (k, v) in map {
                    let p = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    if v.get("$wbEncrypted").and_then(|x| x.as_i64()) == Some(1) {
                        out.push(p);
                    } else {
                        walk_encrypted(v, &p, out);
                    }
                }
            }
            Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    walk_encrypted(v, &format!("{prefix}[{i}]"), out);
                }
            }
            _ => {}
        }
    }
    walk_encrypted(&doc, "", &mut encrypted_fields);

    let access_token = token_of("accessToken");
    let expires_at = auth.get("expiresAt").and_then(|v| v.as_i64());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    Some(WorkBuddyAuthSession {
        file: Some(path.display().to_string()),
        logged_in: access_token.is_some() && expires_at.map(|t| t > now).unwrap_or(false),
        token_type: auth.get("tokenType").and_then(|v| v.as_str()).map(|s| s.to_string()),
        domain: auth.get("domain").and_then(|v| v.as_str()).map(|s| s.to_string()),
        session_state: auth
            .get("sessionState")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        access_token,
        refresh_token: token_of("refreshToken"),
        last_refresh_at: auth.get("lastRefreshTime").and_then(|v| v.as_i64()),
        refresh_expires_at: auth.get("refreshExpiresAt").and_then(|v| v.as_i64()),
        account,
        encrypted_fields,
        observation,
        note: None,
    })
}

/// 读取主密钥容器元信息。
fn read_keyblob(root: &Path) -> Option<WorkBuddyKeyblobInfo> {
    let path = root.join("keyblob");
    let text = fs::read_to_string(&path).ok()?;
    let doc: Value = serde_json::from_str(&text).ok()?;
    let slots = doc.get("slots").and_then(|v| v.as_array());
    let first = slots.and_then(|s| s.first());
    let updated_at = fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);
    Some(WorkBuddyKeyblobInfo {
        version: doc.get("version").and_then(|v| v.as_i64()).unwrap_or(0),
        key_id: doc
            .get("keyId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        slots: slots.map(|s| s.len()).unwrap_or(0),
        slot_type: first
            .and_then(|s| s.get("type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        protector_key_id: first
            .and_then(|s| s.get("protectorKeyId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        updated_at,
    })
}

/// Windows DPAPI 解密（Chromium v10 之前的 cookie 格式）。
fn dpapi_unprotect(blob: &[u8]) -> Option<Vec<u8>> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(blob);
    let script = format!(
        "$ErrorActionPreference='Stop';$b=[Convert]::FromBase64String('{encoded}');\
         $p=[System.Security.Cryptography.ProtectedData]::Unprotect($b,$null,\
         [System.Security.Cryptography.DataProtectionScope]::CurrentUser);\
         [Convert]::ToBase64String($p)"
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    base64::engine::general_purpose::STANDARD.decode(text.as_bytes()).ok()
}

/// Chrome 时间戳（1601 起算的微秒）转毫秒。
fn chrome_time_to_millis(value: i64) -> Option<i64> {
    if value <= 0 {
        return None;
    }
    Some(value / 1000 - 11_644_473_600_000)
}

/// 读取 Chromium Cookie 库（尽力而为；被占用时返回说明）。
fn read_cookies(root: &Path) -> (Vec<WorkBuddyCookie>, Option<String>) {
    let path = root.join("app").join("session").join("Network").join("Cookies");
    if !path.exists() {
        return (Vec::new(), Some("未找到 Cookie 库".to_string()));
    }
    let conn = match Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            return (
                Vec::new(),
                Some(format!(
                    "Cookie 库被 WorkBuddy 占用无法读取（{e}），退出 WorkBuddy 后可重新读取"
                )),
            )
        }
    };
    // 旧版 Chromium 的 cookie 表名可能不同，逐一尝试。
    let mut rows: Vec<(String, String, Option<String>, bool, bool, i64, Option<Vec<u8>>, String)> =
        Vec::new();
    let query = "SELECT host_key, name, path, is_secure, is_httponly, expires_utc, \
                 encrypted_value, value FROM cookies ORDER BY last_access_utc DESC";
    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(e) => return (Vec::new(), Some(format!("读取 Cookie 失败：{e}"))),
    };
    let mapped = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, bool>(3).unwrap_or(false),
            row.get::<_, bool>(4).unwrap_or(false),
            row.get::<_, i64>(5).unwrap_or(0),
            row.get::<_, Option<Vec<u8>>>(6)?,
            row.get::<_, String>(7).unwrap_or_default(),
        ))
    });
    if let Ok(iter) = mapped {
        for r in iter.flatten() {
            rows.push(r);
        }
    }

    let mut out = Vec::new();
    for (host, name, cookie_path, secure, http_only, expires, encrypted, plain) in rows {
        let encrypted_blob = encrypted.filter(|b| !b.is_empty());
        let mut preview = mask(&plain);
        let mut value_len = plain.chars().count();
        let mut encrypted_flag = false;
        if let Some(blob) = encrypted_blob {
            encrypted_flag = true;
            value_len = blob.len();
            if blob.starts_with(b"v10") || blob.starts_with(b"v11") {
                if blob.starts_with(b"v10") {
                    if let Some(plain_bytes) = dpapi_unprotect(&blob[3..]) {
                        if let Ok(text) = String::from_utf8(plain_bytes) {
                            value_len = text.chars().count();
                            preview = mask(&text);
                        }
                    }
                }
            }
        }
        out.push(WorkBuddyCookie {
            host,
            name,
            path: cookie_path,
            secure,
            http_only,
            expires_at: chrome_time_to_millis(expires),
            value_len,
            encrypted: encrypted_flag,
            preview,
        });
    }
    (out, None)
}

/// 读取会话数据。
#[tauri::command]
pub fn workbuddy_sessions(limit: Option<i64>) -> WorkBuddySessionOverview {
    let Some(root) = data_dir() else {
        return WorkBuddySessionOverview {
            note: Some("无法定位用户目录".to_string()),
            ..Default::default()
        };
    };
    let db_path = root.join("workbuddy.db");
    let mut overview = WorkBuddySessionOverview {
        db_path: Some(db_path.display().to_string()),
        ..Default::default()
    };
    if !db_path.exists() {
        overview.note = Some("未找到 WorkBuddy 数据库".to_string());
        return overview;
    }
    let conn = match Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            overview.note = Some(format!("打开数据库失败：{e}"));
            return overview;
        }
    };

    overview.total_sessions = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    overview.active_sessions = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE deleted_at IS NULL AND status IS NOT 'completed'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    overview.total_used = conn
        .query_row("SELECT COALESCE(SUM(used), 0) FROM session_usage", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    // 额度消耗分散在每行的 credit_json 中，逐行求和。
    if let Ok(mut stmt) = conn.prepare("SELECT credit_json FROM session_usage") {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, Option<String>>(0)) {
            for credit in rows.flatten() {
                let Some(raw) = credit else { continue };
                if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(raw.as_str()) {
                    for (_, v) in map {
                        if let Some(n) = v.as_f64() {
                            overview.total_credit += n;
                        }
                    }
                }
            }
        }
    }

    let take = limit.unwrap_or(50).clamp(1, 200);
    let sql = "SELECT s.id, s.title, s.cwd, s.status, s.model, s.expert_id, s.source_mode, \
               s.permission_mode, COALESCE(s.is_background_automation, 0), s.created_at, \
               s.updated_at, u.used, u.size, u.credit_json \
               FROM sessions s LEFT JOIN session_usage u ON u.session_id = s.id \
               WHERE s.deleted_at IS NULL ORDER BY s.updated_at DESC LIMIT ?1";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([take], |row| {
            let credit_json: Option<String> = row.get(13)?;
            let mut credit = 0.0;
            if let Some(raw) = credit_json {
                if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&raw) {
                    for (_, v) in map {
                        if let Some(n) = v.as_f64() {
                            credit += n;
                        }
                    }
                }
            }
            Ok(WorkBuddySessionRow {
                id: row.get(0)?,
                title: row.get(1)?,
                cwd: row.get(2)?,
                status: row.get(3)?,
                model: row.get(4)?,
                expert_id: row.get(5)?,
                source_mode: row.get(6)?,
                permission_mode: row.get(7)?,
                background: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                used: row.get(11)?,
                context_size: row.get(12)?,
                credit,
            })
        }) {
            for r in rows.flatten() {
                overview.sessions.push(r);
            }
        }
    }

    let sql = "SELECT path, last_opened_at, (SELECT COUNT(*) FROM sessions s \
               WHERE s.cwd = w.path AND s.deleted_at IS NULL) FROM workspaces w \
               ORDER BY last_opened_at DESC LIMIT 20";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(WorkBuddyWorkspaceRow {
                path: row.get(0)?,
                last_opened_at: row.get(1)?,
                sessions: row.get(2)?,
            })
        }) {
            for r in rows.flatten() {
                overview.workspaces.push(r);
            }
        }
    }

    overview
}

/// 读取凭据数据。
#[tauri::command]
pub fn workbuddy_credentials() -> WorkBuddyCredentialOverview {
    let Some(root) = data_dir() else {
        return WorkBuddyCredentialOverview {
            note: Some("无法定位用户目录".to_string()),
            ..Default::default()
        };
    };
    let mut overview = WorkBuddyCredentialOverview::default();
    // 真实登录态：共享产品目录下的认证会话文件
    overview.auth_session = read_auth_session();
    let (states, hints) = scan_local_storage(&root);
    overview.logged_in = states.iter().any(|s| s.enabled)
        || overview
            .auth_session
            .as_ref()
            .map(|a| a.logged_in)
            .unwrap_or(false);
    overview.login_states = states;
    overview.token_hints = hints;
    overview.keyblob = read_keyblob(&root);
    let (cookies, err) = read_cookies(&root);
    overview.cookies = cookies;
    overview.cookies_error = err;

    // CLI 目录与登录痕迹
    if let Some(home) = home_dir() {
        let cli = home.join(".codebuddy");
        if cli.exists() {
            overview.cli_dir = Some(cli.display().to_string());
            let has_credential = ["credentials.json", "auth.json", ".credentials.json"]
                .iter()
                .any(|name| cli.join(name).exists());
            overview.cli_logged_in = has_credential;
        }
    }
    if !overview.logged_in && overview.auth_session.is_none() && overview.cookies.is_empty() {
        overview.note = Some(
            "未检出有效登录态：既没有共享目录下的认证会话文件，\
             本地存储中的登录态记录也是 disabled"
                .to_string(),
        );
    }
    overview
}
#[cfg(test)]
mod tests {
    use super::*;

    /// 读取本机数据并打印，用于人工核对。
    #[test]
    fn dump_local_data() {
        let sessions = workbuddy_sessions(Some(3));
        println!(
            "sessions: total={} active={} used={} credit={} rows={} workspaces={}",
            sessions.total_sessions,
            sessions.active_sessions,
            sessions.total_used,
            sessions.total_credit,
            sessions.sessions.len(),
            sessions.workspaces.len()
        );
        for row in &sessions.sessions {
            println!(
                "  - {} | {} | {} | used={:?} | {}",
                row.id,
                row.model.clone().unwrap_or_default(),
                row.status.clone().unwrap_or_default(),
                row.used,
                row.cwd.clone().unwrap_or_default()
            );
        }

        let cred = workbuddy_credentials();
        println!(
            "credentials: logged_in={} states={} cookies={} hints={} keyblob={:?} cli_logged={} err={:?} note={:?}",
            cred.logged_in,
            cred.login_states.len(),
            cred.cookies.len(),
            cred.token_hints.len(),
            cred.keyblob.map(|k| (k.version, k.slot_type.clone(), k.slots)),
            cred.cli_logged_in,
            cred.cookies_error,
            cred.note
        );
        for st in &cred.login_states {
            println!(
                "  state: uid={:?} type={:?} domain={:?} api={:?} enabled={} updated={:?}",
                st.uid, st.account_type, st.domain, st.api_path, st.enabled, st.updated_at
            );
        }
        for h in &cred.token_hints {
            println!("  hint: {} {} len={}", h.entry, h.path, h.length);
        }
        for c in &cred.cookies {
            println!(
                "  cookie: {} {} len={} encrypted={} preview={:?}",
                c.host, c.name, c.value_len, c.encrypted, c.preview
            );
        }
        if let Some(a) = &cred.auth_session {
            println!(
                "auth: logged_in={} file={} token_type={:?} domain={:?} expires_at={:?} refresh_at={:?}",
                a.logged_in,
                a.file.clone().unwrap_or_default(),
                a.token_type,
                a.domain,
                a.access_token.as_ref().and_then(|t| t.expires_at),
                a.refresh_expires_at
            );
            println!("  account: {:?}", a.account);
            println!("  encrypted_fields: {:?}", a.encrypted_fields);
        } else {
            println!("auth: <未找到认证会话文件>");
        }
    }
}
