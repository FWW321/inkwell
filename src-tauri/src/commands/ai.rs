use tauri::State;

use crate::db::models::AiConfig;
use crate::error::AppResult;
use crate::services::ai_service;
use crate::state::AppState;

macro_rules! get_config_from_state {
    ($state:expr) => {{
        let conn = $state
            .db
            .lock()
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        ai_service::get_config(&conn)?
    }};
}

#[tauri::command]
pub fn get_ai_config(state: State<AppState>) -> AppResult<AiConfig> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    ai_service::get_config(&conn)
}

#[tauri::command]
pub fn set_ai_config(
    state: State<AppState>,
    api_key: String,
    model: String,
    base_url: String,
) -> AppResult<()> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    ai_service::set_config(&conn, &api_key, &model, &base_url)
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
            let conn = state
                .db
                .lock()
                .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
            let cfg = ai_service::get_config(&conn)?;
            cfg.api_key
        }
    };
    let url = match base_url {
        Some(u) if !u.is_empty() => u,
        _ => {
            let conn = state
                .db
                .lock()
                .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
            let cfg = ai_service::get_config(&conn)?;
            cfg.base_url
        }
    };
    ai_service::list_models(&key, &url).await
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
