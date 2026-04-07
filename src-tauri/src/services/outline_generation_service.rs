use crate::db::Store;
use crate::db::models::OutlineNode;
use crate::error::AppResult;
use crate::services::context_service;
use crate::state::Db;
use serde::Deserialize;
use surrealdb::types::ToSql;

#[derive(Deserialize)]
struct ParsedOutlineItem {
    title: String,
    outline: String,
}

pub async fn generate_volume_structure(
    db: &Db,
    http: &reqwest::Client,
    project_id: &str,
    concept: &str,
    agent_id: &str,
) -> AppResult<Vec<OutlineNode>> {
    let agent_config = crate::services::agent_service::get_agent_config(db, agent_id).await?;
    let contract = context_service::build_contract(db, project_id, "").await?;

    let context = build_project_context(&contract);
    let preamble = format!("{}\n\n{}", agent_config.system_prompt, context);

    let prompt = format!(
        "小说概念：{}\n\n请规划整体卷结构。严格按照以下 JSON 格式回复（不要输出任何其他内容）：\n\
        [{{\"title\": \"第一卷：卷名\", \"outline\": \"本卷主要内容概述（50-100字）\"}}]\n\n\
        要求：\n- 根据概念合理规划卷数（通常 3-8 卷）\n- 每卷有清晰的标题和内容概述\n- 卷与卷之间有递进关系",
        concept
    );

    let result = crate::services::ai_service::chat_completion(
        http,
        &agent_config.model_config,
        &preamble,
        &prompt,
        agent_config.temperature,
    )
    .await?;

    let items = parse_outline_items(&result)?;
    let mut nodes = Vec::with_capacity(items.len());

    for (i, item) in items.iter().enumerate() {
        let node = create_volume_node(db, project_id, item, i as i64).await?;
        nodes.push(node);
    }

    Ok(nodes)
}

pub async fn generate_chapter_structure(
    db: &Db,
    http: &reqwest::Client,
    volume_id: &str,
    agent_id: &str,
) -> AppResult<Vec<OutlineNode>> {
    let volume = crate::services::outline_service::get_node(db, volume_id).await?;
    let project_id = volume.project.key.to_sql();
    let agent_config = crate::services::agent_service::get_agent_config(db, agent_id).await?;
    let contract = context_service::build_contract(db, &project_id, "").await?;

    let sibling_volumes = get_sibling_volumes(db, &project_id, None).await?;
    let sibling_context = format_siblings(&sibling_volumes, volume_id);

    let context = build_project_context(&contract);
    let preamble = format!("{}\n\n{}", agent_config.system_prompt, context);

    let prompt = format!(
        "当前卷：{}\n卷纲：{}\n{}\
        请为本卷规划章节结构。严格按照以下 JSON 格式回复（不要输出任何其他内容）：\n\
        [{{\"title\": \"第一章 章名\", \"outline\": \"本章主要内容概述（30-60字）\"}}]\n\n\
        要求：\n- 根据卷纲合理规划章节数（通常 10-30 章）\n- 每章有清晰的标题和内容概述\n- 章节之间有递进和节奏变化",
        volume.title,
        volume
            .content_json
            .get("outline")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        sibling_context
    );

    let result = crate::services::ai_service::chat_completion(
        http,
        &agent_config.model_config,
        &preamble,
        &prompt,
        agent_config.temperature,
    )
    .await?;

    let items = parse_outline_items(&result)?;
    let mut nodes = Vec::with_capacity(items.len());

    for item in items.iter() {
        let node = crate::services::outline_service::create_node(
            db,
            &project_id,
            Some(volume_id),
            "chapter",
            &item.title,
        )
        .await?;

        let content = serde_json::json!({ "outline": item.outline });
        let updated = crate::services::outline_service::update_node(
            db,
            &node.id.key.to_sql(),
            &item.title,
            content,
            0,
            "draft",
        )
        .await?;
        nodes.push(updated);
    }

    Ok(nodes)
}

