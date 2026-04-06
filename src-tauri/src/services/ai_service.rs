use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use tauri::ipc::Channel;

use crate::db::created_id;
use crate::db::models::{AiAgent, AiAgentWithModelName, AiConfig};
use crate::error::{AppError, AppResult};
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

async fn promote_default(db: &Db, table: &str) -> AppResult<()> {
    let first: Option<RecordId> = db
        .query("SELECT id, created_at FROM type::table($table) ORDER BY created_at ASC LIMIT 1")
        .bind(("table", table.to_string()))
        .await?
        .check()?
        .take::<Option<RecordId>>(0)?;
    if let Some(first_id) = first {
        db.query("UPDATE type::record($id) SET is_default = true")
            .bind(("id", first_id))
            .await?
            .check()?;
    }
    Ok(())
}

pub async fn fetch_available_models(
    http: &Client,
    api_key: &str,
    base_url: &str,
) -> AppResult<Vec<String>> {
    if api_key.is_empty() {
        return Err(AppError::Ai("请先配置 API Key".to_string()));
    }

    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = http
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
        .await?
        .check()?
        .take::<Vec<AiConfig>>(0)
        .map_err(Into::into)
}

pub async fn get_default_config(db: &Db) -> AppResult<AiConfig> {
    let result: Option<AiConfig> = db
        .query("SELECT * FROM ai_model WHERE is_default = true LIMIT 1")
        .await?
        .check()?
        .take::<Option<AiConfig>>(0)?;

    match result {
        Some(c) => Ok(c),
        None => list_models(db)
            .await?
            .into_iter()
            .next()
            .ok_or(AppError::Ai("请先配置 AI 模型".to_string())),
    }
}

pub async fn get_model(db: &Db, id: &str) -> AppResult<AiConfig> {
    db.select(("ai_model", id))
        .await?
        .ok_or_else(|| AppError::NotFound("模型不存在".to_string()))
}

const AGENT_LIST_SQL: &str = "SELECT id, name, IF model != NONE { record::id(model) } ELSE { NONE } AS model_id, model.name AS model_name, system_prompt, temperature, is_default, created_at FROM ai_agent ORDER BY is_default DESC, created_at ASC FETCH model";

pub async fn list_agents(db: &Db) -> AppResult<Vec<AiAgentWithModelName>> {
    db.query(AGENT_LIST_SQL)
        .await?
        .check()?
        .take::<Vec<AiAgentWithModelName>>(0)
        .map_err(Into::into)
}

pub async fn get_default_agent(db: &Db) -> AppResult<AiAgentWithModelName> {
    let sql = "SELECT id, name, IF model != NONE { record::id(model) } ELSE { NONE } AS model_id, model.name AS model_name, system_prompt, temperature, is_default, created_at FROM ai_agent WHERE is_default = true LIMIT 1 FETCH model";
    let result: Option<AiAgentWithModelName> = db
        .query(sql)
        .await?
        .check()?
        .take::<Option<AiAgentWithModelName>>(0)?;

    match result {
        Some(a) => Ok(a),
        None => list_agents(db)
            .await?
            .into_iter()
            .next()
            .ok_or(AppError::Ai("请先配置 AI 助手".to_string())),
    }
}

pub async fn get_agent(db: &Db, id: &str) -> AppResult<AiAgentWithModelName> {
    let sql = "SELECT id, name, IF model != NONE { record::id(model) } ELSE { NONE } AS model_id, model.name AS model_name, system_prompt, temperature, is_default, created_at FROM ai_agent WHERE id = $id FETCH model";
    let records: Vec<AiAgentWithModelName> = db
        .query(sql)
        .bind(("id", RecordId::new("ai_agent", id)))
        .await?
        .check()?
        .take::<Vec<AiAgentWithModelName>>(0)?;
    records
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("助手不存在".to_string()))
}

