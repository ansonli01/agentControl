//! WorkBuddy 内置 CodeBuddy CLI 引擎。
//!
//! WorkBuddy 安装包已经把 CLI 完整落到磁盘：
//! `<安装目录>/resources/app.asar.unpacked/cli`（无需解包 app.asar）。
//! 其中 `bin/codebuddy` 是 Node 脚本，主 bundle 为 `dist/codebuddy-lite-wb.mjs`，
//! 需要 `CODEBUDDY_FORCE_LITE_WB_BUNDLE=1` 才会命中 lite-wb 产物。
//!
//! 运行时复用 WorkBuddy 自带的 Node（`~/.workbuddy/binaries/node/versions/<v>/node.exe`），
//! 因此 CLI 能直接读到 WorkBuddy 的模型配置、插件与登录态。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::error::AppResult;

/// CLI 就绪信息。
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyCliInfo {
    /// 是否可用于非交互调用。
    pub available: bool,
    /// CLI 入口脚本路径。
    pub cli_path: Option<String>,
    /// CLI 根目录。
    pub cli_root: Option<String>,
    /// 使用的 Node 运行时。
    pub node_path: Option<String>,
    /// CLI 版本。
    pub version: Option<String>,
    /// 命中的 bundle 变体。
    pub bundle: String,
    /// 定位方式：`env` / `process` / `registry` / `common`。
    pub source: Option<String>,
    /// 未就绪时的原因。
    pub reason: Option<String>,
}

/// 一次 CLI 调用的结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyCliRun {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub elapsed_ms: u64,
    /// 输出里检测到未登录时的提示。
    pub auth_required: bool,
}

/// CLI 入口相对安装目录的路径。
const CLI_REL: &str = "resources/app.asar.unpacked/cli";

/// 从 WorkBuddy 可执行文件路径推导 CLI 根目录。
fn cli_root_from_exe(exe: &str) -> Option<PathBuf> {
    let dir = Path::new(exe).parent()?;
    let root = dir.join(CLI_REL);
    root.exists().then_some(root)
}

/// 运行中的 WorkBuddy.exe 路径。
#[cfg(windows)]
fn running_exe() -> Option<String> {
    use std::process::Command;
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Process WorkBuddy -ErrorAction SilentlyContinue | Select-Object -First 1).Path",
        ])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!path.is_empty()).then_some(path)
}

#[cfg(not(windows))]
fn running_exe() -> Option<String> {
    None
}

/// 从注册表读取 WorkBuddy 安装目录。
#[cfg(windows)]
fn registry_install_dir() -> Option<String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let uninstall = hkcu
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            KEY_READ,
        )
        .ok()?;
    for name in uninstall.enum_keys().flatten() {
        let Ok(sub) = uninstall.open_subkey_with_flags(&name, KEY_READ) else {
            continue;
        };
        let display: String = sub.get_value("DisplayName").unwrap_or_default();
        if !display.to_lowercase().contains("workbuddy") {
            continue;
        }
        let location: String = sub.get_value("InstallLocation").unwrap_or_default();
        if !location.is_empty() {
            return Some(location);
        }
    }
    None
}

#[cfg(not(windows))]
fn registry_install_dir() -> Option<String> {
    None
}

/// 常见安装目录候选。
fn common_install_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        out.push(PathBuf::from(local).join("Programs").join("WorkBuddy"));
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        out.push(PathBuf::from(program_files).join("WorkBuddy"));
    }
    out
}

/// 定位 CLI 根目录。
pub(super) fn locate_cli_root() -> (Option<PathBuf>, Option<String>) {
    // 1. 显式覆盖。
    if let Some(custom) = std::env::var_os("WORKBUDDY_CLI_ROOT") {
        let p = PathBuf::from(custom);
        if p.exists() {
            return (Some(p), Some("env".to_string()));
        }
    }
    // 2. 运行中的客户端进程。
    if let Some(exe) = running_exe() {
        if let Some(root) = cli_root_from_exe(&exe) {
            return (Some(root), Some("process".to_string()));
        }
    }
    // 3. 注册表。
    if let Some(dir) = registry_install_dir() {
        let root = PathBuf::from(dir).join(CLI_REL);
        if root.exists() {
            return (Some(root), Some("registry".to_string()));
        }
    }
    // 4. 常见路径。
    for dir in common_install_dirs() {
        let root = dir.join(CLI_REL);
        if root.exists() {
            return (Some(root), Some("common".to_string()));
        }
    }
    (None, None)
}

/// CLI 可用的自定义模型（来自 CLI 配置目录的 models.json）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyCliModel {
    /// 模型 id（原始）。
    pub id: String,
    /// 展示名。
    pub name: String,
    /// 传给 `--model` 的值。
    pub model: String,
    /// 是否支持工具调用。
    pub supports_tool_call: bool,
    /// 是否为本机端点（127.0.0.1 / localhost）。
    pub local: bool,
}

