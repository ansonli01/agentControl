use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

use crate::agent::model::{AgentDefinition, InstallMethod};
use crate::error::{AppError, AppResult};
use crate::events::{events, InstallLogLine, InstallPhase, InstallStatus};
use crate::install::{npm, official, InstallAction};
use crate::state::{InstallHandle, AppState};
use crate::tasks::TaskRecord;

use super::app_state;

/// 安装任务串行队列（npm 全局安装会争抢目录锁）。
static INSTALL_SEM: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

/// 启动安装/升级/卸载任务，返回任务 id。
#[tauri::command]
pub fn install_start(app: AppHandle, agent_id: String, action: String) -> AppResult<String> {
    let state = app_state(&app)?;
    let cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    let def = crate::agent::definition(&cfg, &agent_id)?;
    let act = InstallAction::parse(&action)
        .ok_or_else(|| AppError::msg(format!("未知操作：{action}")))?;
    let task_id = uuid::Uuid::new_v4().to_string();
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut handles = state
            .install_handles
            .lock()
            .map_err(|e| AppError::msg(e.to_string()))?;
        handles.insert(
            task_id.clone(),
            InstallHandle {
                agent_id: def.id.clone(),
                label: format!("{} {}", act.label(), def.name),
                cancelled: cancel.clone(),
            },
        );
    }
    let handle = app.clone();
    let tid = task_id.clone();
    tauri::async_runtime::spawn(async move {
        run_install(handle, def, act, tid, cancel).await;
    });
    Ok(task_id)
}
/// 取消安装任务。
#[tauri::command]
pub fn install_cancel(app: AppHandle, task_id: String) -> AppResult<()> {
    let state = app_state(&app)?;
    let handles = state
        .install_handles
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?;
    if let Some(h) = handles.get(&task_id) {
        h.cancelled.store(true, Ordering::Relaxed);
    }
    Ok(())
}

