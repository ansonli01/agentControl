use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::AppResult;

/// 计算文件的 SHA-256。
pub fn sha256_file(path: &Path) -> AppResult<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// 计算字节数组的 SHA-256。
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}