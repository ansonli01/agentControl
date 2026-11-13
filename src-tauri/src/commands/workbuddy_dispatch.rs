//! 向 WorkBuddy 内置 CLI 派发任务。
//!
//! CLI 以 `-p --output-format stream-json` 非交互运行，每行一个 JSON 事件：
//! - `{"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}` → 助手文本
//! - `{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash"}]}}` → 工具调用
//! - `{"type":"result","is_error":true,"errors":["..."]}` → 错误
//!
//! 事件通过 `workbuddy://log` / `workbuddy://status` 推给前端，完成后写入历史任务。

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::error::{AppError, AppResult};
use crate::events::{events, WorkBuddyLogLine, WorkBuddyTaskStatus};
use crate::state::AppState;
use crate::tasks::TaskRecord;

use super::app_state;
use super::workbuddy_cli::{locate_cli_root, locate_node, read_cli_models};

/// 派发任务给 WorkBuddy CLI，返回任务 id。
#[tauri::command]
pub fn workbuddy_dispatch_start(
    app: AppHandle,
    prompt: String,
    cwd: Option<String>,
    model: Option<String>,
) -> AppResult<String> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::msg("派发内容不能为空"));
    }
    let (Some(root), _) = locate_cli_root() else {
        return Err(AppError::msg(
            "未找到 WorkBuddy 内置 CLI，请先启动一次 WorkBuddy 或设置 WORKBUDDY_CLI_ROOT",
        ));
    };
    let cli = root.join("bin").join("codebuddy");
    if !cli.exists() {
        return Err(AppError::msg("CLI 入口缺失：bin/codebuddy"));
    }
    let Some(node) = locate_node() else {
        return Err(AppError::msg("未找到 Node 运行时（需 18.20.8+）"));
    };

    // 模型：显式指定优先，否则取第一个支持工具调用的自定义模型。
    let models = read_cli_models();
    let model = model
        .filter(|m| !m.trim().is_empty())
        .or_else(|| {
            models
                .iter()
                .find(|m| m.supports_tool_call)
                .map(|m| m.model.clone())
        })
        .unwrap_or_else(|| "auto".to_string());

    let workdir = cwd
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let task_id = uuid::Uuid::new_v4().to_string();
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let state = app_state(&app)?;
        let mut handles = state
            .workbuddy_handles
            .lock()
            .map_err(|e| AppError::msg(e.to_string()))?;
        handles.insert(
            task_id.clone(),
            crate::state::WorkBuddyHandle {
                label: prompt.chars().take(40).collect(),
                cwd: workdir.display().to_string(),
                model: model.clone(),
                cancelled: cancel.clone(),
            },
        );
    }

    let handle = app.clone();
    let tid = task_id.clone();
    tauri::async_runtime::spawn(async move {
        run_dispatch(handle, tid, node, cli, prompt, workdir, model, cancel).await;
    });
    Ok(task_id)
}

/// 取消派发任务。
#[tauri::command]
pub fn workbuddy_dispatch_cancel(app: AppHandle, task_id: String) -> AppResult<()> {
    let state = app_state(&app)?;
    let handles = state
        .workbuddy_handles
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?;
    if let Some(h) = handles.get(&task_id) {
        h.cancelled.store(true, Ordering::Relaxed);
    }
    Ok(())
}

/// 从 stream-json 事件行中提取可展示内容。
fn extract_event(line: &str) -> Option<(String, String)> {
    let value = serde_json::from_str::<Value>(line).ok()?;
    let kind = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "assistant" => {
            let content = value
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array());
            let Some(content) = content else {
                return None;
            };
            let mut text = String::new();
            let mut tool = String::new();
            for item in content {
                let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match item_type {
                    "text" => {
                        if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                            text.push_str(t);
                        }
                    }
                    "tool_use" => {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("tool");
                        tool.push_str(&format!("[调用工具 {name}]"));
                    }
                    _ => {}
                }
            }
            if !tool.is_empty() {
                return Some(("tool".to_string(), tool));
            }
            (!text.is_empty()).then_some(("text".to_string(), text))
        }
        "result" => {
            let is_error = value
                .get("is_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !is_error {
                return None;
            }
            let msg = value
                .get("errors")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    value
                        .get("result")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "任务执行失败".to_string());
            Some(("error".to_string(), msg))
        }
        _ => None,
    }
}

