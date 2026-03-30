mod claude;
mod commands;
mod git;
mod sync;
mod tray;

use commands::{agents::*, git::*, history::*, memory::*, settings::*, sync::*};
use tauri::Manager;

/// Global mutex preventing concurrent sync/push/pull operations
pub struct SyncLock(pub tokio::sync::Mutex<()>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .manage(SyncLock(tokio::sync::Mutex::new(())))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            tray::setup_tray(app)?;

            // Initialize machine config on first run
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match sync::machine::ensure_machine_config().await {
                    Ok(config) => claude::apply_custom_paths(&config),
                    Err(e) => log::error!("Failed to initialize machine config: {e}"),
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
            sync_push_selective,
            get_file_preview,
            get_pull_log,
            get_sync_status,
            get_pending_changes,
            check_repo_status,
            diagnose_push,
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
            // Chat history commands
            list_chat_sessions,
            get_chat_messages,
            delete_chat_session,
            delete_chat_sessions,
            // Sync history & conflict commands
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
            check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
