use crate::serde_helpers::{opt_rid, rid};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct Project {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub title: String,
    pub description: String,
    pub author: String,
    pub language: String,
    pub tags: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct OutlineNode {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    #[serde(
        rename = "parent_id",
        with = "opt_rid",
        skip_serializing_if = "Option::is_none"
    )]
    pub parent: Option<RecordId>,
    pub node_type: String,
    pub title: String,
    pub sort_order: i64,
    pub content_json: serde_json::Value,
    pub word_count: i64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_new: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_mode: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct Character {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub description: String,
    pub personality: String,
    pub background: String,
    pub race: String,
    #[serde(
        rename = "model_id",
        with = "opt_rid",
        skip_serializing_if = "Option::is_none"
    )]
    pub model: Option<RecordId>,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct CharacterWithModelName {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub description: String,
    pub personality: String,
    pub background: String,
    pub race: String,
    #[serde(
        rename = "model_id",
        with = "opt_rid",
        skip_serializing_if = "Option::is_none"
    )]
    pub model: Option<RecordId>,
    pub created_at: Datetime,
    #[serde(rename = "model_id")]
    pub model_id: Option<String>,
    #[serde(rename = "model_name")]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct Faction {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub name: String,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct WorldviewEntry {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub category: String,
    pub title: String,
    pub content: String,
    pub created_at: Datetime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Datetime>,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct AiConfig {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub name: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub is_default: bool,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AiConfigPublic {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub name: String,
    pub model: String,
    pub base_url: String,
    pub is_default: bool,
    pub created_at: Datetime,
}

impl From<AiConfig> for AiConfigPublic {
    fn from(c: AiConfig) -> Self {
        Self {
            id: c.id,
            name: c.name,
            model: c.model,
            base_url: c.base_url,
            is_default: c.is_default,
            created_at: c.created_at,
        }
    }
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct AiAgent {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub name: String,
    #[serde(
        rename = "model_id",
        with = "opt_rid",
        skip_serializing_if = "Option::is_none"
    )]
    pub model: Option<RecordId>,
    pub system_prompt: String,
    pub temperature: f32,
    pub is_default: bool,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct AiAgentWithModelName {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub name: String,
    #[serde(rename = "model_id")]
    pub model_id: Option<String>,
    pub system_prompt: String,
    pub temperature: f32,
    pub is_default: bool,
    pub created_at: Datetime,
    #[serde(rename = "model_name")]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct NarrativeSession {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub title: String,
    pub scene: String,
    pub atmosphere: String,
    pub timeline_id: String,
    pub strand: String,
    pub status: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct NarrativeBeat {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "session_id", with = "rid")]
    pub session: RecordId,
    pub beat_type: String,
    #[serde(
        rename = "character_id",
        with = "opt_rid",
        skip_serializing_if = "Option::is_none"
    )]
    pub character: Option<RecordId>,
    pub character_name: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub sort_order: i64,
    pub timeline_id: String,
    pub strand: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_strength: Option<String>,
    pub micro_payoffs: serde_json::Value,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct NarrativeEvent {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "beat_id", with = "rid")]
    pub beat: RecordId,
    #[serde(rename = "session_id", with = "rid")]
    pub session: RecordId,
    pub event_type: String,
    #[serde(
        rename = "character_id",
        with = "opt_rid",
        skip_serializing_if = "Option::is_none"
    )]
    pub character: Option<RecordId>,
    pub character_name: String,
    pub summary: String,
    pub detail: serde_json::Value,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct WritingReview {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "session_id", with = "rid")]
    pub session: RecordId,
    #[serde(rename = "beat_id", with = "rid")]
    pub beat: RecordId,
    pub dimension: String,
    pub score: f32,
    pub passed: bool,
    pub issues: serde_json::Value,
    pub summary: String,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CharacterState {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "character_id", with = "rid")]
    pub character_id: RecordId,
    #[serde(rename = "beat_id", with = "rid")]
    pub beat_id: RecordId,
    pub emotion: String,
    pub location: String,
    pub knowledge: String,
    pub physical_state: String,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct CharacterRelation {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub char_a_id: String,
    pub char_a_name: Option<String>,
    pub char_b_id: String,
    pub char_b_name: Option<String>,
    pub relationship_type: String,
    pub description: String,
    #[serde(rename = "start_chapter_id", skip_serializing_if = "Option::is_none")]
    pub start_chapter_id: Option<String>,
    #[serde(
        rename = "start_chapter_title",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_chapter_title: Option<String>,
    #[serde(rename = "end_chapter_id", skip_serializing_if = "Option::is_none")]
    pub end_chapter_id: Option<String>,
    #[serde(rename = "end_chapter_title", skip_serializing_if = "Option::is_none")]
    pub end_chapter_title: Option<String>,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct CharacterFaction {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    pub character_id: String,
    pub character_name: Option<String>,
    pub faction_id: String,
    pub faction_name: Option<String>,
    pub role: String,
    #[serde(rename = "start_chapter_id", skip_serializing_if = "Option::is_none")]
    pub start_chapter_id: Option<String>,
    #[serde(
        rename = "start_chapter_title",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_chapter_title: Option<String>,
    #[serde(rename = "end_chapter_id", skip_serializing_if = "Option::is_none")]
    pub end_chapter_id: Option<String>,
    #[serde(rename = "end_chapter_title", skip_serializing_if = "Option::is_none")]
    pub end_chapter_title: Option<String>,
    pub created_at: Datetime,
}
