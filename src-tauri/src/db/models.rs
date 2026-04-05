use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub language: String,
    pub tags: String,
    pub status: String,
    pub cover_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineNode {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub node_type: String,
    pub title: String,
    pub sort_order: i64,
    pub content_json: String,
    pub word_count: i64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_new: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_mode: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub description: String,
    pub personality: String,
    pub background: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldviewEntry {
    pub id: String,
    pub project_id: String,
    pub category: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub is_default: bool,
    pub created_at: String,
}
