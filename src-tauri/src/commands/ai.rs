use tauri::ipc::Channel;
use tauri::State;

use crate::db::models::{AiAgentWithModelName, AiConfig};
use crate::error::AppResult;
use crate::services::ai_service::{self, StreamChunk};
use crate::state::AppState;

#[tauri::command]
pub async fn list_ai_models(state: State<'_, AppState>) -> AppResult<Vec<AiConfig>> {
    ai_service::list_models(&state.db).await
}

#[tauri::command]
pub async fn create_ai_model(
    state: State<'_, AppState>,
    name: String,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<AiConfig> {
    ai_service::create_model(&state.db, &name, &api_key, &model, &base_url).await
}

#[tauri::command]
pub async fn update_ai_model(
    state: State<'_, AppState>,
    id: String,
    name: String,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<AiConfig> {
    ai_service::update_model(&state.db, &id, &name, &api_key, &model, &base_url).await
}

#[tauri::command]
pub async fn delete_ai_model(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::delete_model(&state.db, &id).await
}

#[tauri::command]
pub async fn set_default_ai_model(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::set_default_model(&state.db, &id).await
}

#[tauri::command]
pub async fn list_ai_agents(state: State<'_, AppState>) -> AppResult<Vec<AiAgentWithModelName>> {
    ai_service::list_agents(&state.db).await
}

#[tauri::command]
pub async fn create_ai_agent(
    state: State<'_, AppState>,
    name: String,
    model_id: String,
    system_prompt: String,
) -> AppResult<AiAgentWithModelName> {
    ai_service::create_agent(&state.db, &name, &model_id, &system_prompt).await
}

#[tauri::command]
pub async fn update_ai_agent(
    state: State<'_, AppState>,
    id: String,
    name: String,
    model_id: String,
    system_prompt: String,
) -> AppResult<AiAgentWithModelName> {
    ai_service::update_agent(&state.db, &id, &name, &model_id, &system_prompt).await
}

#[tauri::command]
pub async fn delete_ai_agent(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::delete_agent(&state.db, &id).await
}

#[tauri::command]
pub async fn set_default_ai_agent(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::set_default_agent(&state.db, &id).await
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
            let cfg = ai_service::get_default_config(&state.db).await?;
            cfg.api_key
        }
    };
    let url = match base_url {
        Some(u) if !u.is_empty() => u,
        _ => {
            let cfg = ai_service::get_default_config(&state.db).await?;
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
    let config = ai_service::get_default_config(&state.db).await?;
    ai_service::continue_writing(&config, &context, &style, &length).await
}

#[tauri::command]
pub async fn ai_rewrite(
    state: State<'_, AppState>,
    selected_text: String,
    instruction: String,
) -> AppResult<String> {
    let config = ai_service::get_default_config(&state.db).await?;
    ai_service::rewrite(&config, &selected_text, &instruction).await
}

#[tauri::command]
pub async fn ai_polish(state: State<'_, AppState>, selected_text: String) -> AppResult<String> {
    let config = ai_service::get_default_config(&state.db).await?;
    ai_service::polish(&config, &selected_text).await
}

#[tauri::command]
pub async fn ai_generate_dialogue(
    state: State<'_, AppState>,
    characters: String,
    scenario: String,
) -> AppResult<String> {
    let config = ai_service::get_default_config(&state.db).await?;
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
    let config = ai_service::get_default_config(&state.db).await?;
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
    let config = ai_service::get_default_config(&state.db).await?;

    if mode == "chat" {
        ai_service::save_chat_message(&state.db, &project_id, "user", &text).await?;
    }

    ai_service::ai_stream_with_context(
        &state.db,
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
pub async fn get_chat_history(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<ChatMessage>> {
    let rows = ai_service::get_chat_history(&state.db, &project_id, 50).await?;
    Ok(rows
        .into_iter()
        .map(|(role, content)| ChatMessage { role, content })
        .collect())
}

#[tauri::command]
pub async fn clear_chat_history(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<()> {
    ai_service::clear_chat_history(&state.db, &project_id).await
}

#[derive(serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
