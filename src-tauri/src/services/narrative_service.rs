use crate::db::models::{AiConfig, Character, NarrativeBeat, NarrativeSession};
use crate::error::{AppError, AppResult};
use crate::services::{ai_service, context_service};
use crate::state::Db;
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, ToSql, Value};
use tauri::ipc::Channel;

#[derive(Serialize, Clone)]
pub struct NarrativeStreamChunk {
    pub beat_id: String,
    pub beat_type: String,
    pub character_id: Option<String>,
    pub character_name: String,
    pub text: String,
    pub done: bool,
}

pub async fn create_session(
    db: &Db,
    project_id: &str,
    title: &str,
    scene: &str,
) -> AppResult<NarrativeSession> {
    let characters = get_project_characters(db, project_id).await?;
    let mut character_states = serde_json::json!({})
        .as_object()
        .unwrap()
        .clone();

    for ch in &characters {
        let ch_state = serde_json::json!({
            "emotion": "平静",
            "location": scene,
            "knowledge": ""
        });
        character_states.insert(ch.id.key.to_sql(), ch_state);
    }

    db.query(
        "CREATE narrative_session CONTENT { \
         project: type::record('project', $pid), \
         title: $title, \
         scene: $scene, \
         atmosphere: '', \
         character_states: $char_states, \
         status: 'active' \
         }"
    )
    .bind(("pid", project_id.to_string()))
    .bind(("title", title.to_string()))
    .bind(("scene", scene.to_string()))
    .bind(("char_states", serde_json::Value::Object(character_states)))
    .await?
    .take::<Option<NarrativeSession>>(0)?
    .ok_or_else(|| AppError::Internal("create narrative_session failed".into()))
}

pub async fn list_sessions(db: &Db, project_id: &str) -> AppResult<Vec<NarrativeSession>> {
    db.query("SELECT * FROM narrative_session WHERE project = $pid ORDER BY updated_at DESC")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<NarrativeSession>>(0).map_err(Into::into)
}

pub async fn get_session(db: &Db, id: &str) -> AppResult<NarrativeSession> {
    db.select(("narrative_session", id)).await?
        .ok_or_else(|| AppError::NotFound(format!("推演会话 {} 不存在", id)))
}

pub async fn delete_session(db: &Db, id: &str) -> AppResult<()> {
    let sid = RecordId::new("narrative_session", id);

    let session: Option<NarrativeSession> = db.select((&sid).clone()).await?;
    if session.is_none() {
        return Err(AppError::NotFound("推演会话不存在".to_string()));
    }

    db.query("BEGIN; DELETE type::record($sid); DELETE FROM narrative_beat WHERE session = $sid; COMMIT")
        .bind(("sid", sid))
        .await?;
    Ok(())
}

pub async fn list_beats(db: &Db, session_id: &str) -> AppResult<Vec<NarrativeBeat>> {
    db.query("SELECT * FROM narrative_beat WHERE session = $sid ORDER BY sort_order ASC")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?.take::<Vec<NarrativeBeat>>(0).map_err(Into::into)
}

