use super::model::*;

/// 内置智能体清单：新增一个智能体只需在此追加一条定义。
pub fn builtin_agents() -> Vec<AgentDefinition> {
    vec![
        AgentDefinition {
            id: "opencode".into(),
            name: "OpenCode".into(),
            vendor: "SST".into(),
            kind: AgentKind::Cli,
            description: "终端优先的开源编程智能体，支持多模型与本地模型接入。".into(),
            tags: vec!["开源".into(), "CLI".into(), "多模型".into()],
            cli_names: vec!["opencode".into()],
            version_args: vec!["--version".into()],
            npm_package: Some("opencode-ai".into()),
            official_download: None,
            official_page: Some("https://opencode.ai".into()),
            open_platform: None,
            silent_args: vec![],
            mcp_config_path: Some("~/.config/opencode/opencode.json".into()),
            mcp_config_format: Some(ConfigFormat::Json),
            config_path: Some("~/.config/opencode/opencode.json".into()),
            credential_provider: None,
            icon_letter: "O".into(),
            accent: "#6A93FF".into(),
            install_method: InstallMethod::Npm,
            builtin: true,
        },        AgentDefinition {
            id: "codex".into(),
            name: "Codex CLI".into(),
            vendor: "OpenAI".into(),
            kind: AgentKind::Cli,
            description: "OpenAI 官方终端编程智能体，读取项目上下文并执行多步任务。".into(),
            tags: vec!["CLI".into(), "OpenAI".into()],
            cli_names: vec!["codex".into()],
            version_args: vec!["--version".into()],
            npm_package: Some("@openai/codex".into()),
            official_download: None,
            official_page: Some("https://developers.openai.com/codex/cli".into()),
            open_platform: Some("https://platform.openai.com/api-keys".into()),
            silent_args: vec![],
            mcp_config_path: Some("~/.codex/config.toml".into()),
            mcp_config_format: Some(ConfigFormat::Toml),
            config_path: Some("~/.codex/config.toml".into()),
            credential_provider: Some("openai".into()),
            icon_letter: "X".into(),
            accent: "#3FBF7F".into(),
            install_method: InstallMethod::Npm,
            builtin: true,
        },
        AgentDefinition {
            id: "claude-code".into(),
            name: "Claude Code".into(),
            vendor: "Anthropic".into(),
            kind: AgentKind::Cli,
            description: "Anthropic 官方命令行编码智能体，支持 MCP 扩展与脚本编排。".into(),
            tags: vec!["CLI".into(), "Anthropic".into()],
            cli_names: vec!["claude".into()],
            version_args: vec!["--version".into()],
            npm_package: Some("@anthropic-ai/claude-code".into()),
            official_download: None,
            official_page: Some("https://docs.anthropic.com/en/docs/claude-code/overview".into()),
            open_platform: Some("https://console.anthropic.com/settings/keys".into()),
            silent_args: vec![],
            mcp_config_path: Some("~/.claude.json".into()),
            mcp_config_format: Some(ConfigFormat::Json),
            config_path: Some("~/.claude.json".into()),
            credential_provider: Some("anthropic".into()),
            icon_letter: "C".into(),
            accent: "#F2994A".into(),
            install_method: InstallMethod::Npm,
            builtin: true,
        },        AgentDefinition {
            id: "gemini-cli".into(),
            name: "Gemini CLI".into(),
            vendor: "Google".into(),
            kind: AgentKind::Cli,
            description: "Google 开源终端智能体，接入 Gemini 大模型并支持扩展系统。".into(),
            tags: vec!["CLI".into(), "开源".into()],
            cli_names: vec!["gemini".into()],
            version_args: vec!["--version".into()],
            npm_package: Some("@google/gemini-cli".into()),
            official_download: None,
            official_page: Some("https://github.com/google-gemini/gemini-cli".into()),
            open_platform: Some("https://aistudio.google.com/app/apikey".into()),
            silent_args: vec![],
            mcp_config_path: Some("~/.gemini/settings.json".into()),
            mcp_config_format: Some(ConfigFormat::Json),
            config_path: Some("~/.gemini/settings.json".into()),
            credential_provider: Some("google".into()),
            icon_letter: "G".into(),
            accent: "#8B5CF6".into(),
            install_method: InstallMethod::Npm,
            builtin: true,
        },
        AgentDefinition {
            id: "qwen-code".into(),
            name: "Qwen Code".into(),
            vendor: "阿里云".into(),
            kind: AgentKind::Cli,
            description: "通义千问开源终端编码智能体，支持 Qwen 系列模型与 MCP。".into(),
            tags: vec!["CLI".into(), "开源".into()],
            cli_names: vec!["qwen".into()],
            version_args: vec!["--version".into()],
            npm_package: Some("@qwen-code/qwen-code".into()),
            official_download: None,
            official_page: Some("https://github.com/QwenLM/qwen-code".into()),
            open_platform: Some("https://bailian.console.aliyun.com".into()),
            silent_args: vec![],
            mcp_config_path: Some("~/.qwen/settings.json".into()),
            mcp_config_format: Some(ConfigFormat::Json),
            config_path: Some("~/.qwen/settings.json".into()),
            credential_provider: Some("dashscope".into()),
            icon_letter: "Q".into(),
            accent: "#4F7CFF".into(),
            install_method: InstallMethod::Npm,
            builtin: true,
        },        AgentDefinition {
            id: "workbuddy".into(),
            name: "WorkBuddy".into(),
            vendor: "腾讯".into(),
            kind: AgentKind::Desktop,
            description: "腾讯桌面 AI Agent 办公工作台，可规划并执行资料、文档与数据分析任务。官方未提供 CLI，需下载桌面客户端。".into(),
            tags: vec!["桌面端".into(), "办公".into(), "腾讯".into()],
            cli_names: vec!["workbuddy".into()],
            version_args: vec!["--version".into()],
            npm_package: None,
            official_download: None,
            official_page: Some("https://www.workbuddy.cn".into()),
            open_platform: Some("https://open.workbuddy.cn".into()),
            silent_args: vec!["/S".into()],
            mcp_config_path: None,
            mcp_config_format: None,
            config_path: None,
            credential_provider: Some("workbuddy".into()),
            icon_letter: "W".into(),
            accent: "#6A93FF".into(),
            install_method: InstallMethod::Official,
            builtin: true,
        },
        AgentDefinition {
            id: "doubao".into(),
            name: "豆包".into(),
            vendor: "字节跳动".into(),
            kind: AgentKind::Desktop,
            description: "字节跳动的 AI 助手桌面客户端，可配合火山方舟 API 使用。官方未提供 CLI，需下载桌面客户端。".into(),
            tags: vec!["桌面端".into(), "字节".into()],
            cli_names: vec!["doubao".into()],
            version_args: vec!["--version".into()],
            npm_package: None,
            official_download: None,
            official_page: Some("https://www.doubao.com".into()),
            open_platform: Some("https://www.volcengine.com/product/ark".into()),
            silent_args: vec![],
            mcp_config_path: None,
            mcp_config_format: None,
            config_path: None,
            credential_provider: Some("ark".into()),
            icon_letter: "D".into(),
            accent: "#E5534B".into(),
            install_method: InstallMethod::Official,
            builtin: true,
        },        AgentDefinition {
            id: "opencode-desktop".into(),
            name: "OpenCode 桌面版".into(),
            vendor: "SST".into(),
            kind: AgentKind::Desktop,
            description: "OpenCode 官方桌面客户端，图形化会话与工具编排。".into(),
            tags: vec!["桌面端".into(), "开源".into()],
            cli_names: vec!["opencode desktop".into()],
            version_args: vec!["--version".into()],
            npm_package: None,
            official_download: None,
            official_page: Some("https://opencode.ai/download".into()),
            open_platform: None,
            silent_args: vec![],
            mcp_config_path: None,
            mcp_config_format: None,
            config_path: None,
            credential_provider: None,
            icon_letter: "OD".into(),
            accent: "#6A93FF".into(),
            install_method: InstallMethod::Manual,
            builtin: true,
        },
        AgentDefinition {
            id: "codebuddy".into(),
            name: "CodeBuddy".into(),
            vendor: "腾讯".into(),
            kind: AgentKind::Desktop,
            description: "CodeBuddy IDE 扩展与命令行工具，含技能市场与 MCP 能力。".into(),
            tags: vec!["IDE".into(), "腾讯".into()],
            cli_names: vec!["codebuddy".into()],
            version_args: vec!["--version".into()],
            npm_package: None,
            official_download: None,
            official_page: Some("https://www.codebuddy.ai".into()),
            open_platform: None,
            silent_args: vec![],
            mcp_config_path: Some("~/.codebuddy/mcp.json".into()),
            mcp_config_format: Some(ConfigFormat::Json),
            config_path: Some("~/.codebuddy/models.json".into()),
            credential_provider: None,
            icon_letter: "CB".into(),
            accent: "#3A66E8".into(),
            install_method: InstallMethod::Manual,
            builtin: true,
        },        AgentDefinition {
            id: "trae".into(),
            name: "TRAE".into(),
            vendor: "字节跳动".into(),
            kind: AgentKind::Desktop,
            description: "字节跳动的 AI IDE，内置 Builder 与 SOLO 模式。仅支持官方安装包。".into(),
            tags: vec!["IDE".into(), "字节".into()],
            cli_names: vec!["trae".into()],
            version_args: vec!["--version".into()],
            npm_package: None,
            official_download: None,
            official_page: Some("https://www.trae.cn".into()),
            open_platform: None,
            silent_args: vec!["/S".into()],
            mcp_config_path: None,
            mcp_config_format: None,
            config_path: None,
            credential_provider: None,
            icon_letter: "T".into(),
            accent: "#8B5CF6".into(),
            install_method: InstallMethod::Official,
            builtin: true,
        },
        AgentDefinition {
            id: "cursor".into(),
            name: "Cursor".into(),
            vendor: "Anysphere".into(),
            kind: AgentKind::Desktop,
            description: "AI 原生代码编辑器，Agent 模式基于自有模型。".into(),
            tags: vec!["IDE".into()],
            cli_names: vec!["cursor".into()],
            version_args: vec!["--version".into()],
            npm_package: None,
            official_download: None,
            official_page: Some("https://www.cursor.com/downloads".into()),
            open_platform: None,
            silent_args: vec![],
            mcp_config_path: None,
            mcp_config_format: None,
            config_path: None,
            credential_provider: None,
            icon_letter: "Cu".into(),
            accent: "#A6B0BF".into(),
            install_method: InstallMethod::Manual,
            builtin: true,
        },
    ]
}
/// 凭据服务商标识（API Key 类型）。
pub fn providers() -> Vec<Provider> {
    vec![
        Provider {
            id: "openai".into(),
            name: "OpenAI".into(),
            vendor: "OpenAI".into(),
            key_label: "sk-...".into(),
            base_url: Some("https://api.openai.com/v1".into()),
            docs: Some("https://platform.openai.com/api-keys".into()),
        },
        Provider {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            vendor: "Anthropic".into(),
            key_label: "sk-ant-...".into(),
            base_url: Some("https://api.anthropic.com".into()),
            docs: Some("https://console.anthropic.com/settings/keys".into()),
        },
        Provider {
            id: "google".into(),
            name: "Google AI Studio".into(),
            vendor: "Google".into(),
            key_label: "AIza...".into(),
            base_url: Some("https://generativelanguage.googleapis.com".into()),
            docs: Some("https://aistudio.google.com/app/apikey".into()),
        },
        Provider {
            id: "ark".into(),
            name: "火山方舟".into(),
            vendor: "字节跳动".into(),
            key_label: "ARK-...".into(),
            base_url: Some("https://ark.cn-beijing.volces.com/api/v3".into()),
            docs: Some("https://www.volcengine.com/product/ark".into()),
        },
        Provider {
            id: "dashscope".into(),
            name: "阿里百炼".into(),
            vendor: "阿里云".into(),
            key_label: "sk-...".into(),
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".into()),
            docs: Some("https://bailian.console.aliyun.com".into()),
        },
        Provider {
            id: "workbuddy".into(),
            name: "WorkBuddy 开放平台".into(),
            vendor: "腾讯".into(),
            key_label: "wb-...".into(),
            base_url: Some("https://open.workbuddy.cn".into()),
            docs: Some("https://open.workbuddy.cn".into()),
        },
    ]
}
/// 合并用户覆盖到内置清单。
pub fn merge_overrides(
    defs: Vec<AgentDefinition>,
    overrides: &[AgentOverride],
) -> Vec<AgentDefinition> {
    defs.into_iter()
        .map(|mut d| {
            if let Some(o) = overrides.iter().find(|o| o.agent_id == d.id) {
                if o.official_download.is_some() {
                    d.official_download = o.official_download.clone();
                }
                if o.official_page.is_some() {
                    d.official_page = o.official_page.clone();
                }
                if o.npm_package.is_some() {
                    d.npm_package = o.npm_package.clone();
                }
                if o.silent_args.is_some() {
                    d.silent_args = o.silent_args.clone().unwrap_or_default();
                }
            }
            d
        })
        .filter(|d| {
            overrides
                .iter()
                .find(|o| o.agent_id == d.id)
                .and_then(|o| o.enabled)
                .unwrap_or(true)
        })
        .collect()
}

