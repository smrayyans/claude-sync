use crate::claude::history::{list_sessions, get_session_messages, delete_session as do_delete, delete_sessions as do_delete_many, ChatSession, ChatMessage};

#[tauri::command]
pub fn list_chat_sessions() -> Result<Vec<ChatSession>, String> {
    list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chat_messages(session_path: String) -> Result<Vec<ChatMessage>, String> {
    get_session_messages(&session_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_chat_session(session_path: String, delete_from_sync: bool) -> Result<(), String> {
    do_delete(&session_path, delete_from_sync).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_chat_sessions(paths: Vec<String>, delete_from_sync: bool) -> Result<Vec<String>, String> {
    do_delete_many(paths, delete_from_sync).map_err(|e| e.to_string())
}
