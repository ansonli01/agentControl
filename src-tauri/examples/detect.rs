//! 命令行实机探测工具：`cargo run --example detect`
//!
//! 用与桌面端完全相同的探测链路扫描本机，输出每个智能体的安装状态，
//! 便于在没有 GUI 的环境下验证探测逻辑（PATH / npm 全局包 / 卸载注册表 / 安装目录）。

use std::time::Duration;

use agentcontrol_lib::agent::registry;
use agentcontrol_lib::detect;

#[tokio::main]
async fn main() {
    let defs = registry::builtin_agents();
    println!("正在探测 {} 个智能体 …\n", defs.len());

    let statuses = detect::detect_all(&defs, Duration::from_secs(20), None).await;

    for (def, status) in defs.iter().zip(statuses.iter()) {
        let mark = if status.installed { "√" } else { "×" };
        let version = status.version.clone().unwrap_or_else(|| {
            if status.installed {
                "版本未知".to_string()
            } else {
                "-".to_string()
            }
        });
        println!(
            "{mark} {:<18} {:<10} {:<14} {}",
            def.id,
            def.vendor,
            version,
            status
                .path
                .clone()
                .unwrap_or_else(|| "-".to_string())
        );
        for source in &status.sources {
            println!("      来源: {} {}", source.kind, source.detail.as_deref().unwrap_or(""));
        }
        if let Some(note) = &status.note {
            println!("      备注: {note}");
        }
    }

    let installed = statuses.iter().filter(|s| s.installed).count();
    println!("\n已安装 {installed} / {}", statuses.len());
}
