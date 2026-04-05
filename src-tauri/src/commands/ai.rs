use tauri::ipc::Channel;
use tauri::State;

use crate::db::models::{AiAgent, AiConfig};
use crate::error::AppResult;
use crate::services::ai_service::{self, StreamChunk};
use crate::state::AppState;

macro_rules! get_config_from_state {
    ($state:expr) => {{
        let conn = $state
            .db
            .lock()
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        ai_service::get_default_config(&conn)?
    }};
}

macro_rules! get_conn {
    ($state:expr) => {{
        $state
            .db
            .lock()
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
    }};
}

#[tauri::command]
pub fn list_ai_models(state: State<AppState>) -> AppResult<Vec<AiConfig>> {
    let conn = get_conn!(state);
    ai_service::list_models(&conn)
}

#[tauri::command]
pub fn create_ai_model(
    state: State<AppState>,
    name: String,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<AiConfig> {
    let conn = get_conn!(state);
    ai_service::create_model(&conn, &name, &api_key, &model, &base_url)
}

#[tauri::command]
pub fn update_ai_model(
    state: State<AppState>,
    id: String,
    name: String,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<AiConfig> {
    let conn = get_conn!(state);
    ai_service::update_model(&conn, &id, &name, &api_key, &model, &base_url)
}

#[tauri::command]
pub fn delete_ai_model(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    ai_service::delete_model(&conn, &id)
}

#[tauri::command]
pub fn set_default_ai_model(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    ai_service::set_default_model(&conn, &id)
}

#[tauri::command]
pub fn list_ai_agents(state: State<AppState>) -> AppResult<Vec<AiAgent>> {
    let conn = get_conn!(state);
    ai_service::list_agents(&conn)
}

#[tauri::command]
pub fn create_ai_agent(
    state: State<AppState>,
    name: String,
    model_id: String,
    system_prompt: String,
) -> AppResult<AiAgent> {
    let conn = get_conn!(state);
    ai_service::create_agent(&conn, &name, &model_id, &system_prompt)
}

#[tauri::command]
pub fn update_ai_agent(
    state: State<AppState>,
    id: String,
    name: String,
    model_id: String,
    system_prompt: String,
) -> AppResult<AiAgent> {
    let conn = get_conn!(state);
    ai_service::update_agent(&conn, &id, &name, &model_id, &system_prompt)
}

#[tauri::command]
pub fn delete_ai_agent(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    ai_service::delete_agent(&conn, &id)
}

#[tauri::command]
pub fn set_default_ai_agent(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    ai_service::set_default_agent(&conn, &id)
}

#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
    api_key: Option<String>,
    base_url: Option<String>,
) -> AppResult<Vec<String>> {
    let key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => {
            let conn = get_conn!(state);
            let cfg = ai_service::get_default_config(&conn)?;
            cfg.api_key
        }
    };
    let url = match base_url {
        Some(u) if !u.is_empty() => u,
        _ => {
            let conn = get_conn!(state);
            let cfg = ai_service::get_default_config(&conn)?;
            cfg.base_url
        }
    };
    ai_service::fetch_available_models(&key, &url).await
}

#[tauri::command]
pub async fn ai_continue_writing(
    state: State<'_, AppState>,
    context: String,
    style: String,
    length: String,
) -> AppResult<String> {
    let config = get_config_from_state!(state);
    ai_service::continue_writing(&config, &context, &style, &length).await
}

#[tauri::command]
pub async fn ai_rewrite(
    state: State<'_, AppState>,
    selected_text: String,
    instruction: String,
) -> AppResult<String> {
    let config = get_config_from_state!(state);
    ai_service::rewrite(&config, &selected_text, &instruction).await
}

#[tauri::command]
pub async fn ai_polish(state: State<'_, AppState>, selected_text: String) -> AppResult<String> {
    let config = get_config_from_state!(state);
    ai_service::polish(&config, &selected_text).await
}

#[tauri::command]
pub async fn ai_generate_dialogue(
    state: State<'_, AppState>,
    characters: String,
    scenario: String,
) -> AppResult<String> {
    let config = get_config_from_state!(state);
    ai_service::generate_dialogue(&config, &characters, &scenario).await
}

#[tauri::command]
pub async fn ai_chat(
    state: State<'_, AppState>,
    project_id: String,
    context_type: String,
    context_id: String,
    message: String,
) -> AppResult<String> {
    let config = get_config_from_state!(state);
    ai_service::chat(&config, &project_id, &context_type, &context_id, &message).await
}

#[tauri::command]
pub async fn ai_stream(
    state: State<'_, AppState>,
    project_id: String,
    chapter_id: Option<String>,
    mode: String,
    text: String,
    style: Option<String>,
    length: Option<String>,
    on_chunk: Channel<StreamChunk>,
) -> AppResult<()> {
    let config = get_config_from_state!(state);

    if mode == "chat" {
        let conn = get_conn!(state);
        ai_service::save_chat_message(&conn, &project_id, "user", &text)?;
    }

    ai_service::ai_stream_with_context(
        &config,
        &project_id,
        chapter_id.as_deref(),
        &mode,
        &text,
        style.as_deref(),
        length.as_deref(),
        &on_chunk,
    )
    .await
}

#[tauri::command]
pub fn get_chat_history(
    state: State<AppState>,
    project_id: String,
) -> AppResult<Vec<ChatMessage>> {
    let conn = get_conn!(state);
    let rows = ai_service::get_chat_history(&conn, &project_id, 50)?;
    Ok(rows
        .into_iter()
        .map(|(role, content)| ChatMessage { role, content })
        .collect())
}

#[tauri::command]
pub fn clear_chat_history(
    state: State<AppState>,
    project_id: String,
) -> AppResult<()> {
    let conn = get_conn!(state);
    ai_service::clear_chat_history(&conn, &project_id)
}

#[derive(serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
