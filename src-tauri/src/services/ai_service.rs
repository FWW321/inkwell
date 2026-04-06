use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use tauri::ipc::Channel;

use crate::db::models::{AiAgent, AiAgentWithModelName, AiConfig};
use crate::error::{AppError, AppResult};
use crate::services::context_service;
use crate::state::Db;
use surrealdb::types::{RecordId, SurrealValue, ToSql, Value};

#[derive(serde::Serialize, Clone)]
pub struct StreamChunk {
    pub text: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelObject>,
}

#[derive(Deserialize)]
struct ModelObject {
    id: String,
}

fn is_unique_error(e: &surrealdb::Error) -> bool {
    let msg = e.to_string();
    msg.contains("UNIQUE") || msg.contains("unique")
}

fn is_record_error(e: &surrealdb::Error) -> bool {
    e.to_string().contains("record")
}

fn get_created_id(v: &Value) -> String {
    match v {
        Value::Object(map) => {
            if let Some(Value::RecordId(rid)) = map.get("id") {
                return rid.key.to_sql();
            }
        }
        _ => {}
    }
    String::new()
}

async fn promote_default(db: &Db, table: &str) -> AppResult<()> {
    let first: Option<RecordId> = db
        .query("SELECT id, created_at FROM type::table($table) ORDER BY created_at ASC LIMIT 1")
        .bind(("table", table.to_string()))
        .await?
        .take::<Option<RecordId>>(0)?;
    if let Some(first_id) = first {
        db.query("UPDATE type::record($id) SET is_default = true")
            .bind(("id", first_id))
            .await?;
    }
    Ok(())
}

