pub mod agent;
mod commands;
pub mod config;
pub mod detect;
mod error;
pub mod events;
pub mod install;
pub mod runtime;
mod state;
pub mod tasks;

use tauri::Manager;

/// 应用入口。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("AGENTCONTROL_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let cfg = config::store::load().unwrap_or_default();
            let handle = app.handle().clone();
            let state = state::AppState::new(cfg);
            app.manage(state);
            let managed = handle.state::<state::AppState>();
            managed.set_handle(handle.clone());
            if let Err(e) = managed.init_db() {
                tracing::warn!("初始化历史数据库失败: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent::list_agents,
            commands::agent::list_providers,
            commands::agent_credentials::agent_credentials,
            commands::detect::detect_all,
            commands::install::install_start,
            commands::install::install_cancel,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::save_agent_override,
            commands::config::save_project,
            commands::config::delete_project,
            commands::config::save_credential,
            commands::config::delete_credential,
            commands::config::load_mcp,
            commands::config::save_mcp,
            commands::config::list_workspace_roots,
            commands::tasks::list_tasks,
            commands::tasks::clear_tasks,
            commands::workbuddy::workbuddy_status,
            commands::workbuddy::workbuddy_overview,
            commands::workbuddy::workbuddy_paths,
            commands::workbuddy::workbuddy_open,
            commands::workbuddy::workbuddy_account,
            commands::workbuddy_cli::workbuddy_cli_info,
            commands::workbuddy_cli::workbuddy_cli_run,
            commands::workbuddy_cli::workbuddy_cli_models,
            commands::workbuddy_dispatch::workbuddy_dispatch_start,
            commands::workbuddy_dispatch::workbuddy_dispatch_cancel,
            commands::workbuddy_data::workbuddy_sessions,
            commands::workbuddy_data::workbuddy_credentials,
            commands::workbuddy_catalog::workbuddy_catalog,
            commands::platform_catalogs::platform_catalogs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}