use crate::db::models::{AiConfig, WritingReview};
use crate::error::AppResult;
use crate::services::context_service;
use crate::state::Db;
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue, ToSql, Value};
#[derive(Debug, Clone, Serialize)]
pub struct ReviewStreamChunk {
    pub dimension: String,
    pub score: f32,
    pub passed: bool,
    pub issues: Vec<ReviewIssue>,
    pub summary: String,
    pub done: bool,
}

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
    db: &Db,
    session_id: &str,
    beat_id: &str,
    beat_content: &str,
    beat_type: &str,
) -> AppResult<AggregateReview> {
    let config = crate::services::ai_service::get_default_config(db).await?;
    let contract = context_service::build_contract(db, &get_project_id(db, session_id).await?, session_id).await?;

    let mut dimensions = Vec::new();

    for &dimension in DIMENSIONS {
        let prompt = build_review_prompt(dimension, beat_content, beat_type, &contract);
        let result = crate::services::ai_service::chat_completion(&config, REVIEW_SYSTEM_PROMPT, &prompt, 0.3).await?;

        let parsed = parse_review_response(&result);
        let score = parsed.score;
        let passed = score >= 60.0;

        save_review(db, session_id, beat_id, dimension, score, passed, &serde_json::to_value(&parsed.issues).unwrap_or(serde_json::Value::Array(vec![])), &parsed.summary).await?;

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

pub async fn list_reviews(db: &Db, session_id: &str) -> AppResult<Vec<WritingReview>> {
    db.query("SELECT * FROM writing_review WHERE session = $sid ORDER BY created_at DESC")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?.check()?.take::<Vec<WritingReview>>(0).map_err(Into::into)
}

const REVIEW_SYSTEM_PROMPT: &str = "你是一位专业的网文质量审查员。你需要审查一段小说内容，并给出结构化的评价。

请严格按照以下 JSON 格式回复（不要输出任何其他内容）：
{
  \"score\": 85,
  \"summary\": \"一句话总结评价\",
  \"issues\": [
    {
      \"severity\": \"high|medium|low\",
      \"description\": \"问题描述\",
      \"suggestion\": \"改进建议\"
    }
  ]
}

评分标准：
- 90-100：优秀，无需修改
- 70-89：良好，有小问题
- 60-69：及格，有明显问题
- 0-59：不及格，需要重写

severity 说明：
- high：严重问题（事实矛盾、角色崩坏、节奏灾难）
- medium：中等问题（逻辑跳跃、模式重复、情绪断裂）
- low：小问题（措辞优化、描写不足）";

fn build_review_prompt(
    dimension: &str,
    beat_content: &str,
    beat_type: &str,
    contract: &context_service::ContextContract,
) -> String {
    let mut prompt = String::new();

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
            prompt.push_str(&format!("- {}：情绪 {}，位于 {}\n", cs.name, cs.emotion, cs.location));
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

fn parse_review_response(response: &str) -> ParsedReview {
    let clean = response.trim().trim_start_matches("```json").trim_end_matches("```").trim();

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(clean) {
        let score = v["score"].as_f64().unwrap_or(70.0) as f32;
        let summary = v["summary"].as_str().unwrap_or("无法解析评价").to_string();
        let issues = v["issues"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(ReviewIssue {
                            severity: item["severity"].as_str().unwrap_or("low").to_string(),
                            description: item["description"].as_str().unwrap_or("").to_string(),
                            suggestion: item["suggestion"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        return ParsedReview { score, summary, issues };
    }

    ParsedReview {
        score: 70.0,
        summary: "审查结果解析失败，请重试".to_string(),
        issues: vec![],
    }
}

async fn save_review(
    db: &Db,
    session_id: &str,
    beat_id: &str,
    dimension: &str,
    score: f32,
    passed: bool,
    issues: &serde_json::Value,
    summary: &str,
) -> AppResult<()> {
    db.query(
        "CREATE writing_review CONTENT { \
         session: type::record('narrative_session', $sid), \
         beat: type::record('narrative_beat', $bid), \
         dimension: $dim, \
         score: $score, \
         passed: $passed, \
         issues: $issues, \
         summary: $summary \
         }"
    )
    .bind(("sid", session_id.to_string()))
    .bind(("bid", beat_id.to_string()))
    .bind(("dim", dimension.to_string()))
    .bind(("score", score))
    .bind(("passed", passed))
    .bind(("issues", issues.clone()))
    .bind(("summary", summary.to_string()))
    .await?.check()?;
    Ok(())
}

async fn get_project_id(db: &Db, session_id: &str) -> AppResult<String> {
    let result: Option<String> = db
        .query("SELECT VALUE project.id FROM narrative_session WHERE id = $sid")
        .bind(("sid", RecordId::new("narrative_session", session_id)))
        .await?.check()?.take::<Option<String>>(0)?;

    result.ok_or_else(|| crate::error::AppError::NotFound("会话不存在".to_string()))
}