pub async fn delete_beat(db: &Db, id: &str) -> AppResult<()> {
    let _: Option<Value> = db.delete(("narrative_beat", id)).await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct BeatInput {
    pub character_id: Option<String>,
    pub character_name: String,
    pub content: String,
}

pub async fn add_beat(
    db: &Db,
    session_id: &str,
    beat_type: &str,
    input: BeatInput,
) -> AppResult<NarrativeBeat> {
    let result: Option<i64> = db
        .query("SELECT VALUE math::max(sort_order) + 1 FROM narrative_beat WHERE session = $sid")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?
        .take::<Option<i64>>(0)?;
    let max_order = result.unwrap_or(0);

    let has_char = input.character_id.is_some();

    let mut q = db.query(
        "BEGIN; \
         CREATE narrative_beat CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat_type: $beat_type, \
         character: if $has_char { type::record('character', $cid) } else { NONE }, \
         character_name: $char_name, \
         content: $content, \
         sort_order: $sort_order \
         }; \
         UPDATE type::record('narrative_session', $sid) SET status = 'active'; \
         COMMIT"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("beat_type", beat_type.to_string()))
    .bind(("has_char", has_char))
    .bind(("char_name", input.character_name.clone()))
    .bind(("content", input.content.clone()))
    .bind(("sort_order", max_order));
    if let Some(cid) = &input.character_id {
        q = q.bind(("cid", cid.to_string()));
    }

    let created: Option<NarrativeBeat> = q.await?
        .check()?
        .take::<Option<NarrativeBeat>>(1)?;

    created.ok_or_else(|| AppError::Internal("add beat failed".into()))
}

async fn get_project_characters(db: &Db, project_id: &str) -> AppResult<Vec<Character>> {
    db.query("SELECT * FROM character WHERE project = $pid ORDER BY name")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<Character>>(0).map_err(Into::into)
}

pub async fn advance_narration(
    db: &Db,
    session_id: &str,
    narrator_instruction: Option<&str>,
    on_chunk: &Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    let session = get_session(db, session_id).await?;
    let project_id = session.project.key.to_sql();
    let characters = get_project_characters(db, &project_id).await?;
    let beats = list_beats(db, session_id).await?;

    let default_config = ai_service::get_default_config(db).await?;

    let recent_beats: Vec<&NarrativeBeat> = beats.iter().rev().take(20).rev().collect();
    let context_summary = build_context_summary(&recent_beats);
    let worldview_summary = context_service::build_worldview_summary(db, &project_id).await?;
    let characters_summary = build_narrative_characters_summary(&characters);

    let narrator_preamble = build_narrator_preamble(
        &session,
        &characters_summary,
        &worldview_summary,
        &context_summary,
    );

    let mut prompt = String::new();
    if let Some(instruction) = narrator_instruction {
        prompt.push_str(&format!("作者指令：{}\n\n", instruction));
    }
    prompt.push_str("请推进剧情，描述场景变化、角色行动和情节发展。只输出叙事内容，不要加任何标记或说明。");

    let beat_id = String::new();

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id: beat_id.clone(),
        beat_type: "narration".to_string(),
        character_id: None,
        character_name: "叙事者".to_string(),
        text: String::new(),
        done: false,
    });

    let narrator_config = get_narrator_config(db, &default_config).await?;
    let result = call_narrator(&narrator_config, &narrator_preamble, &prompt).await?;

    let mut full_text = String::new();
    for ch in result.chars() {
        full_text.push(ch);
        let _ = on_chunk.send(NarrativeStreamChunk {
            beat_id: beat_id.clone(),
            beat_type: "narration".to_string(),
            character_id: None,
            character_name: "叙事者".to_string(),
            text: ch.to_string(),
            done: false,
        });
    }

    let max_order: i64 = beats.last().map_or(0, |b| b.sort_order + 1);

    db.query(
        "BEGIN; \
         CREATE narrative_beat CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat_type: 'narration', \
         character_name: '叙事者', \
         content: $content, \
         sort_order: $sort_order \
         }; \
         UPDATE type::record('narrative_session', $sid) MERGE { scene: $scene, status: 'active' }; \
         COMMIT"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("content", full_text))
    .bind(("sort_order", max_order))
    .bind(("scene", session.scene))
    .await?;

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id,
        beat_type: "narration".to_string(),
        character_id: None,
        character_name: "叙事者".to_string(),
        text: String::new(),
        done: true,
    });

    Ok(())
}

pub async fn invoke_character(
    db: &Db,
    session_id: &str,
    character_id: &str,
    instruction: Option<&str>,
    on_chunk: &Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    let session = get_session(db, session_id).await?;
    let project_id = session.project.key.to_sql();
    let all_characters = get_project_characters(db, &project_id).await?;

    let character = all_characters
        .iter()
        .find(|c| c.id.key.to_sql() == character_id)
        .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?
        .clone();

    let beats = list_beats(db, session_id).await?;
    let recent_beats: Vec<&NarrativeBeat> = beats.iter().rev().take(20).rev().collect();
    let context_summary = build_context_summary(&recent_beats);
    let worldview_summary = context_service::build_worldview_summary(db, &project_id).await?;
    let characters_summary = build_narrative_characters_summary(&all_characters);

    let character_config = match &character.model {
        Some(mid) => {
            let model_key = mid.key.to_sql();
            ai_service::get_model(db, &model_key).await?
        }
        None => ai_service::get_default_config(db).await?,
    };

    let character_preamble = build_character_preamble(
        &character,
        &characters_summary,
        &worldview_summary,
        &context_summary,
        &session.scene,
    );

    let mut prompt = String::new();
    if let Some(inst) = instruction {
        prompt.push_str(&format!("{}\n\n", inst));
    }
    prompt.push_str("基于当前剧情场景，以该角色的身份做出反应。包括对话、行动和内心活动。只输出角色的反应内容，不要加任何标记或说明。");

    let beat_id = String::new();

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id: beat_id.clone(),
        beat_type: "character_action".to_string(),
        character_id: Some(character_id.to_string()),
        character_name: character.name.clone(),
        text: String::new(),
        done: false,
    });

    let result = call_narrator(&character_config, &character_preamble, &prompt).await?;

    let mut full_text = String::new();
    for ch in result.chars() {
        full_text.push(ch);
        let _ = on_chunk.send(NarrativeStreamChunk {
            beat_id: beat_id.clone(),
            beat_type: "character_action".to_string(),
            character_id: Some(character_id.to_string()),
            character_name: character.name.clone(),
            text: ch.to_string(),
            done: false,
        });
    }

    let max_order: i64 = beats.last().map_or(0, |b| b.sort_order + 1);

    db.query(
        "BEGIN; \
         CREATE narrative_beat CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat_type: 'character_action', \
         character: type::record('character', $cid), \
         character_name: $char_name, \
         content: $content, \
         sort_order: $sort_order \
         }; \
         UPDATE type::record('narrative_session', $sid) SET status = 'active'; \
         COMMIT"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("cid", character_id.to_string()))
    .bind(("char_name", character.name.clone()))
    .bind(("content", full_text))
    .bind(("sort_order", max_order))
    .await?;

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id,
        beat_type: "character_action".to_string(),
        character_id: Some(character_id.to_string()),
        character_name: character.name,
        text: String::new(),
        done: true,
    });

    Ok(())
}

