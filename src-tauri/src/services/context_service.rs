use rusqlite::{Connection, params};

use crate::error::AppResult;

pub struct ProjectContext {
    pub project_title: String,
    pub characters_summary: String,
    pub worldview_summary: String,
    pub chapter_title: Option<String>,
    pub adjacent_chapters: String,
}

pub fn build_project_context(
    conn: &Connection,
    project_id: &str,
    chapter_id: Option<&str>,
) -> AppResult<ProjectContext> {
    let project_title = conn
        .query_row("SELECT title FROM projects WHERE id = ?1", params![project_id], |row| {
            row.get::<_, String>(0)
        })
        .unwrap_or_default();

    let characters_summary = build_characters_summary(conn, project_id)?;
    let worldview_summary = build_worldview_summary(conn, project_id)?;

    let (chapter_title, adjacent_chapters) = match chapter_id {
        Some(cid) => (
            get_chapter_title(conn, cid)?,
            get_adjacent_chapters(conn, project_id, cid)?,
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

fn build_characters_summary(conn: &Connection, project_id: &str) -> AppResult<String> {
    let mut stmt = conn.prepare(
        "SELECT name, personality, description FROM characters WHERE project_id = ?1 ORDER BY name",
    )?;

    let chars: Vec<String> = stmt
        .query_map(params![project_id], |row| {
            let name: String = row.get(0)?;
            let personality: String = row.get(1)?;
            let description: String = row.get(2)?;
            Ok((name, personality, description))
        })?
        .filter_map(|r| r.ok())
        .map(|(name, personality, description)| {
            let mut info = format!("- {}：{}", name, personality);
            if !description.is_empty() {
                info.push_str(&format!("（{}）", description));
            }
            info
        })
        .collect();

    Ok(chars.join("\n"))
}

fn build_worldview_summary(conn: &Connection, project_id: &str) -> AppResult<String> {
    let mut stmt = conn.prepare(
        "SELECT category, title, content FROM worldview_entries WHERE project_id = ?1 ORDER BY category, title",
    )?;

    let entries: Vec<String> = stmt
        .query_map(params![project_id], |row| {
            let category: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            Ok((category, title, content))
        })?
        .filter_map(|r| r.ok())
        .map(|(category, title, content)| {
            if content.is_empty() {
                format!("- [{}] {}", category, title)
            } else {
                format!("- [{}] {}：{}", category, title, content)
            }
        })
        .collect();

    Ok(entries.join("\n"))
}

fn get_chapter_title(conn: &Connection, chapter_id: &str) -> AppResult<Option<String>> {
    let result = conn.query_row(
        "SELECT title FROM outline_nodes WHERE id = ?1",
        params![chapter_id],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(title) => Ok(Some(title)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn get_adjacent_chapters(
    conn: &Connection,
    project_id: &str,
    chapter_id: &str,
) -> AppResult<String> {
    let current: Option<(i64, Option<String>)> = conn
        .query_row(
            "SELECT sort_order, parent_id FROM outline_nodes WHERE id = ?1",
            params![chapter_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .ok();

    let (_sort_order, parent_id) = match current {
        Some((s, p)) => (s, p),
        None => return Ok(String::new()),
    };

    let mut stmt = conn.prepare(
        "SELECT title, node_type FROM outline_nodes WHERE project_id = ?1 AND (parent_id IS ?2 OR parent_id = ?2) AND node_type = 'chapter' ORDER BY sort_order LIMIT 10",
    )?;

    let chapters: Vec<String> = stmt
        .query_map(params![project_id, parent_id], |row| {
            let title: String = row.get(0)?;
            let node_type: String = row.get(1)?;
            Ok((title, node_type))
        })?
        .filter_map(|r| r.ok())
        .map(|(title, _)| title)
        .collect();

    Ok(chapters.join(" → "))
}
