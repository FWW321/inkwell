use crate::db::models::WritingReview;
use crate::error::AppResult;
use crate::services::review_service;
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct AggregateReviewResult {
    pub dimensions: Vec<DimensionResult>,
    pub overall_score: f32,
    pub passed: bool,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct DimensionResult {
    pub dimension: String,
    pub score: f32,
    pub passed: bool,
    pub issues: Vec<serde_json::Value>,
    pub summary: String,
}

#[tauri::command]
pub async fn review_beat(
    state: State<'_, AppState>,
    session_id: String,
    beat_id: String,
    beat_content: String,
    beat_type: String,
) -> AppResult<AggregateReviewResult> {
    let result = review_service::review_beat(
        &state.db,
        &session_id,
        &beat_id,
        &beat_content,
        &beat_type,
    )
    .await?;

    Ok(AggregateReviewResult {
        dimensions: result
            .dimensions
            .into_iter()
            .map(|d| DimensionResult {
                dimension: d.dimension,
                score: d.score,
                passed: d.passed,
                issues: d.issues.iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect(),
                summary: d.summary,
            })
            .collect(),
        overall_score: result.overall_score,
        passed: result.passed,
    })
}

#[tauri::command]
pub async fn list_writing_reviews(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<Vec<WritingReview>> {
    review_service::list_reviews(&state.db, &session_id).await
}
