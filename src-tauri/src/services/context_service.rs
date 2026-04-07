use crate::db::Store;
use crate::db::models::Character;
use crate::error::AppResult;
use crate::state::Db;
use surrealdb::types::SurrealValue;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextContract {
    pub project_title: String,
    pub characters: Vec<CharacterContext>,
    pub worldview: Vec<WorldviewItem>,
    pub recent_beats: Vec<BeatSummary>,
    pub character_states: Vec<CharacterStateView>,
    pub strand_context: StrandContext,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CharacterContext {
    pub name: String,
    pub personality: String,
    pub description: String,
    pub background: String,
    pub state: Option<CharacterStateView>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorldviewItem {
    pub category: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BeatSummary {
    pub beat_type: String,
    pub character_name: String,
    pub content: String,
    pub strand: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CharacterStateView {
    pub name: String,
    pub emotion: String,
    pub location: String,
    pub knowledge: String,
    pub physical_state: String,
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct StrandContext {
    pub current_strand: String,
    pub quest_streak: i64,
    pub fire_gap: i64,
    pub constellation_gap: i64,
}

pub fn beat_label(beat: &BeatSummary) -> std::borrow::Cow<'_, str> {
    match beat.beat_type.as_str() {
        "narration" => "叙事".into(),
        "character_action" => beat.character_name.as_str().into(),
        "scene_change" => "场景切换".into(),
        "author_intervention" => "作者指令".into(),
        _ => beat.character_name.as_str().into(),
    }
}

pub async fn build_contract(
    db: &Db,
    project_id: &str,
    session_id: &str,
) -> AppResult<ContextContract> {
    let (project_title, characters, worldview, recent_beats, character_states, strand_context) = tokio::join!(
        get_project_title(db, project_id),
        get_project_characters(db, project_id),
        get_worldview_entries(db, project_id),
        get_recent_beats(db, session_id),
        get_character_states(db, session_id),
        get_strand_context(db, session_id),
    );

    let project_title = project_title?;
    let characters = characters?;
    let worldview = worldview?;
    let recent_beats = recent_beats?;
    let character_states = character_states?;
    let strand_context = strand_context?;

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

pub fn build_narrative_context(
    contract: &ContextContract,
    scene: &str,
    atmosphere: &str,
) -> String {
    let mut parts = Vec::with_capacity(6);

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
            .map(|b| format!("[{}] {}", beat_label(b), b.content))
            .collect();
        parts.push(format!("\n前情提要：\n{}", beats.join("\n")));
    }

    parts.join("\n")
}

pub fn build_character_context(
    contract: &ContextContract,
    character: &Character,
    scene: &str,
) -> String {
    let mut parts = Vec::with_capacity(6);

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

    if let Some(self_state) = contract
        .character_states
        .iter()
        .find(|s| s.name == character.name)
    {
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
            .map(|b| format!("[{}] {}", beat_label(b), b.content))
            .collect();
        parts.push(format!("\n最近发生的事：\n{}", beats.join("\n")));
    }

    parts.join("\n")
}

pub fn build_editor_context(contract: &ContextContract) -> String {
    let mut parts = Vec::with_capacity(3);

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

    parts.join("\n\n")
}

pub async fn get_project_characters(db: &Db, project_id: &str) -> AppResult<Vec<Character>> {
    Store::new(db)
        .find("character")
        .filter_ref("project", "project", project_id)
        .order("name")
        .all()
        .await
}

async fn get_project_title(db: &Db, project_id: &str) -> AppResult<String> {
    Store::new(db)
        .find("project")
        .project("VALUE title")
        .filter_ref("id", "project", project_id)
        .one()
        .await
}

async fn get_worldview_entries(db: &Db, project_id: &str) -> AppResult<Vec<WorldviewItem>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
        category: String,
        title: String,
        content: String,
    }

    let rows: Vec<Row> = Store::new(db)
        .find("worldview_entry")
        .project("category, title, content")
        .filter_ref("project", "project", project_id)
        .order("category, title")
        .all()
        .await?;

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
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
        beat_type: String,
        character_name: String,
        content: String,
        strand: String,
    }

    let rows: Vec<Row> = Store::new(db)
        .find("narrative_beat")
        .project("beat_type, character_name, content, strand")
        .filter_ref("session", "narrative_session", session_id)
        .order("sort_order DESC")
        .limit(20)
        .all()
        .await?;

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
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
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
             LIMIT 1 BY in",
        )
        .bind(("sid", session_id.to_string()))
        .await?
        .check()?
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
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct StrandRow {
        strand: String,
        #[serde(rename = "sort_order")]
        sort_order: Option<i64>,
    }

    let rows: Vec<StrandRow> = db
        .query(
            "SELECT strand, sort_order FROM narrative_beat \
             WHERE session = type::record('narrative_session', $sid) \
             AND beat_type IN ['narration', 'character_action'] \
             ORDER BY sort_order DESC \
             LIMIT 30",
        )
        .bind(("sid", session_id.to_string()))
        .await?
        .check()?
        .take::<Vec<StrandRow>>(0)?;

    let current_strand = rows
        .first()
        .map(|r| r.strand.clone())
        .unwrap_or_else(|| "quest".to_string());

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