/// 读取 CLI 可用的自定义模型。
///
/// CLI 只识别自己配置目录（`~/.codebuddy`）里的模型，因此以该目录为准，
/// `~/.workbuddy`（WorkBuddy 桌面端）作为补充。
pub(super) fn read_cli_models() -> Vec<WorkBuddyCliModel> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let mut out: Vec<WorkBuddyCliModel> = Vec::new();
    for dir in [home.join(".codebuddy"), home.join(".workbuddy")] {
        let path = dir.join("models.json");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let items = value
            .get("models")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_else(|| {
                value
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
            });
        for item in items {
            let Some(id) = item.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            if out.iter().any(|m| m.id == id) {
                continue;
            }
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string();
            let supports_tool_call = item
                .get("supportsToolCall")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let local = url.starts_with("http://127.0.0.1")
                || url.starts_with("http://localhost")
                || url.starts_with("https://127.0.0.1");
            out.push(WorkBuddyCliModel {
                id: id.to_string(),
                name,
                model: format!("custom-local:{id}"),
                supports_tool_call,
                local,
            });
        }
    }
    // 远程端点优先（本机端点常未启动）。
    out.sort_by_key(|m| m.local);
    out
}

/// 查询 CLI 可用模型列表。
#[tauri::command]
pub fn workbuddy_cli_models() -> Vec<WorkBuddyCliModel> {
    read_cli_models()
}

/// 定位 Node 运行时。
pub(super) fn locate_node() -> Option<PathBuf> {
    // WorkBuddy 自带的 Node。
    if let Some(home) = dirs::home_dir() {
        let versions = home
            .join(".workbuddy")
            .join("binaries")
            .join("node")
            .join("versions");
        if let Ok(current) = std::fs::read_to_string(versions.join("current")) {
            let name = current.trim();
            if !name.is_empty() {
                let exe = versions.join(name).join("node.exe");
                if exe.exists() {
                    return Some(exe);
                }
                let exe = versions.join(name).join("bin").join("node");
                if exe.exists() {
                    return Some(exe);
                }
            }
        }
    }
    // 系统 PATH。
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let exe = dir.join("node.exe");
            if exe.exists() {
                return Some(exe);
            }
            let exe = dir.join("node");
            if exe.exists() {
                return Some(exe);
            }
        }
    }
    None
}

/// 查询 CLI 就绪状态。
#[tauri::command]
pub fn workbuddy_cli_info() -> WorkBuddyCliInfo {
    let (root, source) = locate_cli_root();
    let Some(root) = root else {
        return WorkBuddyCliInfo {
            available: false,
            bundle: "lite-wb".to_string(),
            reason: Some(
                "未找到 WorkBuddy 内置 CLI，请先启动一次 WorkBuddy 或设置 WORKBUDDY_CLI_ROOT".to_string(),
            ),
            ..Default::default()
        };
    };
    let cli = root.join("bin").join("codebuddy");
    if !cli.exists() {
        return WorkBuddyCliInfo {
            available: false,
            cli_root: Some(root.display().to_string()),
            source,
            bundle: "lite-wb".to_string(),
            reason: Some("CLI 入口缺失：bin/codebuddy".to_string()),
            ..Default::default()
        };
    }
    let node = locate_node();
    let mut info = WorkBuddyCliInfo {
        available: node.is_some(),
        cli_path: Some(cli.display().to_string()),
        cli_root: Some(root.display().to_string()),
        node_path: node.as_ref().map(|p| p.display().to_string()),
        version: None,
        bundle: "lite-wb".to_string(),
        source,
        reason: if node.is_none() {
            Some("未找到 Node 运行时（需 18.20.8+）".to_string())
        } else {
            None
        },
    };
    if let (Some(node), Some(cli)) = (node, &info.cli_path) {
        if let Some(v) = run_cli_version(&node, Path::new(cli)) {
            info.version = Some(v);
        }
    }
    info
}

/// 读取 CLI 版本号。
fn run_cli_version(node: &Path, cli: &Path) -> Option<String> {
    let out = std::process::Command::new(node)
        .arg(cli)
        .arg("--version")
        .env("CODEBUDDY_FORCE_LITE_WB_BUNDLE", "1")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().next()?.trim().to_string();
    (!line.is_empty()).then_some(line)
}

/// 以非交互方式运行一次 WorkBuddy CLI。
#[tauri::command]
pub async fn workbuddy_cli_run(
    prompt: String,
    cwd: Option<String>,
    timeout_secs: Option<u64>,
) -> AppResult<WorkBuddyCliRun> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(crate::error::AppError::msg("提示词不能为空"));
    }
    let info = workbuddy_cli_info();
    let (Some(node), Some(cli)) = (info.node_path.clone(), info.cli_path.clone()) else {
        return Err(crate::error::AppError::msg(
            info.reason.unwrap_or_else(|| "WorkBuddy CLI 不可用".to_string()),
        ));
    };

    let mut cmd = tokio::process::Command::new(node);
    cmd.arg(cli)
        .arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("text")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .env("CODEBUDDY_FORCE_LITE_WB_BUNDLE", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(dir) = cwd.as_ref().filter(|d| !d.trim().is_empty()) {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            cmd.current_dir(&path);
            cmd.arg("--add-dir").arg(&path);
        }
    }

    let started = Instant::now();
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(180).clamp(5, 1800));
    let output = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(crate::error::AppError::msg(e.to_string())),
        Err(_) => {
            return Ok(WorkBuddyCliRun {
                ok: false,
                stdout: String::new(),
                stderr: format!("调用超时（{}s）", timeout.as_secs()),
                exit_code: None,
                elapsed_ms: started.elapsed().as_millis() as u64,
                auth_required: false,
            })
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");
    let auth_required = combined.contains("Authentication required")
        || combined.contains("use /login")
        || combined.contains("未登录");
    Ok(WorkBuddyCliRun {
        ok: output.status.success() && !auth_required,
        stdout,
        stderr,
        exit_code: output.status.code(),
        elapsed_ms: started.elapsed().as_millis() as u64,
        auth_required,
    })
}
