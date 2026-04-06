use crate::db::models::Character;
use crate::error::AppResult;
use crate::state::Db;
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize)]
pub struct ContextContract {
    pub project_title: String,
    pub characters: Vec<CharacterContext>,
    pub worldview: Vec<WorldviewItem>,
    pub recent_beats: Vec<BeatSummary>,
    pub character_states: Vec<CharacterStateView>,
    pub strand_context: StrandContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterContext {
    pub name: String,
    pub personality: String,
    pub description: String,
    pub background: String,
    pub state: Option<CharacterStateView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldviewItem {
    pub category: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BeatSummary {
    pub beat_type: String,
    pub character_name: String,
    pub content: String,
    pub strand: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterStateView {
    pub name: String,
    pub emotion: String,
    pub location: String,
    pub knowledge: String,
    pub physical_state: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StrandContext {
    pub current_strand: String,
    pub quest_streak: i64,
    pub fire_gap: i64,
    pub constellation_gap: i64,
}

pub async fn build_contract(
    db: &Db,
    project_id: &str,
    session_id: &str,
) -> AppResult<ContextContract> {
    let project_title = get_project_title(db, project_id).await?;
    let characters = get_project_characters(db, project_id).await?;
    let worldview = get_worldview_entries(db, project_id).await?;
    let recent_beats = get_recent_beats(db, session_id).await?;
    let character_states = get_character_states(db, session_id).await?;
    let strand_context = get_strand_context(db, session_id).await?;

    let character_contexts: Vec<CharacterContext> = characters
        .iter()
        .map(|c| {
            let state = character_states.iter().find(|s| s.name == c.name).cloned();
            CharacterContext {
                name: c.name.clone(),
                personality: c.personality.clone(),
                description: c.description.clone(),
                background: c.background.clone(),
                state,
            }
        })
        .collect();

    Ok(ContextContract {
        project_title,
        characters: character_contexts,
        worldview,
        recent_beats,
        character_states,
        strand_context,
    })
}

pub fn format_narrator_prompt(contract: &ContextContract, scene: &str, atmosphere: &str) -> String {
    let mut parts = Vec::new();

    parts.push("你是一位专业的小说叙事者，负责推动剧情发展。你的职责是：描述场景变化和氛围、推动情节发展创造戏剧冲突、控制节奏在紧张和舒缓之间切换、为角色创造互动机会。只输出叙事内容，不要加任何标记或说明。".to_string());

    parts.push(format!("\n当前场景：{}", scene));
    if !atmosphere.is_empty() {
        parts.push(format!("氛围：{}", atmosphere));
    }

    if !contract.character_states.is_empty() {
        parts.push("\n角色当前状态：".to_string());
        for cs in &contract.character_states {
            let mut info = format!("- {}：情绪 {}", cs.name, cs.emotion);
            if !cs.location.is_empty() {
                info.push_str(&format!("，位于 {}", cs.location));
            }
            if !cs.physical_state.is_empty() {
                info.push_str(&format!("，{}", cs.physical_state));
            }
            parts.push(info);
        }
    }

    let chars_summary: Vec<String> = contract
        .characters
        .iter()
        .map(|c| {
            let mut info = format!("- {}：{}", c.name, c.personality);
            if !c.description.is_empty() {
                info.push_str(&format!("（{}）", c.description));
            }
            info
        })
        .collect();
    if !chars_summary.is_empty() {
        parts.push(format!("\n在场角色：\n{}", chars_summary.join("\n")));
    }

    if !contract.worldview.is_empty() {
        let wv: Vec<String> = contract
            .worldview
            .iter()
            .map(|w| {
                if w.content.is_empty() {
                    format!("- [{}] {}", w.category, w.title)
                } else {
                    format!("- [{}] {}：{}", w.category, w.title, w.content)
                }
            })
            .collect();
        parts.push(format!("\n世界观设定：\n{}", wv.join("\n")));
    }

    if !contract.recent_beats.is_empty() {
        let beats: Vec<String> = contract
            .recent_beats
            .iter()
            .map(|b| {
                let label = match b.beat_type.as_str() {
                    "narration" => "叙事",
                    "character_action" => &b.character_name,
                    "scene_change" => "场景切换",
                    "author_intervention" => "作者指令",
                    _ => &b.character_name,
                };
                format!("[{}] {}", label, b.content)
            })
            .collect();
        parts.push(format!("\n前情提要：\n{}", beats.join("\n")));
    }

    parts.join("\n")
}

pub fn format_character_prompt(
    contract: &ContextContract,
    character: &Character,
    scene: &str,
) -> String {
    let mut parts = Vec::new();

    parts.push(format!(
        "你现在正在扮演小说中的角色「{}」。你必须完全沉浸在这个角色中，按照角色的性格、背景和当前处境做出真实的反应。只输出角色的反应（对话用「」包裹、行动描述、心理活动），不要跳出角色，不要加任何元说明。",
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

    parts.push(format!("\n当前场景：{}", scene));

    if let Some(self_state) = contract.character_states.iter().find(|s| s.name == character.name) {
        let mut state_info = "你当前状态：".to_string();
        if !self_state.emotion.is_empty() {
            state_info.push_str(&format!("情绪 {}", self_state.emotion));
        }
        if !self_state.location.is_empty() {
            state_info.push_str(&format!("，位于 {}", self_state.location));
        }
        if !self_state.knowledge.is_empty() {
            state_info.push_str(&format!("，已知 {}", self_state.knowledge));
        }
        if !self_state.physical_state.is_empty() {
            state_info.push_str(&format!("，{}", self_state.physical_state));
        }
        parts.push(state_info);
    }

    let others: Vec<String> = contract
        .characters
        .iter()
        .filter(|c| c.name != character.name)
        .map(|c| format!("- {}：{}", c.name, c.personality))
        .collect();
    if !others.is_empty() {
        parts.push(format!("\n其他角色：\n{}", others.join("\n")));
    }

    if !contract.worldview.is_empty() {
        let wv: Vec<String> = contract
            .worldview
            .iter()
            .map(|w| format!("- [{}] {}", w.category, w.title))
            .collect();
        parts.push(format!("\n世界观设定：\n{}", wv.join("\n")));
    }

    if !contract.recent_beats.is_empty() {
        let beats: Vec<String> = contract
            .recent_beats
            .iter()
            .map(|b| {
                let label = match b.beat_type.as_str() {
                    "narration" => "叙事",
                    "character_action" => &b.character_name,
                    "scene_change" => "场景切换",
                    _ => &b.character_name,
                };
                format!("[{}] {}", label, b.content)
            })
            .collect();
        parts.push(format!("\n最近发生的事：\n{}", beats.join("\n")));
    }

    parts.join("\n")
}

pub fn format_editor_system_prompt(contract: &ContextContract, mode: &str) -> String {
    let mut parts = Vec::new();

    parts.push("你是 Inkwell 写作助手，一位专业的小说创作顾问。".to_string());

    if !contract.project_title.is_empty() {
        parts.push(format!("当前项目：《{}》", contract.project_title));
    }

    let chars_summary: Vec<String> = contract
        .characters
        .iter()
        .map(|c| format!("- {}：{}", c.name, c.personality))
        .collect();
    if !chars_summary.is_empty() {
        parts.push(format!("主要角色：\n{}", chars_summary.join("\n")));
    }

    if !contract.worldview.is_empty() {
        let wv: Vec<String> = contract
            .worldview
            .iter()
            .map(|w| format!("- [{}] {}", w.category, w.title))
            .collect();
        parts.push(format!("世界观设定：\n{}", wv.join("\n")));
    }

    match mode {
        "continue" => {
            parts.push(
                "你的任务是根据已有内容续写故事。请保持与前文一致的风格、语气和叙事视角。\
                 续写内容要自然流畅，与前文无缝衔接。直接输出续写内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        "rewrite" => {
            parts.push(
                "你的任务是改写用户提供的文字。改写时要保持原文的核心意思，根据用户指令调整风格和表达。\
                 直接输出改写后的内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        "polish" => {
            parts.push(
                "你的任务是润色用户提供的文字，提升表达质量，使之更加流畅优美。\
                 保持原文的核心意思不变，适当优化用词和句式。直接输出润色后的内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        "dialogue" => {
            parts.push(
                "你是一位擅长创作对话的小说家。根据提供的角色信息和场景描述，生成自然生动的角色对话。\
                 对话要符合每个角色的性格特点，推动情节发展。使用中文引号「」包裹对话内容，并标注说话者。\
                 直接输出对话内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        _ => {
            parts.push(
                "你可以帮助用户解决写作中的各种问题：情节构思、角色塑造、文笔提升、结构规划等。\
                 请给出具体、有建设性的建议。用中文回复。"
                    .to_string(),
            );
        }
    }

    parts.join("\n\n")
}

pub async fn build_worldview_summary(db: &Db, project_id: &str) -> AppResult<String> {
    #[derive(Debug, Clone, SurrealValue)]
    struct WvRow {
        category: String,
        title: String,
        content: String,
    }

    let entries: Vec<WvRow> = db
        .query("SELECT category, title, content FROM worldview_entry WHERE project = $pid ORDER BY category, title")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Vec<WvRow>>(0)?;

    let lines: Vec<String> = entries
        .iter()
        .map(|e| {
            if e.content.is_empty() {
                format!("- [{}] {}", e.category, e.title)
            } else {
                format!("- [{}] {}：{}", e.category, e.title, e.content)
            }
        })
        .collect();

    Ok(lines.join("\n"))
}

async fn get_project_title(db: &Db, project_id: &str) -> AppResult<String> {
    let result: Option<String> = db
        .query("SELECT VALUE title FROM project WHERE id = $pid")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Option<String>>(0)?;
    Ok(result.unwrap_or_default())
}

async fn get_project_characters(db: &Db, project_id: &str) -> AppResult<Vec<Character>> {
    db.query("SELECT * FROM character WHERE project = $pid ORDER BY name")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Vec<Character>>(0)
        .map_err(Into::into)
}

async fn get_worldview_entries(db: &Db, project_id: &str) -> AppResult<Vec<WorldviewItem>> {
    #[derive(Debug, Clone, SurrealValue)]
    struct Row {
        category: String,
        title: String,
        content: String,
    }

    let rows: Vec<Row> = db
        .query("SELECT category, title, content FROM worldview_entry WHERE project = $pid ORDER BY category, title")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Vec<Row>>(0)?;

    Ok(rows
        .into_iter()
        .map(|r| WorldviewItem {
            category: r.category,
            title: r.title,
            content: r.content,
        })
        .collect())
}

async fn get_recent_beats(db: &Db, session_id: &str) -> AppResult<Vec<BeatSummary>> {
    #[derive(Debug, Clone, SurrealValue)]
    struct Row {
        beat_type: String,
        character_name: String,
        content: String,
        strand: String,
    }

    let rows: Vec<Row> = db
        .query(
            "SELECT beat_type, character_name, content, strand \
             FROM narrative_beat \
             WHERE session = type::record('narrative_session', $sid) \
             ORDER BY sort_order DESC \
             LIMIT 20"
        )
        .bind(("sid", session_id.to_string()))
        .await?
        .take::<Vec<Row>>(0)?;

    let mut beats: Vec<BeatSummary> = rows
        .into_iter()
        .map(|r| BeatSummary {
            beat_type: r.beat_type,
            character_name: r.character_name,
            content: r.content,
            strand: r.strand,
        })
        .collect();
    beats.reverse();
    Ok(beats)
}

async fn get_character_states(db: &Db, session_id: &str) -> AppResult<Vec<CharacterStateView>> {
    #[derive(Debug, Clone, SurrealValue)]
    struct StateRow {
        character_name: String,
        emotion: String,
        location: String,
        knowledge: String,
        physical_state: String,
    }

    let rows: Vec<StateRow> = db
        .query(
            "SELECT in.name AS character_name, emotion, location, knowledge, physical_state \
             FROM character_state \
             WHERE out.session = type::record('narrative_session', $sid) \
             ORDER BY out.sort_order DESC \
             LIMIT 1 BY in"
        )
        .bind(("sid", session_id.to_string()))
        .await?
        .take::<Vec<StateRow>>(0)?;

    Ok(rows
        .into_iter()
        .map(|r| CharacterStateView {
            name: r.character_name,
            emotion: r.emotion,
            location: r.location,
            knowledge: r.knowledge,
            physical_state: r.physical_state,
        })
        .collect())
}

async fn get_strand_context(db: &Db, session_id: &str) -> AppResult<StrandContext> {
    #[derive(Debug, Clone, Deserialize, SurrealValue)]
    struct StrandRow {
        strand: String,
        sort_order: Option<i64>,
    }

    let rows: Vec<StrandRow> = db
        .query(
            "SELECT strand, sort_order FROM narrative_beat \
             WHERE session = type::record('narrative_session', $sid) \
             AND beat_type IN ['narration', 'character_action'] \
             ORDER BY sort_order DESC \
             LIMIT 30"
        )
        .bind(("sid", session_id.to_string()))
        .await?
        .take::<Vec<StrandRow>>(0)?;

    let current_strand = rows.first().map(|r| r.strand.clone()).unwrap_or_else(|| "quest".to_string());

    let mut quest_streak: i64 = 0;
    let mut fire_gap: i64 = 0;
    let mut constellation_gap: i64 = 0;

    for row in &rows {
        match row.strand.as_str() {
            "quest" => {
                quest_streak += 1;
                fire_gap += 1;
                constellation_gap += 1;
            }
            "fire" => {
                quest_streak = 0;
                fire_gap = 0;
                constellation_gap += 1;
            }
            "constellation" => {
                quest_streak = 0;
                fire_gap += 1;
                constellation_gap = 0;
            }
            _ => {}
        }
    }

    Ok(StrandContext {
        current_strand,
        quest_streak,
        fire_gap,
        constellation_gap,
    })
}