/// 后台执行派发任务。
async fn run_dispatch(
    app: AppHandle,
    task_id: String,
    node: PathBuf,
    cli: PathBuf,
    prompt: String,
    workdir: PathBuf,
    model: String,
    cancel: Arc<AtomicBool>,
) {
    let started = chrono::Utc::now().timestamp_millis();
    emit_status(&app, &task_id, "running", Some("任务已派发".into()), None);

    let mut cmd = tokio::process::Command::new(node);
    cmd.arg(cli)
        .arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .arg("--model")
        .arg(&model)
        .arg("--add-dir")
        .arg(&workdir)
        .current_dir(&workdir)
        .env("CODEBUDDY_FORCE_LITE_WB_BUNDLE", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("无法启动 WorkBuddy CLI：{e}");
            emit_status(&app, &task_id, "failed", Some(msg.clone()), None);
            write_history(&app, &task_id, &workdir, &model, "failed", 0, Some(msg));
            return;
        }
    };

    // stderr 单独读取，避免管道阻塞。
    if let Some(stderr) = child.stderr.take() {
        let app = app.clone();
        let tid = task_id.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let text = line.trim();
                if text.is_empty() {
                    continue;
                }
                emit_log(&app, &tid, "error", text.to_string());
            }
        });
    }

    let stdout = child.stdout.take();
    let mut collected = String::new();
    let mut cancelled = false;
    let mut failed: Option<String> = None;
    let mut saw_error = false;

    if let Some(stdout) = stdout {
        let mut lines = BufReader::new(stdout).lines();
        loop {
            tokio::select! {
                line = lines.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            if let Some((stream, text)) = extract_event(&line) {
                                if stream == "text" {
                                    collected.push_str(&text);
                                }
                                if stream == "error" {
                                    saw_error = true;
                                }
                                emit_log(&app, &task_id, &stream, text);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            failed = Some(format!("读取 CLI 输出失败：{e}"));
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {
                    if cancel.load(Ordering::Relaxed) {
                        cancelled = true;
                        let _ = child.kill().await;
                        break;
                    }
                }
            }
        }
    }

    let status = child.wait().await.ok();
    let duration = chrono::Utc::now().timestamp_millis() - started;

    if cancelled {
        emit_status(&app, &task_id, "cancelled", Some("任务已取消".into()), Some(duration));
        write_history(&app, &task_id, &workdir, &model, "cancelled", duration, Some("任务已取消".into()));
        return;
    }
    if let Some(msg) = failed {
        emit_status(&app, &task_id, "failed", Some(msg.clone()), Some(duration));
        write_history(&app, &task_id, &workdir, &model, "failed", duration, Some(msg));
        return;
    }
    let code = status.and_then(|s| s.code()).unwrap_or(-1);
    if code != 0 {
        let msg = format!("WorkBuddy CLI 退出码 {code}");
        emit_status(&app, &task_id, "failed", Some(msg.clone()), Some(duration));
        write_history(&app, &task_id, &workdir, &model, "failed", duration, Some(msg));
        return;
    }
    if saw_error {
        let msg = "WorkBuddy 返回错误，请查看输出".to_string();
        emit_status(&app, &task_id, "failed", Some(msg.clone()), Some(duration));
        write_history(&app, &task_id, &workdir, &model, "failed", duration, Some(msg));
        return;
    }

    let summary = collected.trim().to_string();
    let summary = if summary.chars().count() > 500 {
        let head: String = summary.chars().take(500).collect();
        format!("{head}…")
    } else {
        summary
    };
    emit_status(&app, &task_id, "finished", Some(summary.clone()), Some(duration));
    write_history(&app, &task_id, &workdir, &model, "success", duration, Some(summary));
}

/// 推送日志行。
fn emit_log(app: &AppHandle, task_id: &str, stream: &str, text: String) {
    let _ = app.emit(
        events::WORKBUDDY_LOG,
        WorkBuddyLogLine {
            task_id: task_id.to_string(),
            stream: stream.to_string(),
            text,
            ts: chrono::Utc::now().timestamp_millis(),
        },
    );
}

/// 推送任务状态。
fn emit_status(
    app: &AppHandle,
    task_id: &str,
    phase: &str,
    message: Option<String>,
    duration_ms: Option<i64>,
) {
    let _ = app.emit(
        events::WORKBUDDY_STATUS,
        WorkBuddyTaskStatus {
            task_id: task_id.to_string(),
            phase: phase.to_string(),
            message,
            duration_ms,
            ts: chrono::Utc::now().timestamp_millis(),
        },
    );
}

/// 写入历史任务。
fn write_history(
    app: &AppHandle,
    task_id: &str,
    workdir: &std::path::Path,
    model: &str,
    result: &str,
    duration_ms: i64,
    message: Option<String>,
) {
    let state = app.state::<AppState>();
    let Ok(db_guard) = state.db.lock() else {
        return;
    };
    let Some(db) = db_guard.as_ref() else {
        return;
    };
    let rec = TaskRecord {
        id: task_id.to_string(),
        ts: chrono::Utc::now().timestamp_millis(),
        kind: "workbuddy".to_string(),
        agent_id: "workbuddy-cli".to_string(),
        agent_name: "WorkBuddy CLI".to_string(),
        target: workdir.display().to_string(),
        source: model.to_string(),
        result: result.to_string(),
        duration_ms,
        message,
    };
    if let Err(e) = db.insert(&rec) {
        tracing::warn!("写入历史记录失败: {e}");
    }
}
