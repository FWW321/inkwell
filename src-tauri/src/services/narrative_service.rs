use crate::db::Store;
use crate::db::created_id;
use crate::db::models::{NarrativeBeat, NarrativeSession};
use crate::error::{AppError, AppResult};
use crate::services::agent_service::AgentConfig;
use crate::services::ai_service::StreamChunk;
use crate::services::{agent_service, ai_service, context_service};
use crate::state::Db;
use serde::Deserialize;
use surrealdb::types::{RecordId, ToSql, Value};
use tauri::ipc::Channel;

const QUEST_SWITCH_THRESHOLD: i64 = 5;
const FIRE_GAP_THRESHOLD: i64 = 10;
const CONSTELLATION_GAP_THRESHOLD: i64 = 15;
const QUEST_WARNING_THRESHOLD: i64 = 4;
const FIRE_GAP_WARNING: i64 = 8;
const CONSTELLATION_GAP_WARNING: i64 = 12;

#[derive(serde::Serialize, Clone)]
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
    let characters = context_service::get_project_characters(db, project_id).await?;
    let character_ids: Vec<String> = characters.iter().map(|c| c.id.key.to_sql()).collect();

    let session_id: String = db.query("CREATE narrative_session CONTENT { project: type::record('project', $pid), title: $title, scene: $scene, atmosphere: '', timeline_id: 'main', strand: 'quest', status: 'active' }")
        .bind(("pid", project_id.to_string()))
        .bind(("title", title.to_string()))
        .bind(("scene", scene.to_string()))
        .await?.check()?.take::<Option<Value>>(0)?
        .map(|v| created_id(&v))
        .transpose()?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("create narrative_session failed")))?;

    let beat_id: String = db.query("CREATE narrative_beat CONTENT { session: type::record('narrative_session', $sid), beat_type: 'scene_change', character_name: '系统', content: $scene, sort_order: 0, timeline_id: 'main', strand: 'quest' }")
        .bind(("sid", session_id.clone()))
        .bind(("scene", scene.to_string()))
        .await?.check()?.take::<Option<Value>>(0)?
        .map(|v| created_id(&v))
        .transpose()?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("create initial beat failed")))?;

    for cid in &character_ids {
        db.query(
            "RELATE type::record('character', $cid)->character_state->type::record('narrative_beat', $bid) \
             CONTENT { emotion: '平静', location: $scene, knowledge: '', physical_state: '' }"
        )
        .bind(("cid", cid.to_string()))
        .bind(("bid", beat_id.to_string()))
        .bind(("scene", scene.to_string()))
        .await?.check()?;
    }

    get_session(db, &session_id).await
}

pub async fn list_sessions(db: &Db, project_id: &str) -> AppResult<Vec<NarrativeSession>> {
    Store::new(db)
        .find("narrative_session")
        .filter_ref("project", "project", project_id)
        .order("updated_at DESC")
        .all()
        .await
}

pub async fn get_session(db: &Db, id: &str) -> AppResult<NarrativeSession> {
    Store::new(db).get("narrative_session", id).await
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
    Store::new(db)
        .find("narrative_beat")
        .filter_ref("session", "narrative_session", session_id)
        .order("sort_order ASC")
        .all()
        .await
}

pub async fn list_events(
    db: &Db,
    session_id: &str,
) -> AppResult<Vec<crate::db::models::NarrativeEvent>> {
    Store::new(db)
        .find("narrative_event")
        .filter_ref("session", "narrative_session", session_id)
        .order("created_at ASC")
        .all()
        .await
}

pub async fn delete_beat(db: &Db, id: &str) -> AppResult<()> {
    let _: Option<Value> = db.delete(("narrative_beat", id)).await?;
    db.query("DELETE FROM character_state WHERE out = type::record('narrative_beat', $bid)")
        .bind(("bid", id.to_string()))
        .await?
        .check()?;
    db.query("DELETE FROM narrative_event WHERE beat = type::record('narrative_beat', $bid)")
        .bind(("bid", id.to_string()))
        .await?
        .check()?;
    Ok(())
}

