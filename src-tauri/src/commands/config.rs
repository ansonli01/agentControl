use tauri::AppHandle;

use crate::agent::model::AgentOverride;
use crate::config::mcp::McpDoc;
use crate::config::store::{self, AppConfig, Credential, Project};
use crate::config::{mcp, secrets, workspace};
use crate::error::{AppError, AppResult};

use super::app_state;

/// 读取全局配置。
#[tauri::command]
pub fn get_config(app: AppHandle) -> AppResult<AppConfig> {
    let state = app_state(&app)?;
    let cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?;
    Ok(cfg.clone())
}

/// 保存全局配置。
#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> AppResult<AppConfig> {
    store::save(&config)?;
    let state = app_state(&app)?;
    let mut guard = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?;
    *guard = config.clone();
    Ok(config)
}

/// 保存智能体自定义源（覆盖下载地址 / npm 包名等）。
#[tauri::command]
pub fn save_agent_override(app: AppHandle, item: AgentOverride) -> AppResult<AppConfig> {
    let state = app_state(&app)?;
    let mut cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    match cfg.agent_overrides.iter_mut().find(|o| o.agent_id == item.agent_id) {
        Some(existing) => {
            if item.official_download.is_some() {
                existing.official_download = item.official_download;
            }
            if item.official_page.is_some() {
                existing.official_page = item.official_page;
            }
            if item.npm_package.is_some() {
                existing.npm_package = item.npm_package;
            }
            if item.npm_registry.is_some() {
                existing.npm_registry = item.npm_registry;
            }
            if item.silent_args.is_some() {
                existing.silent_args = item.silent_args;
            }
            if item.enabled.is_some() {
                existing.enabled = item.enabled;
            }
        }
        None => cfg.agent_overrides.push(item),
    }
    store::save(&cfg)?;
    *state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))? = cfg.clone();
    Ok(cfg)
}
/// 新增项目。
#[tauri::command]
pub fn save_project(app: AppHandle, project: Project) -> AppResult<AppConfig> {
    let state = app_state(&app)?;
    let mut cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    if !project.workspace.is_empty() {
        workspace::validate_workspace(&project.workspace)?;
    }
    match cfg.projects.iter_mut().find(|p| p.id == project.id) {
        Some(existing) => *existing = project,
        None => cfg.projects.push(project),
    }
    store::save(&cfg)?;
    *state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))? = cfg.clone();
    Ok(cfg)
}

/// 删除项目。
#[tauri::command]
pub fn delete_project(app: AppHandle, project_id: String) -> AppResult<AppConfig> {
    let state = app_state(&app)?;
    let mut cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    cfg.projects.retain(|p| p.id != project_id);
    store::save(&cfg)?;
    *state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))? = cfg.clone();
    Ok(cfg)
}

/// 保存凭据：密钥写入系统凭据库，配置文件仅保存掩码。
#[tauri::command]
pub fn save_credential(
    app: AppHandle,
    provider: String,
    label: String,
    secret: String,
) -> AppResult<AppConfig> {
    if secret.trim().is_empty() {
        return Err(AppError::msg("密钥不能为空"));
    }
    let state = app_state(&app)?;
    let mut cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    let id = uuid::Uuid::new_v4().to_string();
    secrets::set_secret(&id, secret.trim())?;
    let masked = secrets::mask(secret.trim());
    cfg.credentials.retain(|c| c.provider != provider || c.label != label);
    cfg.credentials.push(Credential {
        id,
        provider,
        label,
        masked,
        created_at: chrono::Utc::now().timestamp_millis(),
    });
    store::save(&cfg)?;
    *state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))? = cfg.clone();
    Ok(cfg)
}
/// 删除凭据。
#[tauri::command]
pub fn delete_credential(app: AppHandle, credential_id: String) -> AppResult<AppConfig> {
    let state = app_state(&app)?;
    let mut cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    cfg.credentials.retain(|c| c.id != credential_id);
    let _ = secrets::delete_secret(&credential_id);
    store::save(&cfg)?;
    *state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))? = cfg.clone();
    Ok(cfg)
}

/// 读取某个智能体的 MCP 配置。
#[tauri::command]
pub fn load_mcp(app: AppHandle, agent_id: String) -> AppResult<McpDoc> {
    let state = app_state(&app)?;
    let cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    let def = crate::agent::definition(&cfg, &agent_id)?;
    let template = def
        .mcp_config_path
        .clone()
        .ok_or_else(|| AppError::msg(format!("{} 未提供 MCP 配置文件", def.name)))?;
    if def.mcp_config_format == Some(crate::agent::model::ConfigFormat::Toml) {
        return Err(AppError::msg(format!(
            "{} 的配置为 TOML 格式，当前仅支持 JSON 格式的 MCP 编辑",
            def.name
        )));
    }
    let path = mcp::expand_path(&template);
    mcp::read_mcp(&path)
}

/// 保存某个智能体的 MCP 配置。
#[tauri::command]
pub fn save_mcp(app: AppHandle, agent_id: String, content: String) -> AppResult<McpDoc> {
    let state = app_state(&app)?;
    let cfg = state
        .config
        .lock()
        .map_err(|e| AppError::msg(e.to_string()))?
        .clone();
    let def = crate::agent::definition(&cfg, &agent_id)?;
    let template = def
        .mcp_config_path
        .clone()
        .ok_or_else(|| AppError::msg(format!("{} 未提供 MCP 配置文件", def.name)))?;
    let path = mcp::expand_path(&template);
    mcp::save_mcp(&path, &content)?;
    mcp::read_mcp(&path)
}

/// 可选工作区根目录。
#[tauri::command]
pub fn list_workspace_roots() -> Vec<String> {
    workspace::list_roots()
}