/// 构造 npm 命令参数。
pub fn npm_args(action: super::InstallAction, package: &str, registry: &str) -> Vec<String> {
    let mut args = vec!["-g".to_string()];
    match action {
        super::InstallAction::Install => {
            args.push("install".into());
            args.push(package.into());
        }
        super::InstallAction::Upgrade => {
            args.push("update".into());
            args.push(package.into());
        }
        super::InstallAction::Uninstall => {
            args.push("uninstall".into());
            args.push(package.into());
        }
    }
    args.push("--registry".into());
    args.push(registry.to_string());
    args.push("--no-fund".into());
    args.push("--no-audit".into());
    args.push("--loglevel".into());
    args.push("info".into());
    args
}

/// 人类可读的命令预览。
pub fn npm_preview(action: super::InstallAction, package: &str, registry: &str) -> String {
    format!("npm {}", npm_args(action, package, registry).join(" "))
}
#[cfg(test)]
mod tests {
    use super::super::InstallAction;
    use super::*;

    #[test]
    fn install_args_use_mirror_and_package() {
        let args = npm_args(InstallAction::Install, "opencode-ai", "https://registry.npmmirror.com");
        assert!(args.contains(&"opencode-ai".to_string()));
        assert!(args.contains(&"--registry".to_string()));
        assert!(args.contains(&"https://registry.npmmirror.com".to_string()));
    }

    #[test]
    fn upgrade_and_uninstall_differ() {
        let up = npm_args(InstallAction::Upgrade, "@openai/codex", "r");
        let un = npm_args(InstallAction::Uninstall, "@openai/codex", "r");
        assert_eq!(up[0], "-g");
        assert!(up.contains(&"update".to_string()));
        assert!(un.contains(&"uninstall".to_string()));
    }

    #[test]
    fn preview_contains_command() {
        let text = npm_preview(InstallAction::Install, "opencode-ai", "r");
        assert!(text.starts_with("npm "));
        assert!(text.contains("opencode-ai"));
    }
}