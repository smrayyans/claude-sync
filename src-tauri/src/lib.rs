mod claude;
mod commands;
mod git;
mod sync;
mod tray;

use commands::{agents::*, git::*, memory::*, settings::*, sync::*};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            tray::setup_tray(app)?;

            // Initialize machine config on first run
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = sync::machine::ensure_machine_config().await {
                    log::error!("Failed to initialize machine config: {e}");
                }

                // Start auto-sync timer
                sync::engine::start_auto_sync(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Sync commands
            sync_now,
            sync_pull,
            sync_push,
            get_sync_status,
            get_pending_changes,
            // Agent commands
            list_agents,
            get_agent,
            save_agent,
            delete_agent,
            list_agent_templates,
            create_agent_from_template,
            // Memory commands
            list_memory_files,
            get_memory_file,
            save_memory_file,
            delete_memory_file,
            get_project_memories,
            // History & conflict commands
            get_commit_history,
            get_commit_diff,
            resolve_conflict,
            // Settings commands
            get_app_settings,
            save_app_settings,
            get_machine_config,
            save_machine_config,
            setup_remote,
            test_remote_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
