use surrealdb::types::{Datetime, RecordId, SurrealValue};

use crate::serde_helpers::{opt_rid, rid};

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
    #[serde(rename = "parent_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
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
    pub avatar_url: Option<String>,
    pub description: String,
    pub personality: String,
    pub background: String,
    pub race: String,
    #[serde(rename = "model_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
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
    pub avatar_url: Option<String>,
    pub description: String,
    pub personality: String,
    pub background: String,
    pub race: String,
    #[serde(rename = "model_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
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
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub is_default: bool,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct AiAgent {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub name: String,
    #[serde(rename = "model_id", with = "rid")]
    pub model: RecordId,
    pub system_prompt: String,
    pub is_default: bool,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct AiAgentWithModelName {
    #[serde(with = "rid")]
    pub id: RecordId,
    pub name: String,
    #[serde(rename = "model_id", with = "rid")]
    pub model: RecordId,
    pub system_prompt: String,
    pub is_default: bool,
    pub created_at: Datetime,
    #[serde(rename = "model_id")]
    pub model_id: String,
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
    pub character_states: serde_json::Value,
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
    #[serde(rename = "character_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
    pub character: Option<RecordId>,
    pub character_name: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub sort_order: i64,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct CharacterRelation {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    #[serde(rename = "char_a_id", with = "rid")]
    pub r#in: RecordId,
    #[serde(rename = "char_b_id", with = "rid")]
    pub r#out: RecordId,
    pub relationship_type: String,
    pub description: String,
    #[serde(rename = "start_chapter_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
    pub start_chapter: Option<RecordId>,
    #[serde(rename = "end_chapter_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
    pub end_chapter: Option<RecordId>,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct CharacterRelationWithNames {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    #[serde(rename = "char_a_id")]
    pub char_a_id: String,
    #[serde(rename = "char_a_name")]
    pub char_a_name: Option<String>,
    #[serde(rename = "char_b_id")]
    pub char_b_id: String,
    #[serde(rename = "char_b_name")]
    pub char_b_name: Option<String>,
    pub relationship_type: String,
    pub description: String,
    #[serde(rename = "start_chapter_id", skip_serializing_if = "Option::is_none")]
    pub start_chapter_id: Option<String>,
    #[serde(rename = "start_chapter_title", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "character_id", with = "rid")]
    pub r#in: RecordId,
    #[serde(rename = "faction_id", with = "rid")]
    pub r#out: RecordId,
    pub role: String,
    #[serde(rename = "start_chapter_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
    pub start_chapter: Option<RecordId>,
    #[serde(rename = "end_chapter_id", with = "opt_rid", skip_serializing_if = "Option::is_none")]
    pub end_chapter: Option<RecordId>,
    pub created_at: Datetime,
}

#[derive(Debug, Clone, SurrealValue, serde::Serialize)]
pub struct CharacterFactionWithNames {
    #[serde(with = "rid")]
    pub id: RecordId,
    #[serde(rename = "project_id", with = "rid")]
    pub project: RecordId,
    #[serde(rename = "character_id")]
    pub character_id: String,
    #[serde(rename = "character_name")]
    pub character_name: Option<String>,
    #[serde(rename = "faction_id")]
    pub faction_id: String,
    #[serde(rename = "faction_name")]
    pub faction_name: Option<String>,
    pub role: String,
    #[serde(rename = "start_chapter_id", skip_serializing_if = "Option::is_none")]
    pub start_chapter_id: Option<String>,
    #[serde(rename = "start_chapter_title", skip_serializing_if = "Option::is_none")]
    pub start_chapter_title: Option<String>,
    #[serde(rename = "end_chapter_id", skip_serializing_if = "Option::is_none")]
    pub end_chapter_id: Option<String>,
    #[serde(rename = "end_chapter_title", skip_serializing_if = "Option::is_none")]
    pub end_chapter_title: Option<String>,
    pub created_at: Datetime,
}