#[derive(Deserialize)]
pub struct BeatInput {
    pub character_id: Option<String>,
    pub character_name: String,
    pub content: String,
    pub strand: Option<String>,
}

const CREATE_BEAT_SQL: &str = "\
    BEGIN; \
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
     COMMIT";

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
    let strand = input.strand.as_deref().unwrap_or("quest");

    let mut q = db
        .query(CREATE_BEAT_SQL)
        .bind(("sid", session_id.to_string()))
        .bind(("beat_type", beat_type.to_string()))
        .bind(("has_char", has_char))
        .bind(("char_name", input.character_name.clone()))
        .bind(("content", input.content.clone()))
        .bind(("sort_order", max_order))
        .bind(("strand", strand.to_string()));
    if let Some(cid) = &input.character_id {
        q = q.bind(("cid", cid.to_string()));
    }

    let created: Option<NarrativeBeat> = q.await?.check()?.take::<Option<NarrativeBeat>>(1)?;

    created.ok_or_else(|| AppError::Internal(anyhow::anyhow!("add beat failed")))
}

pub async fn advance_narration(
    db: &Db,
    http: &reqwest::Client,
    session_id: &str,
    agent_id: Option<&str>,
    instruction: Option<&str>,
    on_chunk: &Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    let session = get_session(db, session_id).await?;
    let project_id = session.project.key.to_sql();

    let agent_config = agent_service::resolve_agent(db, agent_id, "叙事者", None).await?;
    let contract = context_service::build_contract(db, &project_id, session_id).await?;
    let strand = pick_next_strand(&contract.strand_context);

    let context =
        context_service::build_narrative_context(&contract, &session.scene, &session.atmosphere);
    let strand_guidance = build_strand_instruction(&strand, &contract.strand_context);
    let preamble = format!(
        "{}{}{}",
        agent_config.system_prompt, context, strand_guidance
    );

    let mut prompt = String::new();
    if let Some(instruction) = instruction {
        prompt.push_str(&format!("作者指令：{}\n\n", instruction));
    }
    prompt.push_str(
        "请推进剧情，描述场景变化、角色行动和情节发展。只输出叙事内容，不要加任何标记或说明。",
    );

    generate_and_save_beat(
        db,
        http,
        session_id,
        "narration",
        &strand,
        None,
        "叙事者",
        &agent_config,
        &preamble,
        &prompt,
        on_chunk,
    )
    .await
}

pub async fn invoke_character(
    db: &Db,
    http: &reqwest::Client,
    session_id: &str,
    character_id: &str,
    agent_id: Option<&str>,
    instruction: Option<&str>,
    on_chunk: &Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    let session = get_session(db, session_id).await?;
    let project_id = session.project.key.to_sql();
    let all_characters = context_service::get_project_characters(db, &project_id).await?;

    let character = all_characters
        .iter()
        .find(|c| c.id.key.to_sql() == character_id)
        .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?
        .clone();

    let agent_config = agent_service::resolve_agent(db, agent_id, "角色扮演", Some(&character.name)).await?;
    let contract = context_service::build_contract(db, &project_id, session_id).await?;
    let strand = session.strand.clone();

    let context = context_service::build_character_context(&contract, &character, &session.scene);
    let preamble = format!("{}\n{}", agent_config.system_prompt, context);

    let mut prompt = String::new();
    if let Some(inst) = instruction {
        prompt.push_str(&format!("{}\n\n", inst));
    }
    prompt.push_str("基于当前剧情场景，以该角色的身份做出反应。包括对话、行动和内心活动。只输出角色的反应内容，不要加任何标记或说明。");

    generate_and_save_beat(
        db,
        http,
        session_id,
        "character_action",
        &strand,
        Some(character_id),
        &character.name,
        &agent_config,
        &preamble,
        &prompt,
        on_chunk,
    )
    .await
}

