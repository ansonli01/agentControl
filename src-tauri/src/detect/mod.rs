pub mod scanners;

use std::collections::HashMap;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::agent::model::{AgentDefinition, AgentKind, AgentStatus, DetectSource};
use crate::events::{events, DetectProgress};

/// 探测全部智能体，返回状态列表。
pub async fn detect_all(
    defs: &[AgentDefinition],
    timeout: Duration,
    handle: Option<AppHandle>,
) -> Vec<AgentStatus> {
    let npm_map = scanners::npm_global_packages(timeout).await;
    let total = defs.len();
    let handle_ref = handle.clone();

    let futures = defs.iter().enumerate().map(|(idx, def)| {
        let npm_map = npm_map.clone();
        let timeout = timeout;
        let handle_ref = handle_ref.clone();
        async move {
            let status = detect_one(def, &npm_map, timeout).await;
            if let Some(h) = handle_ref.as_ref() {
                let _ = h.emit(
                    events::DETECT_PROGRESS,
                    DetectProgress {
                        agent_id: def.id.clone(),
                        agent_name: def.name.clone(),
                        phase: "已检测".into(),
                        done: idx + 1,
                        total,
                        finished: true,
                    },
                );
            }
            status
        }
    });

    futures_util::future::join_all(futures).await
}
/// 探测单个智能体。
pub async fn detect_one(
    def: &AgentDefinition,
    npm_map: &HashMap<String, String>,
    timeout: Duration,
) -> AgentStatus {
    let mut status = AgentStatus {
        agent_id: def.id.clone(),
        installed: false,
        version: None,
        path: None,
        sources: Vec::new(),
        note: None,
        checked_at: chrono::Utc::now().timestamp_millis(),
        version_unknown: false,
    };

    match def.kind {
        AgentKind::Cli => detect_cli(def, npm_map, timeout, &mut status).await,
        AgentKind::Desktop => detect_desktop(def, &mut status),
    }
    status
}

async fn detect_cli(
    def: &AgentDefinition,
    npm_map: &HashMap<String, String>,
    timeout: Duration,
    status: &mut AgentStatus,
) {
    let extra = scanners::extra_bin_dirs().await;
    let mut exe: Option<std::path::PathBuf> = None;
    for name in &def.cli_names {
        if let Some(p) = scanners::find_executable(name, &extra) {
            exe = Some(p);
            break;
        }
    }

    if let Some(pkg) = &def.npm_package {
        if let Some(ver) = npm_map.get(pkg) {
            status.installed = true;
            status.version = Some(ver.clone());
            status.sources.push(DetectSource {
                kind: "npm".into(),
                detail: Some(pkg.clone()),
            });
        }
    }

    if let Some(path) = &exe {
        status.installed = true;
        status.path = Some(path.to_string_lossy().to_string());
        status.sources.push(DetectSource {
            kind: "path".into(),
            detail: Some(path.to_string_lossy().to_string()),
        });
        if status.version.is_none() {
            let args = if def.version_args.is_empty() {
                vec!["--version".to_string()]
            } else {
                def.version_args.clone()
            };
            match scanners::probe_version(path, &args, timeout).await {
                Some(v) => status.version = Some(v),
                None => {
                    status.version_unknown = true;
                    status.note = Some("已安装，版本探测失败".into());
                }
            }
        }
    }
}
fn detect_desktop(def: &AgentDefinition, status: &mut AgentStatus) {
    // 仅使用名称与命令名做匹配：厂商名过于宽泛（如 SST）容易误命中无关条目
    let mut keywords: Vec<String> = def.cli_names.clone();
    keywords.push(def.name.clone());

    let hits = scanners::scan_uninstall_registry(&keywords);
    if let Some(hit) = hits.into_iter().next() {
        status.installed = true;
        status.version = hit.version;
        status.path = hit.location;
        status.sources.push(DetectSource {
            kind: hit.source,
            detail: Some(hit.name),
        });
        return;
    }

    let dirs = scanners::scan_install_dirs(&def.cli_names);
    if let Some(hit) = dirs.into_iter().next() {
        status.installed = true;
        status.path = hit.location;
        status.sources.push(DetectSource {
            kind: hit.source,
            detail: Some(hit.name),
        });
        status.note = Some("通过安装目录识别，可能为便携版".into());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::registry;

    #[tokio::test]
    async fn detect_all_covers_every_agent() {
        let defs = registry::builtin_agents();
        let statuses = detect_all(&defs, Duration::from_secs(20), None).await;
        assert_eq!(statuses.len(), defs.len());
        for s in &statuses {
            assert!(!s.agent_id.is_empty());
            assert!(s.checked_at > 0);
        }
    }

    #[tokio::test]
    async fn npm_probe_returns_packages_or_empty() {
        let map = scanners::npm_global_packages(Duration::from_secs(30)).await;
        for (name, version) in &map {
            assert!(!name.is_empty());
            assert!(!version.is_empty());
        }
    }
}