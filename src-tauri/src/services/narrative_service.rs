use crate::db::get_created_id;
use crate::db::models::{AiConfig, Character, NarrativeBeat, NarrativeSession, NarrativeEvent};
use crate::error::{AppError, AppResult};
use crate::services::{ai_service, context_service};
use crate::state::Db;
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue, ToSql, Value};
use tauri::ipc::Channel;

#[derive(Serialize, Clone)]
pub struct NarrativeStreamChunk {
    pub beat_id: String,
    pub beat_type: String,
    pub strand: String,
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
    let character_ids: Vec<String> = characters.iter().map(|c| c.id.key.to_sql()).collect();

    let session_id: String = db.query("CREATE narrative_session CONTENT { project: type::record('project', $pid), title: $title, scene: $scene, atmosphere: '', timeline_id: 'main', strand: 'quest', status: 'active' }")
        .bind(("pid", project_id.to_string()))
        .bind(("title", title.to_string()))
        .bind(("scene", scene.to_string()))
        .await?.check()?.take::<Option<Value>>(0)?
        .map(|v| get_created_id(&v))
        .ok_or_else(|| AppError::Internal("create narrative_session failed".into()))?;

    let beat_id: String = db.query("CREATE narrative_beat CONTENT { session: type::record('narrative_session', $sid), beat_type: 'scene_change', character_name: '系统', content: $scene, sort_order: 0, timeline_id: 'main', strand: 'quest' }")
        .bind(("sid", session_id.clone()))
        .bind(("scene", scene.to_string()))
        .await?.check()?.take::<Option<Value>>(0)?
        .map(|v| get_created_id(&v))
        .ok_or_else(|| AppError::Internal("create initial beat failed".into()))?;

    for cid in &character_ids {
        let sql = format!(
            "RELATE character:{cid}->character_state->narrative_beat:{bid} CONTENT {{ emotion: '平静', location: $scene, knowledge: '', physical_state: '' }}",
            cid = cid,
            bid = beat_id
        );
        db.query(&sql)
            .bind(("scene", scene.to_string()))
            .await?.check()?;
    }

    get_session(db, &session_id).await
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

    let session: Option<NarrativeSession> = db.select(&sid).await?;
    if session.is_none() {
        return Err(AppError::NotFound("推演会话不存在".to_string()));
    }

    db.query(
        "BEGIN; \
         DELETE FROM character_state WHERE out IN (SELECT VALUE id FROM narrative_beat WHERE session = $sid); \
         DELETE FROM narrative_event WHERE session = $sid; \
         DELETE FROM writing_review WHERE session = $sid; \
         DELETE FROM narrative_beat WHERE session = $sid; \
         DELETE $sid; \
         COMMIT"
    )
    .bind(("sid", sid))
    .await?
    .check()?;
    Ok(())
}

pub async fn list_beats(db: &Db, session_id: &str) -> AppResult<Vec<NarrativeBeat>> {
    db.query("SELECT * FROM narrative_beat WHERE session = $sid ORDER BY sort_order ASC")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?.take::<Vec<NarrativeBeat>>(0).map_err(Into::into)
}

pub async fn list_events(db: &Db, session_id: &str) -> AppResult<Vec<NarrativeEvent>> {
    db.query("SELECT * FROM narrative_event WHERE session = $sid ORDER BY created_at ASC")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?.take::<Vec<NarrativeEvent>>(0).map_err(Into::into)
}