pub async fn expand_chapter_outline(
    db: &Db,
    http: &reqwest::Client,
    chapter_id: &str,
    agent_id: &str,
) -> AppResult<OutlineNode> {
    let chapter = crate::services::outline_service::get_node(db, chapter_id).await?;
    let project_id = chapter.project.key.to_sql();
    let agent_config = crate::services::agent_service::get_agent_config(db, agent_id).await?;
    let contract = context_service::build_contract(db, &project_id, "").await?;

    let volume_outline = if let Some(ref parent_id) = chapter.parent {
        let parent =
            crate::services::outline_service::get_node(db, &parent_id.key.to_sql()).await?;
        parent
            .content_json
            .get("outline")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    let parent_key = chapter.parent.as_ref().map(|p| p.key.to_sql());
    let sibling_chapters = get_sibling_chapters(db, &project_id, parent_key.as_deref()).await?;
    let sibling_context = format_chapter_siblings(&sibling_chapters, chapter_id);

    let context = build_project_context(&contract);
    let preamble = format!("{}\n\n{}", agent_config.system_prompt, context);

    let prompt = format!(
        "当前章节：{}\n章纲：{}\n所在卷纲：{}\n{}\
        请为本章生成详细大纲。严格按照以下 JSON 格式回复（不要输出任何其他内容）：\n\
        {{\"outline\": \"详细大纲（200-500字，包含场景、关键情节、角色出场、情绪节奏）\"}}",
        chapter.title,
        chapter
            .content_json
            .get("outline")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        volume_outline,
        sibling_context
    );

    let result = crate::services::ai_service::chat_completion(
        http,
        &agent_config.model_config,
        &preamble,
        &prompt,
        agent_config.temperature,
    )
    .await?;

    let parsed: serde_json::Value = crate::services::ai_service::parse_json_response(
        &result,
        "详细大纲解析失败，请重试",
    )?;

    let detailed_outline = parsed["outline"]
        .as_str()
        .ok_or_else(|| crate::error::AppError::Ai("大纲缺少 outline 字段".to_string()))?;

    let content = serde_json::json!({
        "outline": chapter.content_json.get("outline").and_then(|v| v.as_str()).unwrap_or(""),
        "detailed_outline": detailed_outline,
    });

    crate::services::outline_service::update_node(
        db,
        chapter_id,
        &chapter.title,
        content,
        0,
        "draft",
    )
    .await
}

fn build_project_context(contract: &context_service::ContextContract) -> String {
    let mut parts = Vec::with_capacity(3);

    if !contract.project_title.is_empty() {
        parts.push(format!("小说：《{}》", contract.project_title));
    }

    if !contract.characters.is_empty() {
        let chars: Vec<String> = contract
            .characters
            .iter()
            .map(|c| {
                let mut info = format!("- {}（{}）", c.name, c.personality);
                if !c.background.is_empty() {
                    info.push_str(&format!("：{}", c.background));
                }
                info
            })
            .collect();
        parts.push(format!("\n主要角色：\n{}", chars.join("\n")));
    }

    if !contract.worldview.is_empty() {
        let wv: Vec<String> = contract
            .worldview
            .iter()
            .map(|w| format!("- [{}] {}：{}", w.category, w.title, w.content))
            .collect();
        parts.push(format!("\n世界观设定：\n{}", wv.join("\n")));
    }

    parts.join("\n")
}

async fn create_volume_node(
    db: &Db,
    project_id: &str,
    item: &ParsedOutlineItem,
    sort_order: i64,
) -> AppResult<OutlineNode> {
    let content = serde_json::json!({ "outline": item.outline });

    let node: OutlineNode = Store::new(db)
        .content("outline_node")
        .ref_id("project", "project", project_id)
        .field("node_type", "volume")
        .field("title", &item.title)
        .field("sort_order", sort_order)
        .field("content_json", content)
        .field("word_count", 0_i64)
        .field("status", "draft")
        .exec::<OutlineNode>()
        .await?;
    Ok(node)
}

async fn get_sibling_volumes(
    db: &Db,
    project_id: &str,
    _exclude_id: Option<&str>,
) -> AppResult<Vec<OutlineNode>> {
    crate::services::outline_service::list_children(db, project_id, None).await
}

async fn get_sibling_chapters(
    db: &Db,
    project_id: &str,
    volume_id: Option<&str>,
) -> AppResult<Vec<OutlineNode>> {
    crate::services::outline_service::list_children(db, project_id, volume_id).await
}

fn format_siblings(siblings: &[OutlineNode], exclude_id: &str) -> String {
    let items: Vec<String> = siblings
        .iter()
        .filter(|s| s.id.key.to_sql() != exclude_id)
        .map(|s| {
            let outline = s
                .content_json
                .get("outline")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("- {}：{}", s.title, outline)
        })
        .collect();

    if items.is_empty() {
        String::new()
    } else {
        format!("\n已有卷结构：\n{}\n", items.join("\n"))
    }
}

fn format_chapter_siblings(siblings: &[OutlineNode], exclude_id: &str) -> String {
    let items: Vec<String> = siblings
        .iter()
        .filter(|s| s.id.key.to_sql() != exclude_id)
        .map(|s| {
            let outline = s
                .content_json
                .get("outline")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("- {}：{}", s.title, outline)
        })
        .collect();

    if items.is_empty() {
        String::new()
    } else {
        format!("\n已有章节：\n{}\n", items.join("\n"))
    }
}

fn parse_outline_items(response: &str) -> Result<Vec<ParsedOutlineItem>, crate::error::AppError> {
    crate::services::ai_service::parse_json_response(response, "大纲生成结果解析失败，请重试")
}
