use crate::claude::agents::{
    delete_agent as do_delete, list_agents as do_list, list_templates, read_agent,
    save_agent as do_save, Agent, Template,
};

#[tauri::command]
pub fn list_agents() -> Result<Vec<Agent>, String> {
    do_list().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_agent(name: String) -> Result<Agent, String> {
    let path = crate::claude::agents::get_agent_path(&name);
    read_agent(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_agent(agent: Agent) -> Result<(), String> {
    do_save(&agent).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_agent(name: String) -> Result<(), String> {
    do_delete(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_agent_templates() -> Vec<Template> {
    list_templates()
}

#[tauri::command]
pub fn create_agent_from_template(template_name: String) -> Result<Agent, String> {
    let templates = list_templates();
    let template = templates
        .into_iter()
        .find(|t| t.name == template_name)
        .ok_or_else(|| format!("Template not found: {template_name}"))?;

    let agent = Agent {
        name: template.name.clone(),
        description: template.description.clone(),
        content: template.content.clone(),
        path: String::new(),
        frontmatter: crate::claude::agents::AgentFrontmatter {
            name: Some(template.name),
            description: Some(template.description),
            ..Default::default()
        },
    };

    do_save(&agent).map_err(|e| e.to_string())?;
    Ok(agent)
}