pub async fn delete_beat(db: &Db, id: &str) -> AppResult<()> {
    let _: Option<Value> = db.delete(("narrative_beat", id)).await?;
    db.query("DELETE FROM character_state WHERE out = type::record('narrative_beat', $bid)")
        .bind(("bid", id.to_string()))
        .await?;
    db.query("DELETE FROM narrative_event WHERE beat = type::record('narrative_beat', $bid)")
        .bind(("bid", id.to_string()))
        .await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct BeatInput {
    pub character_id: Option<String>,
    pub character_name: String,
    pub content: String,
    pub strand: Option<String>,
}

pub async fn add_beat(
    db: &Db,
    session_id: &str,
    beat_type: &str,
    input: BeatInput,
) -> AppResult<NarrativeBeat> {
    let result: Option<i64> = db
        .query("SELECT VALUE sort_order FROM narrative_beat WHERE session = $sid ORDER BY sort_order DESC LIMIT 1")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?.check()?.take::<Option<i64>>(0)?;
    let max_order = result.map(|v| v + 1).unwrap_or(0);
    let has_char = input.character_id.is_some();
    let strand = input.strand.unwrap_or_else(|| "quest".to_string());

    let mut q = db.query(
        "BEGIN; \
         CREATE narrative_beat CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat_type: $beat_type, \
         character: if $has_char { type::record('character', $cid) } else { NONE }, \
         character_name: $char_name, \
         content: $content, \
         sort_order: $sort_order, \
         timeline_id: 'main', \
         strand: $strand \
         }; \
         UPDATE type::record('narrative_session', $sid) SET status = 'active'; \
         COMMIT"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("beat_type", beat_type.to_string()))
    .bind(("has_char", has_char))
    .bind(("char_name", input.character_name.clone()))
    .bind(("content", input.content.clone()))
    .bind(("sort_order", max_order))
    .bind(("strand", strand));
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

    let contract = context_service::build_contract(db, &project_id, session_id).await?;

    let strand = pick_next_strand(&contract.strand_context);

    let preamble = context_service::format_narrator_prompt(&contract, &session.scene, &session.atmosphere);
    inject_strand_guidance(&preamble, &strand, &contract.strand_context, narrator_instruction);

    let narrator_preamble = format!(
        "{}\n\n{}",
        preamble,
        build_strand_instruction(&strand, &contract.strand_context)
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
        strand: strand.clone(),
        character_id: None,
        character_name: "叙事者".to_string(),
        text: String::new(),
        done: false,
    });

    let narrator_config = get_narrator_config(db).await?;
    let result = call_narrator(&narrator_config, &narrator_preamble, &prompt).await?;

    let mut full_text = String::new();
    for ch in result.chars() {
        full_text.push(ch);
        let _ = on_chunk.send(NarrativeStreamChunk {
            beat_id: beat_id.clone(),
            beat_type: "narration".to_string(),
            strand: strand.clone(),
            character_id: None,
            character_name: "叙事者".to_string(),
            text: ch.to_string(),
            done: false,
        });
    }

    let beats = list_beats(db, session_id).await?;
    let max_order: i64 = beats.last().map_or(0, |b| b.sort_order + 1);

    let _beat: Option<NarrativeBeat> = db.query(
        "BEGIN; \
         CREATE narrative_beat CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat_type: 'narration', \
         character_name: '叙事者', \
         content: $content, \
         sort_order: $sort_order, \
         timeline_id: 'main', \
         strand: $strand \
         }; \
         UPDATE type::record('narrative_session', $sid) MERGE { scene: $scene, status: 'active' }; \
         COMMIT"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("content", full_text))
    .bind(("sort_order", max_order))
    .bind(("scene", session.scene))
    .bind(("strand", strand.clone()))
    .await?
    .check()?
    .take::<Option<NarrativeBeat>>(1)?;

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id,
        beat_type: "narration".to_string(),
        strand,
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

    let contract = context_service::build_contract(db, &project_id, session_id).await?;
    let strand = session.strand.clone();

    let character_preamble = context_service::format_character_prompt(&contract, &character, &session.scene);

    let mut prompt = String::new();
    if let Some(inst) = instruction {
        prompt.push_str(&format!("{}\n\n", inst));
    }
    prompt.push_str("基于当前剧情场景，以该角色的身份做出反应。包括对话、行动和内心活动。只输出角色的反应内容，不要加任何标记或说明。");

    let beat_id = String::new();

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id: beat_id.clone(),
        beat_type: "character_action".to_string(),
        strand: strand.clone(),
        character_id: Some(character_id.to_string()),
        character_name: character.name.clone(),
        text: String::new(),
        done: false,
    });

    let character_config = match &character.model {
        Some(mid) => {
            let model_key = mid.key.to_sql();
            ai_service::get_model(db, &model_key).await?
        }
        None => ai_service::get_default_config(db).await?,
    };

    let result = call_narrator(&character_config, &character_preamble, &prompt).await?;

    let mut full_text = String::new();
    for ch in result.chars() {
        full_text.push(ch);
        let _ = on_chunk.send(NarrativeStreamChunk {
            beat_id: beat_id.clone(),
            beat_type: "character_action".to_string(),
            strand: strand.clone(),
            character_id: Some(character_id.to_string()),
            character_name: character.name.clone(),
            text: ch.to_string(),
            done: false,
        });
    }

    let beats = list_beats(db, session_id).await?;
    let max_order: i64 = beats.last().map_or(0, |b| b.sort_order + 1);

    db.query(
        "BEGIN; \
         CREATE narrative_beat CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat_type: 'character_action', \
         character: type::record('character', $cid), \
         character_name: $char_name, \
         content: $content, \
         sort_order: $sort_order, \
         timeline_id: 'main', \
         strand: $strand \
         }; \
         UPDATE type::record('narrative_session', $sid) SET status = 'active'; \
         COMMIT"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("cid", character_id.to_string()))
    .bind(("char_name", character.name.clone()))
    .bind(("content", full_text))
    .bind(("sort_order", max_order))
    .bind(("strand", strand.clone()))
    .await?
    .check()?;

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id,
        beat_type: "character_action".to_string(),
        strand,
        character_id: Some(character_id.to_string()),
        character_name: character.name,
        text: String::new(),
        done: true,
    });

    Ok(())
}

