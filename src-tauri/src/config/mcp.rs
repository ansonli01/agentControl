use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// MCP 文档（读取结果）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDoc {
    pub path: String,
    pub exists: bool,
    pub content: String,
    pub valid: bool,
    pub error: Option<String>,
}

/// 展开路径模板：支持 ~/ 前缀与环境变量占位。
pub fn expand_path(template: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    if template == "~" {
        return home;
    }
    if let Some(rest) = template.strip_prefix("~/").or_else(|| template.strip_prefix("~\\")) {
        return home.join(rest);
    }
    let pct = char::from(37u8);
    let parts: Vec<&str> = template.split(pct).collect();
    if parts.len() < 3 {
        return PathBuf::from(template);
    }
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 1 && !part.is_empty() {
            out.push_str(&std::env::var(part).unwrap_or_default());
        } else {
            out.push_str(part);
        }
    }
    PathBuf::from(out)
}
/// 读取 MCP 配置文件内容。
pub fn read_mcp(path: &Path) -> AppResult<McpDoc> {
    if !path.exists() {
        return Ok(McpDoc {
            path: path.to_string_lossy().to_string(),
            exists: false,
            content: default_template(),
            valid: true,
            error: None,
        });
    }
    let content = std::fs::read_to_string(path)?;
    let (valid, error) = validate_json(&content);
    Ok(McpDoc {
        path: path.to_string_lossy().to_string(),
        exists: true,
        content,
        valid,
        error,
    })
}

/// 校验 JSON 是否可解析。
pub fn validate_json(content: &str) -> (bool, Option<String>) {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(_) => (true, None),
        Err(e) => (false, Some(format!("JSON 解析失败（第 {} 行）：{}", e.line(), e))),
    }
}

/// 保存 MCP 配置（先校验 JSON，再原子写入）。
pub fn save_mcp(path: &Path, content: &str) -> AppResult<()> {
    let (valid, error) = validate_json(content);
    if !valid {
        return Err(AppError::msg(error.unwrap_or_default()));
    }
    let pretty = serde_json::to_string_pretty(
        &serde_json::from_str::<serde_json::Value>(content)?,
    )?;
    crate::config::store::write_atomic(path, pretty.as_bytes())
}

/// 空配置模板。
pub fn default_template() -> String {
    "{\n  \"mcpServers\": {}\n}\n".to_string()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_prefix() {
        let home = dirs::home_dir().unwrap();
        let path = expand_path("~/mcp.json");
        assert_eq!(path, home.join("mcp.json"));
    }

    #[test]
    fn validate_json_detects_error() {
        let (ok, err) = validate_json("{ \"a\": 1 }");
        assert!(ok);
        assert!(err.is_none());
        let (ok2, err2) = validate_json("{ \"a\": ");
        assert!(!ok2);
        assert!(err2.is_some());
    }

    #[test]
    fn default_template_is_valid_json() {
        let (ok, _) = validate_json(&default_template());
        assert!(ok);
    }
}