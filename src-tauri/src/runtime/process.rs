use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::error::{AppError, AppResult};

/// 命令执行结果。
#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CmdOutput {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

/// 在 PATH 与常见附加目录中查找可执行文件。
pub fn which(name: &str) -> Option<PathBuf> {
    if name.contains(' ') {
        return None;
    }
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        vec![".cmd".into(), ".bat".into(), ".exe".into(), ".ps1".into(), "".into()]
    } else {
        vec!["".into()]
    };
    for dir in std::env::split_paths(&path) {
        for ext in &exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 构造可执行命令：Windows 下 `.cmd/.bat` 通过 `cmd /C` 调用。
fn build_command(program: &str, args: &[String]) -> Command {
    let path = which(program);
    let needs_shell = path
        .as_ref()
        .and_then(|p| p.extension().map(|e| e.to_string_lossy().to_lowercase()))
        .map(|e| e == "cmd" || e == "bat")
        .unwrap_or(false);

    let mut cmd = if needs_shell {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(program);
        for a in args {
            c.arg(a);
        }
        c
    } else {
        let mut c = Command::new(path.as_deref().unwrap_or(Path::new(program)));
        c.args(args);
        c
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    cmd
}

/// 执行命令并收集输出（带超时）。
pub async fn run_capture_timeout(
    program: &str,
    args: &[String],
    timeout: Duration,
) -> AppResult<CmdOutput> {
    let mut cmd = build_command(program, args);
    let child = cmd
        .spawn()
        .map_err(|e| AppError::msg(format!("无法启动 {program}：{e}")))?;
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(res) => {
            let out = res.map_err(|e| AppError::msg(format!("读取 {program} 输出失败：{e}")))?;
            Ok(CmdOutput {
                code: out.status.code(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            })
        }
        Err(_) => Err(AppError::msg(format!(
            "{program} 执行超时（{timeout:?}）"
        ))),
    }
}

/// 流式执行命令：stdout/stderr 按行推送到 channel，返回退出码。
pub async fn run_streaming(
    program: &str,
    args: &[String],
    tx: Sender<(String, String)>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> AppResult<i32> {
    let mut cmd = build_command(program, args);
    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::msg(format!("无法启动 {program}：{e}")))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let mut tasks = Vec::new();
    if let Some(out) = stdout {
        let tx = tx.clone();
        tasks.push(tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(("stdout".into(), line)).await.is_err() {
                    break;
                }
            }
        }));
    }
    if let Some(err) = stderr {
        let tx = tx.clone();
        tasks.push(tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(("stderr".into(), line)).await.is_err() {
                    break;
                }
            }
        }));
    }

    let status = loop {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = child.kill().await;
            let _ = child.wait().await;
            for t in tasks {
                t.abort();
            }
            return Ok(-1);
        }
        match tokio::time::timeout(Duration::from_millis(200), child.wait()).await {
            Ok(Ok(status)) => break status.code().unwrap_or(-1),
            Ok(Err(e)) => {
                for t in tasks {
                    t.abort();
                }
                return Err(AppError::msg(format!("等待 {program} 失败：{e}")));
            }
            Err(_) => continue,
        }
    };

    for t in tasks {
        let _ = t.await;
    }
    Ok(status)
}