async fn generate_and_save_beat(
    db: &Db,
    http: &reqwest::Client,
    session_id: &str,
    beat_type: &str,
    strand: &str,
    character_id: Option<&str>,
    character_name: &str,
    agent_config: &AgentConfig,
    preamble: &str,
    prompt: &str,
    on_chunk: &Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id: String::new(),
        beat_type: beat_type.to_string(),
        strand: strand.to_string(),
        character_id: character_id.map(String::from),
        character_name: character_name.to_string(),
        text: String::new(),
        done: false,
    });

    let mut full_text = String::new();

    ai_service::stream_ai_with_callback(
        http,
        &agent_config.model_config,
        preamble,
        prompt,
        agent_config.temperature,
        |chunk: StreamChunk| {
            if !chunk.text.is_empty() {
                full_text.push_str(&chunk.text);
                let _ = on_chunk.send(NarrativeStreamChunk {
                    beat_id: String::new(),
                    beat_type: beat_type.to_string(),
                    strand: strand.to_string(),
                    character_id: character_id.map(String::from),
                    character_name: character_name.to_string(),
                    text: chunk.text,
                    done: false,
                });
            }
        },
    )
    .await?;

    let beats = list_beats(db, session_id).await?;
    let max_order: i64 = beats.last().map_or(0, |b| b.sort_order + 1);
    let has_char = character_id.is_some();

    let mut q = db
        .query(CREATE_BEAT_SQL)
        .bind(("sid", session_id.to_string()))
        .bind(("beat_type", beat_type.to_string()))
        .bind(("has_char", has_char))
        .bind(("char_name", character_name.to_string()))
        .bind(("content", full_text))
        .bind(("sort_order", max_order))
        .bind(("strand", strand.to_string()));
    if let Some(cid) = character_id {
        q = q.bind(("cid", cid.to_string()));
    }

    q.await?.check()?;

    let _ = on_chunk.send(NarrativeStreamChunk {
        beat_id: String::new(),
        beat_type: beat_type.to_string(),
        strand: strand.to_string(),
        character_id: character_id.map(String::from),
        character_name: character_name.to_string(),
        text: String::new(),
        done: true,
    });

    Ok(())
}

pub fn pick_next_strand(ctx: &context_service::StrandContext) -> String {
    if ctx.quest_streak >= QUEST_SWITCH_THRESHOLD {
        return "fire".to_string();
    }
    if ctx.fire_gap >= FIRE_GAP_THRESHOLD {
        return "fire".to_string();
    }
    if ctx.constellation_gap >= CONSTELLATION_GAP_THRESHOLD {
        return "constellation".to_string();
    }
    ctx.current_strand.clone()
}

pub fn build_strand_instruction(strand: &str, ctx: &context_service::StrandContext) -> String {
    let guidance = match strand {
        "quest" => "本段以主线剧情为主，聚焦核心冲突和情节推进。",
        "fire" => "本段以感情线为主，展现角色之间的情感互动和关系变化。",
        "constellation" => "本段以世界观展开为主，揭示设定、背景或力量体系。",
        _ => "",
    };

    let mut warnings = Vec::with_capacity(3);
    if ctx.quest_streak >= QUEST_WARNING_THRESHOLD {
        warnings.push(format!(
            "注意：主线已连续 {} 段，建议下一轮切换到感情线或世界观线。",
            ctx.quest_streak
        ));
    }
    if ctx.fire_gap >= FIRE_GAP_WARNING {
        warnings.push(format!(
            "注意：感情线已断档 {} 段，读者可能感到缺失。",
            ctx.fire_gap
        ));
    }
    if ctx.constellation_gap >= CONSTELLATION_GAP_WARNING {
        warnings.push(format!(
            "注意：世界观线已断档 {} 段。",
            ctx.constellation_gap
        ));
    }

    if warnings.is_empty() {
        format!("\n\n{}", guidance)
    } else {
        format!("\n\n{}\n\n{}", guidance, warnings.join("\n"))
    }
}
