use std::path::{Path, PathBuf};

use futures_util::StreamExt;

use crate::error::{AppError, AppResult};
use crate::runtime::process::run_capture_timeout;

/// 下载结果。
pub struct DownloadResult {
    pub path: PathBuf,
    pub size: u64,
}

/// 默认下载目录：用户下载目录下的 AgentControl 子目录。
pub fn default_download_dir() -> PathBuf {
    let base = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(std::env::temp_dir);
    base.join("AgentControl")
}

/// 构建 HTTP 客户端（支持代理）。
pub fn build_client(proxy: Option<&str>) -> AppResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent("AgentControl/0.1")
        .redirect(reqwest::redirect::Policy::limited(10));
    if let Some(p) = proxy {
        let proxy = reqwest::Proxy::all(p)
            .map_err(|e| AppError::msg(format!("代理地址无效：{e}")))?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|e| AppError::msg(format!("创建 HTTP 客户端失败：{e}")))
}
/// 下载官方安装包到本地目录，返回文件路径与大小。
pub async fn download(
    url: &str,
    dest_dir: &Path,
    proxy: Option<&str>,
    mut on_progress: impl FnMut(u64, u64),
) -> AppResult<DownloadResult> {
    let client = build_client(proxy)?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::msg(format!("下载失败：{e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::msg(format!(
            "下载失败：HTTP {}",
            resp.status().as_u16()
        )));
    }
    let total = resp.content_length().unwrap_or(0);
    let file_name = url
        .split(['?', '#'])
        .next()
        .and_then(|s| s.rsplit('/').next())
        .filter(|s| !s.is_empty())
        .unwrap_or("agentcontrol-download.bin");
    std::fs::create_dir_all(dest_dir)?;
    let target = dest_dir.join(file_name);
    let mut file = tokio::fs::File::create(&target).await?;
    let mut stream = resp.bytes_stream();
    let mut received: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::msg(format!("下载中断：{e}")))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        received += chunk.len() as u64;
        on_progress(received, total);
    }
    tokio::io::AsyncWriteExt::flush(&mut file).await?;
    Ok(DownloadResult {
        path: target,
        size: received,
    })
}
/// 运行安装包（尝试静默参数），返回退出码。
pub async fn run_installer(path: &Path, silent_args: &[String]) -> AppResult<i32> {
    let args: Vec<String> = silent_args.to_vec();
    let out = run_capture_timeout(
        path.to_string_lossy().to_string().as_str(),
        &args,
        std::time::Duration::from_secs(900),
    )
    .await?;
    if !out.stdout.trim().is_empty() {
        tracing::info!("installer stdout: {}", out.stdout);
    }
    Ok(out.code.unwrap_or(-1))
}