async fn run_install(
    app: AppHandle,
    def: AgentDefinition,
    action: InstallAction,
    task_id: String,
    cancel: Arc<AtomicBool>,
) {
    let started = chrono::Utc::now().timestamp_millis();
    emit_status(
        &app,
        &task_id,
        &def,
        action,
        InstallPhase::Queued,
        None,
        Some("任务已排队".into()),
    );

    let _permit = INSTALL_SEM.acquire().await.ok();

    let outcome: Result<String, AppError> = match def.install_method {
        InstallMethod::Npm => run_npm(&app, &def, action, &task_id, cancel.clone()).await,
        InstallMethod::Official => run_official(&app, &def, action, &task_id, cancel.clone()).await,
        InstallMethod::Manual => Err(AppError::msg(format!(
            "{} 未提供自动安装方式，请前往官网手动下载",
            def.name
        ))),
    };

    let duration = chrono::Utc::now().timestamp_millis() - started;
    let (result, message, phase) = match outcome {
        Ok(msg) => ("success".to_string(), Some(msg), InstallPhase::Finished),
        Err(e) => {
            let cancelled = cancel.load(Ordering::Relaxed);
            if cancelled {
                ("cancelled".to_string(), Some(e.to_string()), InstallPhase::Cancelled)
            } else {
                ("failed".to_string(), Some(e.to_string()), InstallPhase::Failed)
            }
        }
    };

    emit_status(&app, &task_id, &def, action, phase, None, message.clone());
    write_history(&app, &task_id, &def, action, &result, duration, message);
}
async fn run_npm(
    app: &AppHandle,
    def: &AgentDefinition,
    action: InstallAction,
    task_id: &str,
    cancel: Arc<AtomicBool>,
) -> AppResult<String> {
    let pkg = def
        .npm_package
        .clone()
        .ok_or_else(|| AppError::msg(format!("{} 未配置 npm 包名", def.name)))?;
    let registry = {
        let state = app.state::<AppState>();
        let cfg = state
            .config
            .lock()
            .map_err(|e| AppError::msg(e.to_string()))?;
        cfg.npm_registry.clone()
    };
    emit_status(
        app,
        task_id,
        def,
        action,
        InstallPhase::Resolving,
        None,
        Some(npm::npm_preview(action, &pkg, &registry)),
    );

    let (tx, mut rx) = mpsc::channel::<(String, String)>(128);
    let cmd = npm::npm_args(action, &pkg, &registry);
    let worker_cancel = cancel.clone();
    let worker = tauri::async_runtime::spawn(async move {
        crate::runtime::process::run_streaming("npm", &cmd, tx, worker_cancel).await
    });

    while let Some((stream, text)) = rx.recv().await {
        let _ = app.emit(
            events::INSTALL_LOG,
            InstallLogLine {
                task_id: task_id.to_string(),
                agent_id: def.id.clone(),
                stream,
                text,
                ts: chrono::Utc::now().timestamp_millis(),
            },
        );
    }

    let code = worker
        .await
        .map_err(|e| AppError::msg(format!("npm 任务异常：{e}")))?
        .map_err(|e| e)?;
    if code == 0 {
        Ok(format!("{} {} 完成", action.label(), pkg))
    } else {
        Err(AppError::msg(format!(
            "npm 退出码 {code}，请查看上方日志或手动执行：{}",
            npm::npm_preview(action, &pkg, &registry)
        )))
    }
}
async fn run_official(
    app: &AppHandle,
    def: &AgentDefinition,
    action: InstallAction,
    task_id: &str,
    cancel: Arc<AtomicBool>,
) -> AppResult<String> {
    if action != InstallAction::Install {
        return Err(AppError::msg(format!(
            "{} 为官方客户端，暂不支持自动{}",
            def.name,
            action.label()
        )));
    }
    let url = def.official_download.clone().ok_or_else(|| {
        AppError::msg(format!(
            "尚未配置 {} 的官方安装包地址，请在设置中填写直链，或前往官网手动下载",
            def.name
        ))
    })?;

    let (proxy, download_dir) = {
        let state = app.state::<AppState>();
        let cfg = state
            .config
            .lock()
            .map_err(|e| AppError::msg(e.to_string()))?;
        let dir = cfg
            .download_dir
            .clone()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(official::default_download_dir);
        (cfg.proxy.clone(), dir)
    };

    emit_status(
        app,
        task_id,
        def,
        action,
        InstallPhase::Downloading,
        Some(0.0),
        Some(url.clone()),
    );

    let app_for_progress = app.clone();
    let tid = task_id.to_string();
    let def_clone = def.clone();
    let result = official::download(&url, &download_dir, proxy.as_deref(), move |received, total| {
        let progress = if total > 0 {
            Some(received as f32 / total as f32)
        } else {
            None
        };
        emit_status(
            &app_for_progress,
            &tid,
            &def_clone,
            action,
            InstallPhase::Downloading,
            progress,
            Some(format!("已下载 {} 字节", received)),
        );
    })
    .await?;

    if cancel.load(Ordering::Relaxed) {
        return Err(AppError::msg("任务已取消"));
    }

    emit_status(
        app,
        task_id,
        def,
        action,
        InstallPhase::Verifying,
        Some(1.0),
        Some(format!("校验 {}", result.path.display())),
    );
    let hash = crate::install::checksum::sha256_file(&result.path)?;
    emit_log(app, task_id, &def.id, "stdout", format!("SHA-256: {hash}"));

    emit_status(
        app,
        task_id,
        def,
        action,
        InstallPhase::Installing,
        None,
        Some("正在执行安装程序".into()),
    );
    let code = official::run_installer(&result.path, &def.silent_args).await?;
    if code == 0 {
        Ok(format!("安装包已执行：{}", result.path.display()))
    } else {
        Err(AppError::msg(format!(
            "安装程序退出码 {code}，安装包已保留：{}，可手动双击安装",
            result.path.display()
        )))
    }
}
fn emit_status(
    app: &AppHandle,
    task_id: &str,
    def: &AgentDefinition,
    action: InstallAction,
    phase: InstallPhase,
    progress: Option<f32>,
    message: Option<String>,
) {
    let _ = app.emit(
        events::INSTALL_STATUS,
        InstallStatus {
            task_id: task_id.to_string(),
            agent_id: def.id.clone(),
            agent_name: def.name.clone(),
            action: action.label().to_string(),
            phase,
            progress,
            message,
            ts: chrono::Utc::now().timestamp_millis(),
        },
    );
}

fn emit_log(app: &AppHandle, task_id: &str, agent_id: &str, stream: &str, text: String) {
    let _ = app.emit(
        events::INSTALL_LOG,
        InstallLogLine {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            stream: stream.to_string(),
            text,
            ts: chrono::Utc::now().timestamp_millis(),
        },
    );
}

fn write_history(
    app: &AppHandle,
    task_id: &str,
    def: &AgentDefinition,
    action: InstallAction,
    result: &str,
    duration: i64,
    message: Option<String>,
) {
    let state = app.state::<AppState>();
    let Ok(db_guard) = state.db.lock() else {
        return;
    };
    let Some(db) = db_guard.as_ref() else {
        return;
    };
    let source = match def.install_method {
        InstallMethod::Npm => def.npm_package.clone().unwrap_or_default(),
        InstallMethod::Official => def.official_download.clone().unwrap_or_default(),
        InstallMethod::Manual => "manual".to_string(),
    };
    let rec = TaskRecord {
        id: task_id.to_string(),
        ts: chrono::Utc::now().timestamp_millis(),
        kind: match action {
            InstallAction::Install => "install",
            InstallAction::Upgrade => "upgrade",
            InstallAction::Uninstall => "uninstall",
        }
        .to_string(),
        agent_id: def.id.clone(),
        agent_name: def.name.clone(),
        target: def.name.clone(),
        source,
        result: result.to_string(),
        duration_ms: duration,
        message,
    };
    if let Err(e) = db.insert(&rec) {
        tracing::warn!("写入历史记录失败: {e}");
    }
}