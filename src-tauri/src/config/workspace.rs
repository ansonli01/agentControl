use std::path::Path;

use crate::error::{AppError, AppResult};

/// 校验工作区目录是否存在。
pub fn validate_workspace(path: &str) -> AppResult<()> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::msg(format!("工作区目录不存在：{path}")));
    }
    if !p.is_dir() {
        return Err(AppError::msg(format!("工作区路径不是目录：{path}")));
    }
    Ok(())
}

/// 列出驱动器根目录，作为工作区选择的候选项。
pub fn list_roots() -> Vec<String> {
    let mut out = Vec::new();
    if let Some(home) = dirs::home_dir() {
        out.push(home.to_string_lossy().to_string());
    }
    for letter in b'A'..=b'Z' {
        let root = format!("{}:\\", letter as char);
        if Path::new(&root).exists() {
            out.push(root);
        }
    }
    out
}