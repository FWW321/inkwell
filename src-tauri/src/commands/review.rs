use crate::db::models::WritingReview;
use crate::error::AppResult;
use crate::services::review_service;
use crate::state::AppState;
use tauri::State;

pub type AggregateReviewResult = review_service::AggregateReview;
pub type DimensionResult = review_service::DimensionResult;

#[tauri::command]
pub async fn review_beat(
    state: State<'_, AppState>,
    session_id: String,
    beat_id: String,
    beat_content: String,
    beat_type: String,
    agent_id: Option<String>,
) -> AppResult<AggregateReviewResult> {
    review_service::review_beat(
        state.db(),
        state.http(),
        &session_id,
        &beat_id,
        &beat_content,
        &beat_type,
        agent_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn list_writing_reviews(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<Vec<WritingReview>> {
    review_service::list_reviews(state.db(), &session_id).await
}
