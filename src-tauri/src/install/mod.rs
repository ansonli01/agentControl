pub mod checksum;
pub mod npm;
pub mod official;

/// 安装动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallAction {
    Install,
    Upgrade,
    Uninstall,
}

impl InstallAction {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "install" => Some(InstallAction::Install),
            "upgrade" => Some(InstallAction::Upgrade),
            "uninstall" => Some(InstallAction::Uninstall),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            InstallAction::Install => "安装",
            InstallAction::Upgrade => "升级",
            InstallAction::Uninstall => "卸载",
        }
    }
}