pub async fn fetch_available_models(api_key: &str, base_url: &str) -> AppResult<Vec<String>> {
    if api_key.is_empty() {
        return Err(AppError::Ai("请先配置 API Key".to_string()));
    }

    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = Client::new()
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

pub async fn list_models(db: &Db) -> AppResult<Vec<AiConfig>> {
    db.query("SELECT * FROM ai_model ORDER BY is_default DESC, created_at ASC")
        .await?.take::<Vec<AiConfig>>(0).map_err(Into::into)
}

pub async fn get_default_config(db: &Db) -> AppResult<AiConfig> {
    let result: Option<AiConfig> = db
        .query("SELECT * FROM ai_model WHERE is_default = true LIMIT 1")
        .await?
        .take::<Option<AiConfig>>(0)?;

    match result {
        Some(c) => Ok(c),
        None => list_models(db).await?
            .into_iter()
            .next()
            .ok_or(AppError::Ai("请先配置 AI 模型".to_string())),
    }
}

pub async fn get_model(db: &Db, id: &str) -> AppResult<AiConfig> {
    db.select(("ai_model", id)).await?
        .ok_or_else(|| AppError::NotFound("模型不存在".to_string()))
}

pub async fn list_agents(db: &Db) -> AppResult<Vec<AiAgentWithModelName>> {
    db.query("SELECT *, model.name AS model_name FROM ai_agent ORDER BY is_default DESC, created_at ASC")
        .await?.take::<Vec<AiAgentWithModelName>>(0).map_err(Into::into)
}

pub async fn get_default_agent(db: &Db) -> AppResult<AiAgentWithModelName> {
    let result: Option<AiAgentWithModelName> = db
        .query("SELECT *, model.name AS model_name FROM ai_agent WHERE is_default = true LIMIT 1")
        .await?
        .take::<Option<AiAgentWithModelName>>(0)?;

    match result {
        Some(a) => Ok(a),
        None => list_agents(db).await?
            .into_iter()
            .next()
            .ok_or(AppError::Ai("请先配置 AI 助手".to_string())),
    }
}

async fn get_agent(db: &Db, id: &str) -> AppResult<AiAgentWithModelName> {
    let records: Vec<AiAgentWithModelName> = db
        .query("SELECT *, model.name AS model_name FROM ai_agent WHERE id = $id")
        .bind(("id", RecordId::new("ai_agent", id)))
        .await?
        .take::<Vec<AiAgentWithModelName>>(0)?;
    records.into_iter().next()
        .ok_or_else(|| AppError::NotFound("助手不存在".to_string()))
}

pub async fn create_agent(
    db: &Db,
    name: &str,
    model_id: &str,
    system_prompt: &str,
) -> AppResult<AiAgentWithModelName> {
    let count: Option<i64> = db
        .query("SELECT VALUE count() FROM ai_agent")
        .await?
        .take::<Option<i64>>(0)?;
    let is_default = count.map(|c| c == 0).unwrap_or(true);

    let result: Result<Option<Value>, surrealdb::Error> = db
        .query(
            "CREATE ai_agent CONTENT { \
             name: $name, \
             model: type::record('ai_model', $mid), \
             system_prompt: $prompt, \
             is_default: $is_default \
             }"
        )
        .bind(("name", name.to_string()))
        .bind(("mid", model_id.to_string()))
        .bind(("prompt", system_prompt.to_string()))
        .bind(("is_default", is_default))
        .await?
        .take::<Option<Value>>(0);

    match result {
        Ok(Some(v)) => {
            let id_str = get_created_id(&v);
            get_agent(db, &id_str).await
        }
        Ok(None) => Err(AppError::Internal("create agent failed".into())),
        Err(e) if is_unique_error(&e) => Err(AppError::Ai("已存在同名的助手".to_string())),
        Err(e) if is_record_error(&e) => Err(AppError::Ai("关联的模型不存在".to_string())),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub async fn update_agent(
    db: &Db,
    id: &str,
    name: &str,
    model_id: &str,
    system_prompt: &str,
) -> AppResult<AiAgentWithModelName> {
    let result: Result<Option<Value>, surrealdb::Error> = db
        .query(
            "UPDATE type::record($id) MERGE { \
             name: $name, \
             model: type::record('ai_model', $mid), \
             system_prompt: $prompt \
             }"
        )
        .bind(("id", RecordId::new("ai_agent", id)))
        .bind(("name", name.to_string()))
        .bind(("mid", model_id.to_string()))
        .bind(("prompt", system_prompt.to_string()))
        .await?
        .take::<Option<Value>>(0);

    match result {
        Ok(Some(_)) => get_agent(db, id).await,
        Ok(None) => Err(AppError::NotFound("助手不存在".to_string())),
        Err(e) if is_unique_error(&e) => Err(AppError::Ai("已存在同名的助手".to_string())),
        Err(e) if is_record_error(&e) => Err(AppError::Ai("关联的模型不存在".to_string())),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub async fn delete_agent(db: &Db, id: &str) -> AppResult<()> {
    let record: Option<AiAgent> = db.select(("ai_agent", id)).await?;
    let was_default = record
        .map(|a| a.is_default)
        .ok_or_else(|| AppError::NotFound("助手不存在".to_string()))?;

    let _: Option<AiAgent> = db.delete(("ai_agent", id)).await?;

    if was_default {
        promote_default(db, "ai_agent").await?;
    }
    Ok(())
}

pub async fn set_default_agent(db: &Db, id: &str) -> AppResult<()> {
    db.query("UPDATE ai_agent SET is_default = false WHERE is_default = true")
        .await?;
    db.query("UPDATE type::record($id) SET is_default = true")
        .bind(("id", RecordId::new("ai_agent", id)))
        .await?;
    Ok(())
}

pub async fn create_model(
    db: &Db,
    name: &str,
    api_key: &str,
    model: &str,
    base_url: &str,
) -> AppResult<AiConfig> {
    let count: Option<i64> = db
        .query("SELECT VALUE count() FROM ai_model")
        .await?
        .take::<Option<i64>>(0)?;
    let is_default = count.map(|c| c == 0).unwrap_or(true);

    let data = serde_json::json!({
        "name": name,
        "api_key": api_key,
        "model": model,
        "base_url": base_url,
        "is_default": is_default,
    });

    let result: Result<Option<AiConfig>, surrealdb::Error> =
        db.create("ai_model").content(data).await;

    match result {
        Ok(Some(cfg)) => Ok(cfg),
        Ok(None) => Err(AppError::Internal("create model failed".into())),
        Err(e) if is_unique_error(&e) => Err(AppError::Ai("已存在同名配置".to_string())),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub async fn update_model(
    db: &Db,
    id: &str,
    name: &str,
    api_key: &str,
    model: &str,
    base_url: &str,
) -> AppResult<AiConfig> {
    let data = serde_json::json!({
        "name": name,
        "api_key": api_key,
        "model": model,
        "base_url": base_url,
    });

    let result: Result<Option<AiConfig>, surrealdb::Error> =
        db.update(("ai_model", id)).merge(data).await;

    match result {
        Ok(Some(_)) => get_model(db, id).await,
        Ok(None) => Err(AppError::NotFound("模型不存在".to_string())),
        Err(e) if is_unique_error(&e) => Err(AppError::Ai("已存在同名配置".to_string())),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub async fn delete_model(db: &Db, id: &str) -> AppResult<()> {
    let record: Option<AiConfig> = db.select(("ai_model", id)).await?;
    let was_default = record
        .map(|c| c.is_default)
        .ok_or_else(|| AppError::NotFound("模型不存在".to_string()))?;

    let _: Option<AiConfig> = db.delete(("ai_model", id)).await?;

    if was_default {
        promote_default(db, "ai_model").await?;
    }
    Ok(())
}

pub async fn set_default_model(db: &Db, id: &str) -> AppResult<()> {
    db.query("UPDATE ai_model SET is_default = false WHERE is_default = true")
        .await?;
    db.query("UPDATE type::record($id) SET is_default = true")
        .bind(("id", RecordId::new("ai_model", id)))
        .await?;
    Ok(())
}

pub(crate) async fn chat_completion(
    config: &AiConfig,
    system_prompt: &str,
    user_message: &str,
    temperature: f32,
) -> AppResult<String> {
    if config.api_key.is_empty() {
        return Err(AppError::Ai("请先在设置中配置 API Key".to_string()));
    }

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_message },
        ],
        "temperature": temperature,
    });

    let resp = Client::new()
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Ai(format!("请求失败: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        return Err(AppError::Ai(format!(
            "AI 请求失败 ({}): {}",
            status, error_body
        )));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Ai(format!("解析响应失败: {}", e)))?;

    data["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Ai("AI 返回内容为空".to_string()))
}

pub async fn stream_ai(
    config: &AiConfig,
    preamble: &str,
    prompt_text: &str,
    on_chunk: &Channel<StreamChunk>,
) -> AppResult<()> {
    if config.api_key.is_empty() {
        return Err(AppError::Ai("请先在设置中配置 API Key".to_string()));
    }

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            { "role": "system", "content": preamble },
            { "role": "user", "content": prompt_text },
        ],
        "temperature": 0.8,
        "stream": true,
    });

    let resp = Client::new()
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Ai(format!("请求失败: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        let _ = on_chunk.send(StreamChunk {
            text: format!("\n\n[错误: {}]", error_body),
            done: true,
            reasoning: None,
        });
        return Err(AppError::Ai(format!(
            "AI 请求失败 ({}): {}",
            status, error_body
        )));
    }

    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| AppError::Ai(format!("流式读取错误: {}", e)))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find("\n\n") {
            let event = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            for line in event.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        let _ = on_chunk.send(StreamChunk {
                            text: String::new(),
                            done: true,
                            reasoning: None,
                        });
                        return Ok(());
                    }

                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                        let delta = &parsed["choices"][0]["delta"];

                        if let Some(text) = delta["content"].as_str() {
                            if !text.is_empty() {
                                let _ = on_chunk.send(StreamChunk {
                                    text: text.to_string(),
                                    done: false,
                                    reasoning: None,
                                });
                            }
                        }

                        if let Some(reasoning) = delta["reasoning_content"].as_str() {
                            if !reasoning.is_empty() {
                                let _ = on_chunk.send(StreamChunk {
                                    text: String::new(),
                                    done: false,
                                    reasoning: Some(reasoning.to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = on_chunk.send(StreamChunk {
        text: String::new(),
        done: true,
        reasoning: None,
    });

    Ok(())
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

    chat_completion(config, &preamble, context, 0.8).await
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

    chat_completion(config, &preamble, selected_text, 0.8).await
}

pub async fn polish(config: &AiConfig, selected_text: &str) -> AppResult<String> {
    let preamble = "你是一位专业的文学编辑。请润色用户提供的文字，提升表达质量，使之更加流畅优美。\
                    保持原文的核心意思不变，适当优化用词和句式。\
                    直接输出润色后的内容，不要加任何说明或标记。";

    chat_completion(config, preamble, selected_text, 0.8).await
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

    chat_completion(config, preamble, &prompt, 0.8).await
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

    chat_completion(config, preamble, message, 0.8).await
}

pub async fn save_chat_message(
    db: &Db,
    project_id: &str,
    role: &str,
    content: &str,
) -> AppResult<()> {
    db.query(
        "CREATE type::record($table) CONTENT { project: type::record('project', $pid), role: $role, content: $content }"
    )
    .bind(("table", "ai_message"))
    .bind(("pid", project_id.to_string()))
    .bind(("role", role.to_string()))
    .bind(("content", content.to_string()))
    .await?;
    Ok(())
}

#[derive(Debug, Clone, SurrealValue)]
pub struct ChatMessageRow {
    pub role: String,
    pub content: String,
}

pub async fn get_chat_history(
    db: &Db,
    project_id: &str,
    limit: usize,
) -> AppResult<Vec<(String, String)>> {
    let records: Vec<ChatMessageRow> = db
        .query("SELECT role, content, created_at FROM ai_message WHERE project = $pid ORDER BY created_at DESC LIMIT $lim")
        .bind(("pid", RecordId::new("project", project_id)))
        .bind(("lim", limit as i64))
        .await?
        .take::<Vec<ChatMessageRow>>(0)?;

    let mut rows: Vec<(String, String)> = records
        .into_iter()
        .map(|r| (r.role, r.content))
        .collect();
    rows.reverse();
    Ok(rows)
}

pub async fn clear_chat_history(db: &Db, project_id: &str) -> AppResult<()> {
    db.query("DELETE FROM ai_message WHERE project = $pid")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?;
    Ok(())
}

pub async fn ai_stream_with_context(
    db: &Db,
    config: &AiConfig,
    project_id: &str,
    chapter_id: Option<&str>,
    mode: &str,
    user_text: &str,
    style: Option<&str>,
    length: Option<&str>,
    on_chunk: &Channel<StreamChunk>,
) -> AppResult<()> {
    let ctx = context_service::build_project_context(db, project_id, chapter_id).await?;

    let mut preamble = context_service::format_system_prompt(&ctx, mode);

    if let Some(s) = style {
        preamble.push_str(&format!("\n\n用户期望的写作风格：{}。", s));
    }
    if let Some(l) = length {
        let guide = match l {
            "short" => "100-200字",
            "medium" => "300-500字",
            "long" => "600-1000字",
            _ => "300-500字",
        };
        preamble.push_str(&format!("\n\n续写长度约{}。", guide));
    }

    stream_ai(config, &preamble, user_text, on_chunk).await
}
