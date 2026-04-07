use crate::db::Store;
use crate::db::models::WritingReview;
use crate::error::{AppError, AppResult};
use crate::services::context_service;
use crate::state::Db;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub severity: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AggregateReview {
    pub dimensions: Vec<DimensionResult>,
    pub overall_score: f32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DimensionResult {
    pub dimension: String,
    pub score: f32,
    pub passed: bool,
    pub issues: Vec<ReviewIssue>,
    pub summary: String,
}

const DIMENSIONS: &[&str] = &["consistency", "pacing", "reader_pull"];

pub async fn review_beat(
    db: &crate::state::Db,
    http: &reqwest::Client,
    session_id: &str,
    beat_id: &str,
    beat_content: &str,
    beat_type: &str,
    agent_id: Option<&str>,
) -> AppResult<AggregateReview> {
    let agent_config = crate::services::agent_service::resolve_agent(db, agent_id, "质量审查", None).await?;

    let contract =
        context_service::build_contract(db, &get_project_id(db, session_id).await?, session_id)
            .await?;

    let mut handles = Vec::with_capacity(DIMENSIONS.len());
    for &dimension in DIMENSIONS {
        let prompt = build_review_prompt(dimension, beat_content, beat_type, &contract);
        let model_config = agent_config.model_config.clone();
        let system_prompt = agent_config.system_prompt.clone();
        let temperature = agent_config.temperature;
        let http = http.clone();

        handles.push(tokio::spawn(async move {
            let result = crate::services::ai_service::chat_completion(
                &http,
                &model_config,
                &system_prompt,
                &prompt,
                temperature,
            )
            .await?;
            Ok::<_, crate::error::AppError>((dimension, parse_review_response(&result)?))
        }));
    }

    let mut dimensions = Vec::with_capacity(DIMENSIONS.len());
    for handle in handles {
        let (dimension, parsed) = handle
            .await
            .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!(e)))??;
        let score = parsed.score;
        let passed = score >= 60.0;

        let issues_value =
            serde_json::to_value(&parsed.issues).unwrap_or(serde_json::Value::Array(vec![]));
        save_review(
            db,
            session_id,
            beat_id,
            dimension,
            score,
            passed,
            &issues_value,
            &parsed.summary,
        )
        .await?;

        dimensions.push(DimensionResult {
            dimension: dimension.to_string(),
            score,
            passed,
            issues: parsed.issues,
            summary: parsed.summary,
        });
    }

    let overall_score = if dimensions.is_empty() {
        0.0
    } else {
        dimensions.iter().map(|d| d.score).sum::<f32>() / dimensions.len() as f32
    };
    let passed = dimensions.iter().all(|d| d.passed);

    Ok(AggregateReview {
        dimensions,
        overall_score,
        passed,
    })
}

pub async fn list_reviews(
    db: &crate::state::Db,
    session_id: &str,
) -> AppResult<Vec<WritingReview>> {
    Store::new(db)
        .find("writing_review")
        .filter_ref("session", "narrative_session", session_id)
        .order("created_at DESC")
        .all()
        .await
}

fn build_review_prompt(
    dimension: &str,
    beat_content: &str,
    beat_type: &str,
    contract: &context_service::ContextContract,
) -> String {
    let mut prompt = String::with_capacity(512);

    prompt.push_str(&format!("审查维度：{}\n\n", dimension));

    match dimension {
        "consistency" => {
            prompt.push_str("检查内容是否与已知设定、角色性格、前文情节保持一致。特别注意：\n");
            prompt.push_str("- 是否出现与世界观设定矛盾的描写\n");
            prompt.push_str("- 角色行为是否符合其性格特点\n");
            prompt.push_str("- 是否出现未解释的能力或知识获取\n");
        }
        "pacing" => {
            prompt.push_str("检查叙事节奏是否合理。特别注意：\n");
            prompt.push_str("- 信息密度是否适当（过密则读者疲劳，过疏则无聊）\n");
            prompt.push_str("- 场景转换是否自然\n");
            prompt.push_str("- 是否存在大段无效描写\n");
        }
        "reader_pull" => {
            prompt.push_str("检查内容的追读力。特别注意：\n");
            prompt.push_str("- 是否有推动读者继续阅读的动力\n");
            prompt.push_str("- 是否设置了有效的钩子（悬念、冲突、期待）\n");
            prompt.push_str("- 是否有微兑现（小收获、小进展）\n");
        }
        _ => {}
    }

    prompt.push_str(&format!("\n节拍类型：{}\n", beat_type));

    if !contract.character_states.is_empty() {
        prompt.push_str("\n角色当前状态：\n");
        for cs in &contract.character_states {
            prompt.push_str(&format!(
                "- {}：情绪 {}，位于 {}\n",
                cs.name, cs.emotion, cs.location
            ));
        }
    }

    if !contract.worldview.is_empty() {
        prompt.push_str("\n世界观设定：\n");
        for w in &contract.worldview {
            prompt.push_str(&format!("- [{}] {}\n", w.category, w.title));
        }
    }

    prompt.push_str(&format!("\n待审查内容：\n{}", beat_content));

    prompt
}

struct ParsedReview {
    score: f32,
    summary: String,
    issues: Vec<ReviewIssue>,
}

fn parse_review_response(response: &str) -> AppResult<ParsedReview> {
    let v: serde_json::Value = crate::services::ai_service::parse_json_response(
        response,
        "审查结果格式错误，请重试",
    )?;

    let score = v
        .get("score")
        .and_then(|s| s.as_f64())
        .ok_or_else(|| AppError::Ai("审查结果缺少 score 字段".to_string()))? as f32;

    let summary = v
        .get("summary")
        .and_then(|s| s.as_str())
        .unwrap_or("无法解析评价")
        .to_string();

    let issues = v
        .get("issues")
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ReviewIssue {
                        severity: item.get("severity")?.as_str()?.to_string(),
                        description: item.get("description")?.as_str()?.to_string(),
                        suggestion: item.get("suggestion")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ParsedReview {
        score,
        summary,
        issues,
    })
}

async fn save_review(
    db: &crate::state::Db,
    session_id: &str,
    beat_id: &str,
    dimension: &str,
    score: f32,
    passed: bool,
    issues: &serde_json::Value,
    summary: &str,
) -> AppResult<()> {
    Store::new(db)
        .content("writing_review")
        .ref_id("session", "narrative_session", session_id)
        .ref_id("beat", "narrative_beat", beat_id)
        .field("dimension", dimension)
        .field("score", score)
        .field("passed", passed)
        .field("issues", issues)
        .field("summary", summary)
        .exec::<WritingReview>()
        .await?;
    Ok(())
}

async fn get_project_id(db: &crate::state::Db, session_id: &str) -> AppResult<String> {
    let result: Option<String> = Store::new(db)
        .find("narrative_session")
        .project("VALUE project.id")
        .filter_ref("id", "narrative_session", session_id)
        .one()
        .await
        .ok();

    result.ok_or_else(|| crate::error::AppError::NotFound("会话不存在".to_string()))
}
