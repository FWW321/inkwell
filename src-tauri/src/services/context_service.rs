use crate::error::AppResult;
use crate::state::Db;
use surrealdb::types::{RecordId, SurrealValue};

pub struct ProjectContext {
    pub project_title: String,
    pub characters_summary: String,
    pub worldview_summary: String,
    pub chapter_title: Option<String>,
    pub adjacent_chapters: String,
}

pub async fn build_project_context(
    db: &Db,
    project_id: &str,
    chapter_id: Option<&str>,
) -> AppResult<ProjectContext> {
    let project_title: Option<String> = db
        .query("SELECT VALUE title FROM project WHERE id = $pid")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Option<String>>(0)?;
    let project_title = project_title.unwrap_or_default();

    let characters_summary = build_characters_summary(db, project_id).await?;
    let worldview_summary = build_worldview_summary(db, project_id).await?;

    let (chapter_title, adjacent_chapters) = match chapter_id {
        Some(cid) => (
            get_chapter_title(db, cid).await?,
            get_adjacent_chapters(db, project_id, cid).await?,
        ),
        None => (None, String::new()),
    };

    Ok(ProjectContext {
        project_title,
        characters_summary,
        worldview_summary,
        chapter_title,
        adjacent_chapters,
    })
}

pub fn format_system_prompt(ctx: &ProjectContext, mode: &str) -> String {
    let mut parts = Vec::new();

    parts.push("你是 Inkwell 写作助手，一位专业的小说创作顾问。".to_string());

    if !ctx.project_title.is_empty() {
        parts.push(format!("当前项目：《{}》", ctx.project_title));
    }

    if !ctx.characters_summary.is_empty() {
        parts.push(format!("主要角色：\n{}", ctx.characters_summary));
    }

    if !ctx.worldview_summary.is_empty() {
        parts.push(format!("世界观设定：\n{}", ctx.worldview_summary));
    }

    if let Some(ref title) = ctx.chapter_title {
        parts.push(format!("当前章节：{}", title));
    }

    if !ctx.adjacent_chapters.is_empty() {
        parts.push(format!("章节大纲：\n{}", ctx.adjacent_chapters));
    }

    match mode {
        "continue" => {
            parts.push(
                "你的任务是根据已有内容续写故事。\
                 请保持与前文一致的风格、语气和叙事视角。\
                 续写内容要自然流畅，与前文无缝衔接。\
                 直接输出续写内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        "rewrite" => {
            parts.push(
                "你的任务是改写用户提供的文字。\
                 改写时要保持原文的核心意思，根据用户指令调整风格和表达。\
                 直接输出改写后的内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        "polish" => {
            parts.push(
                "你的任务是润色用户提供的文字，提升表达质量，使之更加流畅优美。\
                 保持原文的核心意思不变，适当优化用词和句式。\
                 直接输出润色后的内容，不要加任何说明或标记。"
                    .to_string(),
            );
        }
        "dialogue" => {
            parts.push(
                "你是一位擅长创作对话的小说家。\
                 根据提供的角色信息和场景描述，生成自然生动的角色对话。\
                 对话要符合每个角色的性格特点，推动情节发展。\
                 使用中文引号「」包裹对话内容，并标注说话者。\
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

async fn build_characters_summary(db: &Db, project_id: &str) -> AppResult<String> {
    #[derive(Debug, Clone, SurrealValue)]
    struct CharRow {
        name: String,
        personality: String,
        description: String,
    }

    let chars: Vec<CharRow> = db
        .query("SELECT name, personality, description FROM character WHERE project = $pid ORDER BY name")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Vec<CharRow>>(0)?;

    let lines: Vec<String> = chars
        .iter()
        .map(|c| {
            let mut info = format!("- {}：{}", c.name, c.personality);
            if !c.description.is_empty() {
                info.push_str(&format!("（{}）", c.description));
            }
            info
        })
        .collect();

    Ok(lines.join("\n"))
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

async fn get_chapter_title(db: &Db, chapter_id: &str) -> AppResult<Option<String>> {
    let result: Option<String> = db
        .query("SELECT VALUE title FROM outline_node WHERE id = $id")
        .bind(("id", RecordId::new("outline_node", chapter_id)))
        .await?
        .take::<Option<String>>(0)?;

    Ok(result)
}

async fn get_adjacent_chapters(
    db: &Db,
    project_id: &str,
    _chapter_id: &str,
) -> AppResult<String> {
    #[derive(Debug, Clone, SurrealValue)]
    struct ChapterRow {
        title: String,
    }

    let chapters: Vec<ChapterRow> = db
        .query(
             "SELECT title, sort_order FROM outline_node \
              WHERE project = $pid AND node_type = 'chapter' \
              ORDER BY sort_order \
             LIMIT 10"
        )
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .take::<Vec<ChapterRow>>(0)?;

    let titles: Vec<String> = chapters.into_iter().map(|c| c.title).collect();
    Ok(titles.join(" → "))
}