pub async fn create_agent(
    db: &Db,
    name: &str,
    model_id: Option<&str>,
    system_prompt: &str,
    temperature: f32,
) -> AppResult<AiAgentWithModelName> {
    let count: Option<i64> = db
        .query("SELECT VALUE count() FROM [SELECT * FROM ai_agent]")
        .await?
        .check()?
        .take::<Option<i64>>(0)?;
    let is_default = count.map(|c| c == 0).unwrap_or(true);

    let result: Result<Option<Value>, surrealdb::Error> = match model_id {
        Some(mid) => db
            .query(
                "CREATE ai_agent CONTENT { \
                 name: $name, \
                 model: type::record('ai_model', $mid), \
                 system_prompt: $prompt, \
                 temperature: $temp, \
                 is_default: $is_default \
                 }",
            )
            .bind(("name", name.to_string()))
            .bind(("mid", mid.to_string()))
            .bind(("prompt", system_prompt.to_string()))
            .bind(("temp", temperature))
            .bind(("is_default", is_default))
            .await?
            .check()?
            .take::<Option<Value>>(0),
        None => db
            .query(
                "CREATE ai_agent CONTENT { \
                 name: $name, \
                 system_prompt: $prompt, \
                 temperature: $temp, \
                 is_default: $is_default \
                 }",
            )
            .bind(("name", name.to_string()))
            .bind(("prompt", system_prompt.to_string()))
            .bind(("temp", temperature))
            .bind(("is_default", is_default))
            .await?
            .check()?
            .take::<Option<Value>>(0),
    };

    match result {
        Ok(Some(v)) => {
            let id_str = created_id(&v)?;
            get_agent(db, &id_str).await
        }
        Ok(None) => Err(AppError::Internal(anyhow::anyhow!("create agent failed"))),
        Err(e) if is_unique_error(&e) => Err(AppError::Ai("已存在同名的助手".to_string())),
        Err(e) if is_record_error(&e) => Err(AppError::Ai("关联的模型不存在".to_string())),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub async fn update_agent(
    db: &Db,
    id: &str,
    name: &str,
    model_id: Option<&str>,
    system_prompt: &str,
    temperature: f32,
) -> AppResult<AiAgentWithModelName> {
    let result: Result<Option<Value>, surrealdb::Error> = match model_id {
        Some(mid) => db
            .query(
                "UPDATE type::record($id) MERGE { \
                 name: $name, \
                 model: type::record('ai_model', $mid), \
                 system_prompt: $prompt, \
                 temperature: $temp \
                 }",
            )
            .bind(("id", RecordId::new("ai_agent", id)))
            .bind(("name", name.to_string()))
            .bind(("mid", mid.to_string()))
            .bind(("prompt", system_prompt.to_string()))
            .bind(("temp", temperature))
            .await?
            .check()?
            .take::<Option<Value>>(0),
        None => db
            .query(
                "UPDATE type::record($id) MERGE { \
                 name: $name, \
                 model: NONE, \
                 system_prompt: $prompt, \
                 temperature: $temp \
                 }",
            )
            .bind(("id", RecordId::new("ai_agent", id)))
            .bind(("name", name.to_string()))
            .bind(("prompt", system_prompt.to_string()))
            .bind(("temp", temperature))
            .await?
            .check()?
            .take::<Option<Value>>(0),
    };

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
    db.query(
        "BEGIN; \
         UPDATE ai_agent SET is_default = false WHERE is_default = true; \
         UPDATE type::record($id) SET is_default = true; \
         COMMIT",
    )
    .bind(("id", RecordId::new("ai_agent", id)))
    .await?
    .check()?;
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
        .query("SELECT VALUE count() FROM [SELECT * FROM ai_model]")
        .await?
        .check()?
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
        Ok(None) => Err(AppError::Internal(anyhow::anyhow!("create model failed"))),
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
    db.query(
        "BEGIN; \
         UPDATE ai_model SET is_default = false WHERE is_default = true; \
         UPDATE type::record($id) SET is_default = true; \
         COMMIT",
    )
    .bind(("id", RecordId::new("ai_model", id)))
    .await?
    .check()?;
    Ok(())
}

pub async fn chat_completion(
    http: &Client,
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

    let resp = http
        .post(&url)
        .bearer_auth(&config.api_key)
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

    data.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Ai("AI 返回内容为空".to_string()))
}

pub async fn stream_ai(
    http: &Client,
    config: &AiConfig,
    preamble: &str,
    prompt_text: &str,
    temperature: f32,
    on_chunk: &Channel<StreamChunk>,
) -> AppResult<()> {
    stream_ai_with_callback(http, config, preamble, prompt_text, temperature, |chunk| {
        let _ = on_chunk.send(chunk);
    })
    .await
}

pub async fn stream_ai_with_callback<F>(
    http: &Client,
    config: &AiConfig,
    preamble: &str,
    prompt_text: &str,
    temperature: f32,
    mut on_chunk: F,
) -> AppResult<()>
where
    F: FnMut(StreamChunk) + Send,
{
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
        "temperature": temperature,
        "stream": true,
    });

    let resp = http
        .post(&url)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Ai(format!("请求失败: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        on_chunk(StreamChunk {
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
                        on_chunk(StreamChunk {
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
                                on_chunk(StreamChunk {
                                    text: text.to_string(),
                                    done: false,
                                    reasoning: None,
                                });
                            }
                        }

                        if let Some(reasoning) = delta["reasoning_content"].as_str() {
                            if !reasoning.is_empty() {
                                on_chunk(StreamChunk {
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

    on_chunk(StreamChunk {
        text: String::new(),
        done: true,
        reasoning: None,
    });

    Ok(())
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
    .await?
    .check()?;
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
        .check()?
        .take::<Vec<ChatMessageRow>>(0)?;

    let mut rows: Vec<(String, String)> =
        records.into_iter().map(|r| (r.role, r.content)).collect();
    rows.reverse();
    Ok(rows)
}

pub async fn clear_chat_history(db: &Db, project_id: &str) -> AppResult<()> {
    db.query("DELETE FROM ai_message WHERE project = $pid")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .check()?;
    Ok(())
}
