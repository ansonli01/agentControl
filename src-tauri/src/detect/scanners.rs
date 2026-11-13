use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::runtime::process::{run_capture_timeout, which};

/// 附加的可执行文件搜索目录（npm 全局 bin、nodejs 安装目录等）。
pub async fn extra_bin_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(out) = run_capture_timeout(
        "npm",
        &["prefix".to_string(), "-g".to_string()],
        Duration::from_secs(20),
    )
    .await
    {
        let prefix = out.stdout.trim();
        if !prefix.is_empty() {
            let p = PathBuf::from(prefix);
            dirs.push(p.clone());
            dirs.push(p.join("node_modules").join(".bin"));
        }
    }
    if let Some(appdata) = dirs::data_dir() {
        dirs.push(appdata.join("npm"));
    }
    dirs
}

/// 在 PATH 与附加目录中查找可执行文件。
pub fn find_executable(name: &str, extra: &[PathBuf]) -> Option<PathBuf> {
    if let Some(p) = which(name) {
        return Some(p);
    }
    let exts = [".cmd", ".bat", ".exe", ".ps1", ""];
    for dir in extra {
        for ext in exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 执行版本探测命令，返回首行输出。
pub async fn probe_version(
    exe: &Path,
    args: &[String],
    timeout: Duration,
) -> Option<String> {
    let out = run_capture_timeout(exe.to_string_lossy().to_string().as_str(), args, timeout)
        .await
        .ok()?;
    let text = if out.stdout.trim().is_empty() {
        out.stderr
    } else {
        out.stdout
    };
    let first = text.lines().next()?.trim().to_string();
    if first.is_empty() {
        None
    } else {
        Some(first)
    }
}
/// 读取 npm 全局包清单，返回 包名 -> 版本。
pub async fn npm_global_packages(timeout: Duration) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let out = match run_capture_timeout(
        "npm",
        &["ls".to_string(), "-g".to_string(), "--depth=0".to_string(), "--json".to_string()],
        timeout,
    )
    .await
    {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!("npm ls failed: {e}");
            return map;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&out.stdout) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("npm ls json parse failed: {e}");
            return map;
        }
    };
    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
        for (name, info) in deps {
            let version = info
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            map.insert(name.clone(), version);
        }
    }
    map
}

/// 关键词归一化：忽略大小写、空格、连字符与下划线，
/// 使 `OpenCode Desktop`、`opencode-desktop`、`@opencodedesktop` 能互相匹配。
fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// 桌面客户端扫描结果。
#[derive(Debug, Clone)]
pub struct DesktopHit {
    pub name: String,
    pub version: Option<String>,
    pub location: Option<String>,
    pub source: String,
}
/// 扫描 Windows 卸载注册表，匹配桌面客户端。
#[cfg(windows)]
pub fn scan_uninstall_registry(keywords: &[String]) -> Vec<DesktopHit> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let roots: Vec<(winreg::HKEY, &str)> = vec![
        (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];
    let mut hits = Vec::new();
    for (root, path) in roots {
        let Ok(key) = RegKey::predef(root).open_subkey(path) else {
            continue;
        };
        for name in key.enum_keys().flatten() {
            let Ok(sub) = key.open_subkey(&name) else {
                continue;
            };
            let display = sub.get_value::<String, _>("DisplayName").unwrap_or_default();
            if display.is_empty() {
                continue;
            }
            if keywords.is_empty()
                || keywords.iter().any(|k| normalize(&display).contains(&normalize(k)))
            {
                let version = sub.get_value::<String, _>("DisplayVersion").ok();
                let location = sub.get_value::<String, _>("InstallLocation").ok();
                hits.push(DesktopHit {
                    name: display,
                    version,
                    location,
                    source: "registry".into(),
                });
            }
        }
    }
    hits
}

/// 非 Windows 平台暂不扫描注册表。
#[cfg(not(windows))]
pub fn scan_uninstall_registry(_keywords: &[String]) -> Vec<DesktopHit> {
    Vec::new()
}
/// 扫描常见安装目录，匹配桌面客户端目录名。
pub fn scan_install_dirs(keywords: &[String]) -> Vec<DesktopHit> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(local) = dirs::data_local_dir() {
        roots.push(local.join("Programs"));
    }
    for env in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        if let Some(dir) = std::env::var_os(env) {
            roots.push(PathBuf::from(dir));
        }
    }
    let mut hits = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = path
                .file_name()
                .map(|n| normalize(&n.to_string_lossy()))
                .unwrap_or_default();
            if keywords.is_empty()
                || keywords
                    .iter()
                    .any(|k| dir_name.contains(&normalize(k)))
            {
                hits.push(DesktopHit {
                    name: path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    version: None,
                    location: Some(path.to_string_lossy().to_string()),
                    source: "install-dir".into(),
                });
            }
        }
    }
    hits
}