async fn get_narrator_config(db: &Db, default_config: &AiConfig) -> AppResult<AiConfig> {
    let result: Option<RecordId> = db
        .query("SELECT model FROM ai_agent WHERE is_default = true LIMIT 1")
        .await?
        .take::<Option<RecordId>>(0)?;

    match result {
        Some(model_rid) => {
            let model_key = model_rid.key.to_sql();
            ai_service::get_model(db, &model_key).await
        }
        None => Ok(default_config.clone()),
    }
}

fn build_narrator_preamble(
    session: &NarrativeSession,
    characters_summary: &str,
    worldview_summary: &str,
    context_summary: &str,
) -> String {
    let mut parts = Vec::new();

    parts.push("你是一位专业的小说叙事者，负责推动剧情发展。你的职责是：".to_string());
    parts.push("1. 描述场景变化和氛围".to_string());
    parts.push("2. 推动情节发展，创造戏剧冲突".to_string());
    parts.push("3. 控制节奏，在紧张和舒缓之间切换".to_string());
    parts.push("4. 为角色创造互动机会".to_string());

    parts.push(format!("\n当前场景：{}", session.scene));
    if !session.atmosphere.is_empty() {
        parts.push(format!("氛围：{}", session.atmosphere));
    }

    if !characters_summary.is_empty() {
        parts.push(format!("\n在场角色：\n{}", characters_summary));
    }

    if !worldview_summary.is_empty() {
        parts.push(format!("\n世界观设定：\n{}", worldview_summary));
    }

    if !context_summary.is_empty() {
        parts.push(format!("\n前情提要（最近的剧情节拍）：\n{}", context_summary));
    }

    parts.join("\n")
}

fn build_character_preamble(
    character: &Character,
    characters_summary: &str,
    worldview_summary: &str,
    context_summary: &str,
    current_scene: &str,
) -> String {
    let mut parts = Vec::new();

    parts.push(format!(
        "你现在正在扮演小说中的角色「{}」。你必须完全沉浸在这个角色中，按照角色的性格、背景和当前处境做出真实的反应。",
        character.name
    ));

    if !character.personality.is_empty() {
        parts.push(format!("\n性格特点：{}", character.personality));
    }
    if !character.description.is_empty() {
        parts.push(format!("外貌特征：{}", character.description));
    }
    if !character.background.is_empty() {
        parts.push(format!("背景故事：{}", character.background));
    }

    parts.push(format!("\n当前场景：{}", current_scene));

    if !characters_summary.is_empty() {
        parts.push(format!("\n其他角色：\n{}", characters_summary));
    }

    if !worldview_summary.is_empty() {
        parts.push(format!("\n世界观设定：\n{}", worldview_summary));
    }

    if !context_summary.is_empty() {
        parts.push(format!("\n最近发生的事：\n{}", context_summary));
    }

    parts.push("\n重要：只输出角色的反应（对话用「」包裹、行动描述、心理活动），不要跳出角色，不要加任何元说明。".to_string());

    parts.join("\n")
}

fn build_context_summary(beats: &[&NarrativeBeat]) -> String {
    if beats.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for beat in beats {
        match beat.beat_type.as_str() {
            "narration" => {
                lines.push(format!("[叙事] {}", beat.content));
            }
            "character_action" => {
                lines.push(format!("[{}] {}", beat.character_name, beat.content));
            }
            "scene_change" => {
                lines.push(format!("[场景切换] {}", beat.content));
            }
            "author_intervention" => {
                lines.push(format!("[作者指令] {}", beat.content));
            }
            _ => {
                lines.push(format!("[{}] {}", beat.character_name, beat.content));
            }
        }
    }

    lines.join("\n")
}

fn build_narrative_characters_summary(characters: &[Character]) -> String {
    characters
        .iter()
        .map(|c| {
            let mut info = format!("- {}：{}", c.name, c.personality);
            if !c.description.is_empty() {
                info.push_str(&format!("（{}）", c.description));
            }
            if !c.background.is_empty() {
                info.push_str(&format!("\n  背景：{}", c.background));
            }
            info
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn call_narrator(
    config: &AiConfig,
    preamble: &str,
    prompt: &str,
) -> AppResult<String> {
    super::ai_service::chat_completion(config, preamble, prompt, 0.85).await
}
