use tauri::State;
use tauri::ipc::Channel;

use crate::db::models::{AiAgentWithModelName, AiConfigPublic};
use crate::error::AppResult;
use crate::services::agent_service;
use crate::services::ai_service::{self, StreamChunk};
use crate::state::AppState;

#[tauri::command]
pub async fn list_ai_models(state: State<'_, AppState>) -> AppResult<Vec<AiConfigPublic>> {
    let models = ai_service::list_models(state.db()).await?;
    Ok(models.into_iter().map(AiConfigPublic::from).collect())
}

#[tauri::command]
pub async fn create_ai_model(
    state: State<'_, AppState>,
    name: String,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<AiConfigPublic> {
    let cfg = ai_service::create_model(state.db(), &name, &api_key, &model, &base_url).await?;
    Ok(AiConfigPublic::from(cfg))
}

#[tauri::command]
pub async fn update_ai_model(
    state: State<'_, AppState>,
    id: String,
    name: String,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<AiConfigPublic> {
    let cfg = ai_service::update_model(state.db(), &id, &name, &api_key, &model, &base_url).await?;
    Ok(AiConfigPublic::from(cfg))
}

#[tauri::command]
pub async fn delete_ai_model(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::delete_model(state.db(), &id).await
}

#[tauri::command]
pub async fn set_default_ai_model(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::set_default_model(state.db(), &id).await
}

#[tauri::command]
pub async fn list_ai_agents(state: State<'_, AppState>) -> AppResult<Vec<AiAgentWithModelName>> {
    ai_service::list_agents(state.db()).await
}

#[tauri::command]
pub async fn create_ai_agent(
    state: State<'_, AppState>,
    name: String,
    model_id: Option<String>,
    system_prompt: String,
    temperature: Option<f32>,
) -> AppResult<AiAgentWithModelName> {
    ai_service::create_agent(
        state.db(),
        &name,
        model_id.as_deref(),
        &system_prompt,
        temperature.unwrap_or(0.8),
    )
    .await
}

#[tauri::command]
pub async fn update_ai_agent(
    state: State<'_, AppState>,
    id: String,
    name: String,
    model_id: Option<String>,
    system_prompt: String,
    temperature: Option<f32>,
) -> AppResult<AiAgentWithModelName> {
    ai_service::update_agent(
        state.db(),
        &id,
        &name,
        model_id.as_deref(),
        &system_prompt,
        temperature.unwrap_or(0.8),
    )
    .await
}

#[tauri::command]
pub async fn delete_ai_agent(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::delete_agent(state.db(), &id).await
}

#[tauri::command]
pub async fn set_default_ai_agent(state: State<'_, AppState>, id: String) -> AppResult<()> {
    ai_service::set_default_agent(state.db(), &id).await
}

#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
    api_key: Option<String>,
    base_url: Option<String>,
) -> AppResult<Vec<String>> {
    let cfg = ai_service::get_default_config(state.db()).await?;
    let key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => cfg.api_key,
    };
    let url = match base_url {
        Some(u) if !u.is_empty() => u,
        _ => cfg.base_url,
    };
    ai_service::fetch_available_models(state.http(), &key, &url).await
}

#[tauri::command]
pub async fn ai_invoke(
    state: State<'_, AppState>,
    agent_id: String,
    user_text: String,
) -> AppResult<String> {
    let agent_config = agent_service::get_agent_config(state.db(), &agent_id).await?;
    ai_service::chat_completion(
        state.http(),
        &agent_config.model_config,
        &agent_config.system_prompt,
        &user_text,
        agent_config.temperature,
    )
    .await
}

#[tauri::command]
pub async fn ai_invoke_with_context(
    state: State<'_, AppState>,
    agent_id: String,
    project_id: String,
    user_text: String,
    on_chunk: Channel<StreamChunk>,
) -> AppResult<()> {
    let agent_config = agent_service::get_agent_config(state.db(), &agent_id).await?;
    let contract =
        crate::services::context_service::build_contract(state.db(), &project_id, "").await?;
    let context = crate::services::context_service::build_editor_context(&contract);

    let preamble = format!("{}\n\n{}", agent_config.system_prompt, context);

    ai_service::stream_ai(
        state.http(),
        &agent_config.model_config,
        &preamble,
        &user_text,
        agent_config.temperature,
        &on_chunk,
    )
    .await
}

#[tauri::command]
pub async fn ai_stream(
    state: State<'_, AppState>,
    project_id: String,
    agent_id: String,
    save_history: bool,
    text: String,
    on_chunk: Channel<StreamChunk>,
) -> AppResult<()> {
    if save_history {
        ai_service::save_chat_message(state.db(), &project_id, "user", &text).await?;
    }

    let agent_config = agent_service::get_agent_config(state.db(), &agent_id).await?;
    let contract =
        crate::services::context_service::build_contract(state.db(), &project_id, "").await?;
    let context = crate::services::context_service::build_editor_context(&contract);
    let preamble = format!("{}\n\n{}", agent_config.system_prompt, context);

    let mut full_text = String::new();
    ai_service::stream_ai_with_callback(
        state.http(),
        &agent_config.model_config,
        &preamble,
        &text,
        agent_config.temperature,
        |chunk| {
            if !chunk.text.is_empty() {
                full_text.push_str(&chunk.text);
            }
            let _ = on_chunk.send(chunk);
        },
    )
    .await?;

    if save_history {
        ai_service::save_chat_message(state.db(), &project_id, "assistant", &full_text).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_chat_history(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<ChatMessage>> {
    let rows = ai_service::get_chat_history(state.db(), &project_id, 50).await?;
    Ok(rows
        .into_iter()
        .map(|(role, content)| ChatMessage { role, content })
        .collect())
}

#[tauri::command]
pub async fn clear_chat_history(state: State<'_, AppState>, project_id: String) -> AppResult<()> {
    ai_service::clear_chat_history(state.db(), &project_id).await
}

#[derive(serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
