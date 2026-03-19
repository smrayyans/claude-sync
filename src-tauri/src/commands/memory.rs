use crate::claude::memory::{
    delete_memory_file as do_delete, get_memory_file as do_get,
    get_project_memories as do_get_project, list_memory_files as do_list,
    save_memory_file as do_save, MemoryFile,
};

#[tauri::command]
pub fn list_memory_files() -> Result<Vec<MemoryFile>, String> {
    do_list().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_memory_file(path: String) -> Result<MemoryFile, String> {
    do_get(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_memory_file(path: String, content: String) -> Result<(), String> {
    do_save(&path, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_memory_file(path: String) -> Result<(), String> {
    do_delete(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_memories(project_slug: String) -> Result<Vec<MemoryFile>, String> {
    do_get_project(&project_slug).map_err(|e| e.to_string())
}
