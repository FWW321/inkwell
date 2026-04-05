use rig::client::CompletionClient;
use rig::completion::Prompt;
use rusqlite::{Connection, params};
use serde::Deserialize;

use crate::db::models::AiConfig;
use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelObject>,
}

#[derive(Deserialize)]
struct ModelObject {
    id: String,
}

pub async fn list_models(api_key: &str, base_url: &str) -> AppResult<Vec<String>> {
    if api_key.is_empty() {
        return Err(AppError::Ai("请先配置 API Key".to_string()));
    }

    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .get(&url)
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| AppError::Ai(format!("请求失败: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Ai(format!(
            "获取模型列表失败 ({}): {}",
            status, body
        )));
    }

    let models: ModelsResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Ai(format!("解析响应失败: {}", e)))?;

    let mut ids: Vec<String> = models.data.into_iter().map(|m| m.id).collect();
    ids.sort();
    Ok(ids)
}

pub fn get_config(conn: &Connection) -> AppResult<AiConfig> {
    conn.query_row(
        "SELECT api_key, model, base_url FROM ai_config WHERE id = 1",
        [],
        |row| {
            Ok(AiConfig {
                api_key: row.get(0)?,
                model: row.get(1)?,
                base_url: row.get(2)?,
            })
        },
    )
    .map_err(AppError::Database)
}

pub fn set_config(conn: &Connection, api_key: &str, model: &str, base_url: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE ai_config SET api_key = ?1, model = ?2, base_url = ?3 WHERE id = 1",
        params![api_key, model, base_url],
    )?;
    Ok(())
}

fn build_agent(
    config: &AiConfig,
    preamble: &str,
) -> AppResult<rig::agent::Agent<rig::providers::openai::completion::CompletionModel>> {
    if config.api_key.is_empty() {
        return Err(AppError::Ai("请先在设置中配置 API Key".to_string()));
    }

    let client = rig::providers::openai::CompletionsClient::builder()
        .api_key(&config.api_key)
        .base_url(&config.base_url)
        .build()
        .map_err(|e| AppError::Ai(format!("创建 OpenAI 客户端失败: {}", e)))?;

    let agent = client
        .agent(&config.model)
        .preamble(preamble)
        .temperature(0.8)
        .build();

    Ok(agent)
}

pub async fn continue_writing(
    config: &AiConfig,
    context: &str,
    style: &str,
    length: &str,
) -> AppResult<String> {
    let length_guide = match length {
        "short" => "100-200字",
        "medium" => "300-500字",
        "long" => "600-1000字",
        _ => "300-500字",
    };

    let preamble = format!(
        "你是一位专业的小说创作助手。你的任务是根据已有内容续写故事。\
         请保持与前文一致的风格、语气和叙事视角。\
         续写内容要自然流畅，与前文无缝衔接。\
         用户期望的写作风格：{}。\
         续写长度约{}。\
         直接输出续写内容，不要加任何说明或标记。",
        style, length_guide
    );

    let agent = build_agent(config, &preamble)?;
    agent
        .prompt(context)
        .await
        .map_err(|e| AppError::Ai(format!("AI 请求失败: {}", e)))
}

pub async fn rewrite(
    config: &AiConfig,
    selected_text: &str,
    instruction: &str,
) -> AppResult<String> {
    let preamble = format!(
        "你是一位专业的文字编辑。用户会给你一段文字和一个改写指令，请根据指令改写这段文字。\
         改写指令：{}。\
         直接输出改写后的内容，不要加任何说明或标记。",
        instruction
    );

    let agent = build_agent(config, &preamble)?;
    agent
        .prompt(selected_text)
        .await
        .map_err(|e| AppError::Ai(format!("AI 请求失败: {}", e)))
}

pub async fn polish(config: &AiConfig, selected_text: &str) -> AppResult<String> {
    let preamble = "你是一位专业的文学编辑。请润色用户提供的文字，提升表达质量，使之更加流畅优美。\
                    保持原文的核心意思不变，适当优化用词和句式。\
                    直接输出润色后的内容，不要加任何说明或标记。";

    let agent = build_agent(config, preamble)?;
    agent
        .prompt(selected_text)
        .await
        .map_err(|e| AppError::Ai(format!("AI 请求失败: {}", e)))
}

pub async fn generate_dialogue(
    config: &AiConfig,
    characters: &str,
    scenario: &str,
) -> AppResult<String> {
    let preamble = "你是一位擅长创作对话的小说家。根据提供的角色信息和场景描述，\
                    生成自然生动的角色对话。对话要符合每个角色的性格特点，推动情节发展。\
                    使用中文引号「」包裹对话内容，并标注说话者。\
                    直接输出对话内容，不要加任何说明或标记。";

    let prompt = format!("角色信息：\n{}\n\n场景描述：\n{}", characters, scenario);

    let agent = build_agent(config, preamble)?;
    agent
        .prompt(&*prompt)
        .await
        .map_err(|e| AppError::Ai(format!("AI 请求失败: {}", e)))
}

pub async fn chat(
    config: &AiConfig,
    _project_id: &str,
    _context_type: &str,
    _context_id: &str,
    message: &str,
) -> AppResult<String> {
    let preamble = "你是 Inkwell 写作助手，一位专业的小说创作顾问。\
                    你可以帮助用户解决写作中的各种问题：情节构思、角色塑造、文笔提升、结构规划等。\
                    请给出具体、有建设性的建议。用中文回复。";

    let agent = build_agent(config, preamble)?;
    agent
        .prompt(message)
        .await
        .map_err(|e| AppError::Ai(format!("AI 请求失败: {}", e)))
}
