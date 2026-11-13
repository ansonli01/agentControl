pub mod model;
pub mod registry;

use crate::config::store::AppConfig;
use crate::error::{AppError, AppResult};
use model::AgentDefinition;

/// 结合用户覆盖生成完整清单。
pub fn definitions(cfg: &AppConfig) -> Vec<AgentDefinition> {
    registry::merge_overrides(registry::builtin_agents(), &cfg.agent_overrides)
}

/// 按 id 取清单条目。
pub fn definition(cfg: &AppConfig, id: &str) -> AppResult<AgentDefinition> {
    definitions(cfg)
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError::AgentNotFound(id.to_string()))
}