fn pick_next_strand(ctx: &context_service::StrandContext) -> String {
    if ctx.quest_streak >= 5 {
        return "fire".to_string();
    }
    if ctx.fire_gap >= 10 {
        return "fire".to_string();
    }
    if ctx.constellation_gap >= 15 {
        return "constellation".to_string();
    }
    ctx.current_strand.clone()
}

fn inject_strand_guidance(
    _preamble: &str,
    _strand: &str,
    _ctx: &context_service::StrandContext,
    _instruction: Option<&str>,
) {
}

fn build_strand_instruction(strand: &str, ctx: &context_service::StrandContext) -> String {
    let guidance = match strand {
        "quest" => "本段以主线剧情为主，聚焦核心冲突和情节推进。",
        "fire" => "本段以感情线为主，展现角色之间的情感互动和关系变化。",
        "constellation" => "本段以世界观展开为主，揭示设定、背景或力量体系。",
        _ => "",
    };

    let mut warnings = Vec::new();
    if ctx.quest_streak >= 4 {
        warnings.push(format!("注意：主线已连续 {} 段，建议下一轮切换到感情线或世界观线。", ctx.quest_streak));
    }
    if ctx.fire_gap >= 8 {
        warnings.push(format!("注意：感情线已断档 {} 段，读者可能感到缺失。", ctx.fire_gap));
    }
    if ctx.constellation_gap >= 12 {
        warnings.push(format!("注意：世界观线已断档 {} 段。", ctx.constellation_gap));
    }

    if warnings.is_empty() {
        guidance.to_string()
    } else {
        format!("{}\n\n{}", guidance, warnings.join("\n"))
    }
}

async fn get_narrator_config(db: &Db) -> AppResult<AiConfig> {
    let default_config = ai_service::get_default_config(db).await?;

    let result: Option<RecordId> = db
        .query("SELECT model FROM ai_agent WHERE is_default = true LIMIT 1")
        .await?
        .take::<Option<RecordId>>(0)?;

    match result {
        Some(model_rid) => {
            let model_key = model_rid.key.to_sql();
            ai_service::get_model(db, &model_key).await
        }
        None => Ok(default_config),
    }
}

async fn call_narrator(
    config: &AiConfig,
    preamble: &str,
    prompt: &str,
) -> AppResult<String> {
    super::ai_service::chat_completion(config, preamble, prompt, 0.85).await
}