/// 依据 id 查找清单条目。
pub fn find<'a>(defs: &'a [AgentDefinition], id: &str) -> Option<&'a AgentDefinition> {
    defs.iter().find(|d| d.id == id)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_contains_required_agents() {
        let defs = builtin_agents();
        for id in ["opencode", "workbuddy", "doubao", "codex", "claude-code"] {
            assert!(find(&defs, id).is_some(), "missing {id}");
        }
    }

    #[test]
    fn official_packages_have_no_npm_name() {
        let defs = builtin_agents();
        let wb = find(&defs, "workbuddy").unwrap();
        assert_eq!(wb.install_method, InstallMethod::Official);
        assert!(wb.npm_package.is_none());
    }

    #[test]
    fn overrides_replace_download_url() {
        let defs = builtin_agents();
        let merged = merge_overrides(
            defs,
            &[AgentOverride {
                agent_id: "workbuddy".into(),
                official_download: Some("https://example.com/wb.exe".into()),
                ..Default::default()
            }],
        );
        let wb = find(&merged, "workbuddy").unwrap();
        assert_eq!(wb.official_download.as_deref(), Some("https://example.com/wb.exe"));
    }

    #[test]
    fn providers_are_unique() {
        let list = providers();
        let mut ids: Vec<&str> = list.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }
}