use crate::claude::history::{list_sessions, get_session_messages, ChatSession, ChatMessage};

#[tauri::command]
pub fn list_chat_sessions() -> Result<Vec<ChatSession>, String> {
    list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chat_messages(session_path: String) -> Result<Vec<ChatMessage>, String> {
    get_session_messages(&session_path).map_err(|e| e.to_string())
}
