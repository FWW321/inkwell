use serde::Deserialize;
use tauri::ipc::Channel;

use crate::db::Store;
use crate::db::models::{Workflow as WorkflowModel, WorkflowStep};
use crate::error::{AppError, AppResult};
use crate::services::{ai_service, agent_service, context_service};
use crate::state::Db;
use surrealdb::types::ToSql;

pub const STEP_TYPES: &[(&str, &str)] = &[
    ("generate_worldview", "世界观生成"),
    ("generate_characters", "角色生成"),
    ("generate_volume_structure", "卷结构生成"),
    ("generate_chapter_structure", "章节结构生成"),
    ("expand_chapter_outline", "章节扩写"),
    ("narrate", "推进叙事"),
    ("character_action", "角色行动"),
    ("polish", "润色"),
    ("rewrite", "改写"),
    ("continue_writing", "续写"),
    ("dialogue", "对话生成"),
    ("review", "质量审查"),
];

pub fn step_label(step_type: &str) -> String {
    STEP_TYPES
        .iter()
        .find(|(t, _)| *t == step_type)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| step_type.to_string())
}

fn default_agent_name(step_type: &str) -> &str {
    match step_type {
        "narrate" => "叙事者",
        "character_action" => "角色扮演",
        "review" => "质量审查",
        "polish" => "润色助手",
        "rewrite" => "改写助手",
        "continue_writing" => "续写助手",
        "dialogue" => "对话生成",
        _ => "大纲生成",
    }
}

struct PresetStep {
    step_type: &'static str,
    agent_name: Option<&'static str>,
    condition_json: Option<&'static str>,
}

struct PresetWorkflow {
    name: &'static str,
    description: &'static str,
    is_default: bool,
    steps: &'static [PresetStep],
}

const PRESETS: &[PresetWorkflow] = &[
    PresetWorkflow {
        name: "大纲生成",
        description: "从概念到完整大纲：世界观→角色→卷纲→章纲→扩写→润色→审查",
        is_default: true,
        steps: &[
            PresetStep { step_type: "generate_worldview", agent_name: None, condition_json: None },
            PresetStep { step_type: "generate_characters", agent_name: None, condition_json: None },
            PresetStep { step_type: "generate_volume_structure", agent_name: None, condition_json: None },
            PresetStep { step_type: "generate_chapter_structure", agent_name: None, condition_json: None },
            PresetStep { step_type: "expand_chapter_outline", agent_name: None, condition_json: None },
            PresetStep { step_type: "polish", agent_name: None, condition_json: None },
            PresetStep { step_type: "review", agent_name: None, condition_json: None },
        ],
    },
    PresetWorkflow {
        name: "叙事+审查",
        description: "推进叙事后自动审查质量，分数低则改写后再审查",
        is_default: false,
        steps: &[
            PresetStep { step_type: "narrate", agent_name: None, condition_json: None },
            PresetStep { step_type: "review", agent_name: None, condition_json: None },
            PresetStep { step_type: "rewrite", agent_name: None, condition_json: Some(r#"{"field":"prev_score","op":"lt","value":70}"#) },
            PresetStep { step_type: "review", agent_name: None, condition_json: Some(r#"{"after":"rewrite"}"#) },
        ],
    },
    PresetWorkflow {
        name: "润色+审查",
        description: "润色后审查质量",
        is_default: false,
        steps: &[
            PresetStep { step_type: "polish", agent_name: None, condition_json: None },
            PresetStep { step_type: "review", agent_name: None, condition_json: None },
        ],
    },
    PresetWorkflow {
        name: "续写+润色",
        description: "续写后自动润色",
        is_default: false,
        steps: &[
            PresetStep { step_type: "continue_writing", agent_name: None, condition_json: None },
            PresetStep { step_type: "polish", agent_name: None, condition_json: None },
        ],
    },
];

fn parse_condition(json_str: Option<&str>) -> Option<serde_json::Value> {
    json_str.and_then(|s| serde_json::from_str(s).ok())
}

pub async fn seed_presets(db: &Db) -> AppResult<()> {
    let store = Store::new(db);
    for preset in PRESETS {
        let existing: Option<WorkflowModel> = store
            .find("workflow")
            .project("id")
            .filter_eq("name", preset.name)
            .limit(1)
            .one()
            .await
            .ok();

        if existing.is_some() {
            continue;
        }

        let wf: WorkflowModel = store
            .content("workflow")
            .field("name", preset.name)
            .field("description", preset.description)
            .field("is_preset", true)
            .field("is_default", preset.is_default)
            .field("step_count", preset.steps.len() as i64)
            .exec::<WorkflowModel>()
            .await?;

        let wf_id = wf.id.key.to_sql();

        for (i, step) in preset.steps.iter().enumerate() {
            let target_name = step.agent_name.unwrap_or_else(|| default_agent_name(step.step_type));
            let agent_id = find_agent_id_by_name(db, target_name).await.ok();
            let condition = parse_condition(step.condition_json);

            let mut builder = store
                .content("workflow_step")
                .ref_id("workflow", "workflow", &wf_id)
                .field("sort_order", i as i64)
                .field("step_type", step.step_type)
                .field("condition", condition.as_ref().unwrap_or(&serde_json::Value::Null))
                .field("config", serde_json::json!({}))
                .field("enabled", true);

            if let Some(ref aid) = agent_id {
                builder = builder.opt_ref("agent", "ai_agent", Some(aid.as_str()));
            }

            builder.exec::<WorkflowStep>().await?;
        }
    }

    Ok(())
}

async fn find_agent_id_by_name(db: &Db, name: &str) -> AppResult<String> {
    let agents = ai_service::list_agents(db).await?;
    let agent = agents
        .into_iter()
        .find(|a| a.name == name)
        .ok_or_else(|| AppError::NotFound(format!("助手「{}」不存在", name)))?;
    Ok(agent.id.key.to_sql())
}

async fn resolve_agent_id(db: &Db, step: &WorkflowStep, fallback: &str) -> AppResult<String> {
    match &step.agent {
        Some(rid) => Ok(rid.key.to_sql()),
        None => find_agent_id_by_name(db, fallback).await,
    }
}

pub async fn list_workflows(db: &Db) -> AppResult<Vec<WorkflowModel>> {
    Store::new(db)
        .find("workflow")
        .order("is_default DESC, name ASC")
        .all()
        .await
}

pub async fn list_steps(db: &Db, workflow_id: &str) -> AppResult<Vec<WorkflowStep>> {
    Store::new(db)
        .find("workflow_step")
        .filter_ref("workflow", "workflow", workflow_id)
        .order("sort_order ASC")
        .all()
        .await
}

pub async fn get_workflow(db: &Db, id: &str) -> AppResult<WorkflowModel> {
    Store::new(db)
        .find("workflow")
        .filter_ref("id", "workflow", id)
        .one()
        .await
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateStepInput {
    pub step_type: String,
    pub agent_id: Option<String>,
    pub condition: Option<serde_json::Value>,
    pub config: serde_json::Value,
    pub enabled: bool,
}

pub async fn create_workflow(
    db: &Db,
    name: &str,
    description: &str,
    steps: Vec<CreateStepInput>,
) -> AppResult<WorkflowModel> {
    let store = Store::new(db);
    let step_count = steps.len() as i64;

    let wf: WorkflowModel = store
        .content("workflow")
        .field("name", name)
        .field("description", description)
        .field("is_preset", false)
        .field("is_default", false)
        .field("step_count", step_count)
        .exec::<WorkflowModel>()
        .await?;

    persist_steps(db, &wf.id.key.to_sql(), &steps).await?;

    Ok(wf)
}

pub async fn update_workflow(
    db: &Db,
    id: &str,
    name: &str,
    description: &str,
    steps: Vec<CreateStepInput>,
) -> AppResult<WorkflowModel> {
    let store = Store::new(db);

    delete_steps_for_workflow(db, id).await?;

    let wf: WorkflowModel = store
        .update("workflow", id)
        .set("name", name)
        .set("description", description)
        .set("step_count", steps.len() as i64)
        .touch()
        .get::<WorkflowModel>()
        .await?;

    persist_steps(db, id, &steps).await?;

    Ok(wf)
}

pub async fn delete_workflow(db: &Db, id: &str) -> AppResult<()> {
    delete_steps_for_workflow(db, id).await?;
    Store::new(db).delete("workflow", id).await
}

pub async fn set_default_workflow(db: &Db, id: &str) -> AppResult<()> {
    db.query("UPDATE type::table($table) SET is_default = false WHERE is_default = true")
        .bind(("table", "workflow".to_string()))
        .await?
        .check()?;

    Store::new(db)
        .update("workflow", id)
        .set("is_default", true)
        .touch()
        .get::<WorkflowModel>()
        .await?;

    Ok(())
}

async fn delete_steps_for_workflow(db: &Db, workflow_id: &str) -> AppResult<()> {
    let existing: Vec<WorkflowStep> = Store::new(db)
        .find("workflow_step")
        .filter_ref("workflow", "workflow", workflow_id)
        .project("id")
        .all()
        .await?;

    for old_step in &existing {
        let old_id = old_step.id.key.to_sql();
        let _: Option<surrealdb::types::Value> = db
            .query("DELETE type::record($id)")
            .bind(("id", old_id))
            .await?
            .check()?
            .take(0)?;
    }

    Ok(())
}

async fn persist_steps(db: &Db, workflow_id: &str, steps: &[CreateStepInput]) -> AppResult<()> {
    let store = Store::new(db);
    for (i, step) in steps.iter().enumerate() {
        let mut builder = store
            .content("workflow_step")
            .ref_id("workflow", "workflow", workflow_id)
            .field("sort_order", i as i64)
            .field("step_type", &step.step_type)
            .field("condition", step.condition.as_ref().unwrap_or(&serde_json::Value::Null))
            .field("config", &step.config)
            .field("enabled", step.enabled);

        if let Some(ref agent_id) = step.agent_id {
            if !agent_id.is_empty() {
                builder = builder.opt_ref("agent", "ai_agent", Some(agent_id.as_str()));
            }
        }

        builder.exec::<WorkflowStep>().await?;
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkflowProgress {
    pub step_index: i32,
    pub step_count: i32,
    pub step_type: String,
    pub step_label: String,
    pub status: String,
    pub message: String,
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkflowResult {
    pub steps: Vec<StepResult>,
    pub final_text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StepResult {
    pub step_index: i32,
    pub type_str: String,
    pub label: String,
    pub status: String,
    pub output_text: String,
    pub score: Option<f32>,
    pub error: Option<String>,
}

struct StepOutput {
    text: String,
    score: Option<f32>,
}

pub struct WorkflowRunParams {
    pub project_id: String,
    pub session_id: Option<String>,
    pub text: Option<String>,
    pub instruction: Option<String>,
    pub character_id: Option<String>,
}

pub async fn run_workflow(
    db: &Db,
    http: &reqwest::Client,
    workflow_id: &str,
    params: WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<WorkflowResult> {
    let steps = list_steps(db, workflow_id).await?;
    let enabled: Vec<&WorkflowStep> = steps.iter().filter(|s| s.enabled).collect();
    let total = enabled.len() as i32;

    if total == 0 {
        return Ok(WorkflowResult {
            steps: vec![],
            final_text: String::new(),
        });
    }

    let mut results = Vec::with_capacity(total as usize);
    let mut prev_output: Option<StepOutput> = None;
    let mut rewrite_triggered = false;

    for (idx, step) in enabled.iter().enumerate() {
        let idx_i32 = idx as i32;
        let label = step_label(&step.step_type).to_string();

        let should_run = evaluate_condition(&step.condition, &prev_output, rewrite_triggered);

        if !should_run {
            let _ = on_progress.send(WorkflowProgress {
                step_index: idx_i32,
                step_count: total,
                step_type: step.step_type.clone(),
                step_label: label.clone(),
                status: "skipped".to_string(),
                message: "条件不满足，跳过".to_string(),
                text: String::new(),
                done: false,
            });
            results.push(StepResult {
                step_index: idx_i32,
                type_str: step.step_type.clone(),
                label,
                status: "skipped".to_string(),
                output_text: String::new(),
                score: None,
                error: None,
            });
            continue;
        }

        let _ = on_progress.send(WorkflowProgress {
            step_index: idx_i32,
            step_count: total,
            step_type: step.step_type.clone(),
            step_label: label.clone(),
            status: "running".to_string(),
            message: format!("正在执行：{}", &label),
            text: String::new(),
            done: false,
        });

        let step_result = execute_step(db, http, step, &params, &prev_output, on_progress).await;

        match step_result {
            Ok(output) => {
                if step.step_type == "rewrite" {
                    rewrite_triggered = true;
                }
                results.push(StepResult {
                    step_index: idx_i32,
                    type_str: step.step_type.clone(),
                    label: label.clone(),
                    status: "completed".to_string(),
                    output_text: output.text.clone(),
                    score: output.score,
                    error: None,
                });
                prev_output = Some(output);
            }
            Err(e) => {
                results.push(StepResult {
                    step_index: idx_i32,
                    type_str: step.step_type.clone(),
                    label,
                    status: "error".to_string(),
                    output_text: String::new(),
                    score: None,
                    error: Some(e.to_string()),
                });
                break;
            }
        }
    }

    let final_text = prev_output.map(|o| o.text).unwrap_or_default();

    let _ = on_progress.send(WorkflowProgress {
        step_index: total,
        step_count: total,
        step_type: String::new(),
        step_label: String::new(),
        status: "done".to_string(),
        message: "工作流执行完成".to_string(),
        text: String::new(),
        done: true,
    });

    Ok(WorkflowResult {
        steps: results,
        final_text,
    })
}

fn evaluate_condition(
    condition: &Option<serde_json::Value>,
    prev_output: &Option<StepOutput>,
    rewrite_triggered: bool,
) -> bool {
    let Some(cond) = condition else {
        return true;
    };
    let obj = match cond.as_object() {
        Some(o) => o,
        None => return true,
    };

    if let Some(after) = obj.get("after").and_then(|v| v.as_str()) {
        if after == "rewrite" && !rewrite_triggered {
            return false;
        }
    }

    let Some(field) = obj.get("field").and_then(|v| v.as_str()) else {
        return true;
    };
    let op = obj.get("op").and_then(|v| v.as_str()).unwrap_or("eq");
    let value = obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    let actual = if field == "prev_score" {
        prev_output.as_ref().and_then(|o| o.score).unwrap_or(100.0)
    } else {
        prev_output.as_ref().and_then(|o| o.score).unwrap_or(100.0)
    };

    match op {
        "lt" => actual < value,
        "lte" => actual <= value,
        "gt" => actual > value,
        "gte" => actual >= value,
        "eq" => (actual - value).abs() < f32::EPSILON,
        _ => true,
    }
}

async fn execute_step(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    prev_output: &Option<StepOutput>,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    match step.step_type.as_str() {
        "generate_worldview" => exec_generate_worldview(db, http, step, params, on_progress).await,
        "generate_characters" => exec_generate_characters(db, http, step, params, on_progress).await,
        "generate_volume_structure" => exec_generate_volumes(db, http, step, params, on_progress).await,
        "generate_chapter_structure" => exec_generate_chapters(db, http, step, params, on_progress).await,
        "expand_chapter_outline" => exec_expand_outlines(db, http, step, params, on_progress).await,
        "review" => exec_review(db, http, params, on_progress).await,
        "polish" | "rewrite" | "continue_writing" | "dialogue" => {
            exec_editor_step(db, http, step, params, prev_output, on_progress).await
        }
        "narrate" => exec_narrate(db, http, step, params, on_progress).await,
        _ => Err(AppError::Validation(format!("未知步骤类型: {}", step.step_type))),
    }
}

async fn exec_generate_worldview(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let concept = params.text.as_deref().unwrap_or("");
    let agent_id = resolve_agent_id(db, step, "大纲生成").await?;
    let config = agent_service::get_agent_config(db, &agent_id).await?;
    let contract = context_service::build_contract(db, &params.project_id, "").await?;
    let context = format_project_context(&contract);
    let preamble = format!("{}\n\n{}", config.system_prompt, context);

    let prompt = format!(
        "小说概念：{}\n\n\
        根据以上概念，生成世界观设定。严格按照以下 JSON 格式回复（不要输出任何其他内容）：\n\
        [{{\"category\": \"类别\", \"title\": \"名称\", \"content\": \"详细描述（50-150字）\"}}]\n\n\
        要求：\n- 生成 5-10 个世界观设定条目\n- 涵盖地理、历史、力量体系、社会结构、种族等方面\n- 每个条目内容具体且有深度",
        concept
    );

    let result = ai_service::chat_completion(http, &config.model_config, &preamble, &prompt, config.temperature).await?;

    #[derive(Deserialize)]
    struct Item { category: String, title: String, content: String }

    let items: Vec<Item> = ai_service::parse_json_response(&result, "世界观生成结果解析失败，请重试")?;

    for item in &items {
        crate::services::worldview_service::create(db, &params.project_id, &item.category, &item.title, &item.content).await?;
    }

    let summary = items.iter().map(|i| format!("[{}] {}：{}", i.category, i.title, i.content)).collect::<Vec<_>>().join("\n");

    send_progress(on_progress, step, format!("已生成 {} 条世界观设定", items.len()), &summary);

    Ok(StepOutput { text: summary, score: None })
}

async fn exec_generate_characters(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let concept = params.text.as_deref().unwrap_or("");
    let agent_id = resolve_agent_id(db, step, "大纲生成").await?;
    let config = agent_service::get_agent_config(db, &agent_id).await?;
    let contract = context_service::build_contract(db, &params.project_id, "").await?;
    let context = format_project_context(&contract);
    let preamble = format!("{}\n\n{}", config.system_prompt, context);

    let prompt = format!(
        "小说概念：{}\n\n\
        根据以上概念和世界观，生成主要角色。严格按照以下 JSON 格式回复（不要输出任何其他内容）：\n\
        [{{\"name\": \"角色名\", \"personality\": \"性格描述\", \"description\": \"外貌描述\", \"background\": \"背景故事\"}}]\n\n\
        要求：\n- 生成 3-8 个主要角色\n- 包括主角、重要配角、反派等\n- 每个角色有鲜明的性格和合理的背景",
        concept
    );

    let result = ai_service::chat_completion(http, &config.model_config, &preamble, &prompt, config.temperature).await?;

    #[derive(Deserialize)]
    struct Item { name: String, personality: String, description: String, background: String }

    let items: Vec<Item> = ai_service::parse_json_response(&result, "角色生成结果解析失败，请重试")?;

    for item in &items {
        crate::services::character_service::create(
            db, &params.project_id, &item.name, serde_json::Value::Array(vec![]),
            &item.description, &item.personality, &item.background, "", None,
        ).await?;
    }

    let summary = items.iter().map(|i| format!("- {}（{}）：{}", i.name, i.personality, i.background)).collect::<Vec<_>>().join("\n");

    send_progress(on_progress, step, format!("已生成 {} 个角色", items.len()), &summary);

    Ok(StepOutput { text: summary, score: None })
}

async fn exec_generate_volumes(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let concept = params.text.as_deref().unwrap_or("");
    let agent_id = resolve_agent_id(db, step, "大纲生成").await?;

    let nodes = crate::services::outline_generation_service::generate_volume_structure(
        db, http, &params.project_id, concept, &agent_id,
    ).await?;

    let summary = nodes.iter().map(|n| {
        let outline = n.content_json.get("outline").and_then(|v| v.as_str()).unwrap_or("");
        format!("- {}：{}", n.title, outline)
    }).collect::<Vec<_>>().join("\n");

    send_progress(on_progress, step, format!("已生成 {} 卷", nodes.len()), &summary);

    Ok(StepOutput { text: summary, score: None })
}

async fn exec_generate_chapters(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let volumes = crate::services::outline_service::list_children(db, &params.project_id, None).await?;
    let agent_id = resolve_agent_id(db, step, "大纲生成").await?;

    let mut total = 0usize;
    for volume in &volumes {
        let vid = volume.id.key.to_sql();
        let chapters = crate::services::outline_generation_service::generate_chapter_structure(db, http, &vid, &agent_id).await?;
        total += chapters.len();
        send_progress(on_progress, step, format!("{}：已生成 {} 章", volume.title, chapters.len()), "");
    }

    Ok(StepOutput { text: format!("共生成 {} 个章节", total), score: None })
}

async fn exec_expand_outlines(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let volumes = crate::services::outline_service::list_children(db, &params.project_id, None).await?;
    let agent_id = resolve_agent_id(db, step, "大纲生成").await?;

    let mut expanded = 0usize;
    for volume in &volumes {
        let vid = volume.id.key.to_sql();
        let chapters = crate::services::outline_service::list_children(db, &params.project_id, Some(&vid)).await?;
        for chapter in &chapters {
            let cid = chapter.id.key.to_sql();
            let _ = crate::services::outline_generation_service::expand_chapter_outline(db, http, &cid, &agent_id).await?;
            expanded += 1;
            send_progress(on_progress, step, format!("已扩写：{}", chapter.title), "");
        }
    }

    Ok(StepOutput { text: format!("已扩写 {} 个章节大纲", expanded), score: None })
}

async fn exec_review(
    db: &Db,
    http: &reqwest::Client,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let session_id = params.session_id.as_deref()
        .ok_or_else(|| AppError::Validation("审查步骤需要叙事会话".to_string()))?;

    let beats = crate::services::narrative_service::list_beats(db, session_id).await?;
    let last = beats.iter().rev()
        .find(|b| b.beat_type == "narration" || b.beat_type == "character_action");

    let (beat_id, beat_content, beat_type) = match last {
        Some(b) => (b.id.key.to_sql(), b.content.clone(), b.beat_type.clone()),
        None => return Ok(StepOutput { text: "没有可审查的内容".to_string(), score: Some(0.0) }),
    };

    let review = crate::services::review_service::review_beat(db, http, session_id, &beat_id, &beat_content, &beat_type, None).await?;

    let summary = review.dimensions.iter()
        .map(|d| format!("- {}：{:.0}分 {}", d.dimension, d.score, d.summary))
        .collect::<Vec<_>>().join("\n");

    send_progress_str(on_progress, "review", "质量审查", format!("综合评分：{:.0}", review.overall_score), &summary);

    Ok(StepOutput { text: summary, score: Some(review.overall_score) })
}

async fn exec_editor_step(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    prev_output: &Option<StepOutput>,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let fallback = match step.step_type.as_str() {
        "polish" => "润色助手",
        "rewrite" => "改写助手",
        "continue_writing" => "续写助手",
        "dialogue" => "对话生成",
        _ => "润色助手",
    };

    let agent_id = resolve_agent_id(db, step, fallback).await?;
    let config = agent_service::get_agent_config(db, &agent_id).await?;
    let contract = context_service::build_contract(db, &params.project_id, "").await?;
    let context = context_service::build_editor_context(&contract);
    let preamble = format!("{}\n\n{}", config.system_prompt, context);

    let mut user_text = params.text.clone().unwrap_or_default();
    if user_text.is_empty() {
        if let Some(prev) = prev_output { user_text = prev.text.clone(); }
    }
    if user_text.is_empty() {
        return Ok(StepOutput { text: String::new(), score: None });
    }

    let mut prompt_text = user_text.clone();
    if step.step_type == "rewrite" {
        if let Some(prev) = prev_output {
            if let Some(score) = prev.score {
                let issues = extract_review_issues(&prev.text);
                prompt_text = format!(
                    "[改写指令：根据审查反馈修改以下内容。审查评分：{:.0}分]\n[审查反馈：{}]\n\n[待改写内容：]\n{}",
                    score, issues, user_text
                );
            }
        }
    }
    if step.step_type == "continue_writing" {
        prompt_text = format!("请续写以下内容：\n\n{}", user_text);
    }

    let mut full_text = String::new();
    ai_service::stream_ai_with_callback(http, &config.model_config, &preamble, &prompt_text, config.temperature, |chunk| {
        if !chunk.text.is_empty() {
            full_text.push_str(&chunk.text);
            let _ = on_progress.send(WorkflowProgress {
                step_index: 0, step_count: 0,
                step_type: step.step_type.clone(),
                step_label: step_label(&step.step_type).to_string(),
                status: "running".to_string(),
                message: String::new(), text: chunk.text, done: false,
            });
        }
    }).await?;

    Ok(StepOutput { text: full_text, score: None })
}

async fn exec_narrate(
    db: &Db,
    http: &reqwest::Client,
    step: &WorkflowStep,
    params: &WorkflowRunParams,
    on_progress: &Channel<WorkflowProgress>,
) -> AppResult<StepOutput> {
    let session_id = params.session_id.as_deref()
        .ok_or_else(|| AppError::Validation("叙事步骤需要叙事会话".to_string()))?;

    let agent_id_opt = step.agent.as_ref().map(|a| a.key.to_sql());
    let agent_config = agent_service::resolve_agent(db, agent_id_opt.as_deref(), "叙事者", None).await?;

    let session = crate::services::narrative_service::get_session(db, session_id).await?;
    let project_id = session.project.key.to_sql();
    let contract = context_service::build_contract(db, &project_id, session_id).await?;

    let strand = crate::services::narrative_service::pick_next_strand(&contract.strand_context);
    let context = context_service::build_narrative_context(&contract, &session.scene, &session.atmosphere);
    let strand_guidance = crate::services::narrative_service::build_strand_instruction(&strand, &contract.strand_context);
    let preamble = format!("{}{}{}", agent_config.system_prompt, context, strand_guidance);

    let mut prompt = String::new();
    if let Some(ref inst) = params.instruction {
        prompt.push_str(&format!("作者指令：{}\n\n", inst));
    }
    prompt.push_str("请推进剧情，描述场景变化、角色行动和情节发展。只输出叙事内容，不要加任何标记或说明。");

    let mut full_text = String::new();
    ai_service::stream_ai_with_callback(http, &agent_config.model_config, &preamble, &prompt, agent_config.temperature, |chunk| {
        if !chunk.text.is_empty() {
            full_text.push_str(&chunk.text);
            let _ = on_progress.send(WorkflowProgress {
                step_index: 0, step_count: 0,
                step_type: "narrate".to_string(), step_label: "推进叙事".to_string(),
                status: "running".to_string(), message: String::new(),
                text: chunk.text, done: false,
            });
        }
    }).await?;

    let beats = crate::services::narrative_service::list_beats(db, session_id).await?;
    let max_order: i64 = beats.last().map_or(0, |b| b.sort_order + 1);

    let _ = db
        .query("LET $char_ref = NONE; CREATE narrative_beat CONTENT { session: type::record('narrative_session', $sid), beat_type: $beat_type, character: $char_ref, character_name: $char_name, content: $content, sort_order: $sort_order, strand: $strand }")
        .bind(("sid", session_id.to_string()))
        .bind(("beat_type", "narration".to_string()))
        .bind(("char_name", "叙事者".to_string()))
        .bind(("content", full_text.clone()))
        .bind(("sort_order", max_order))
        .bind(("strand", strand))
        .await?
        .check()?;

    Ok(StepOutput { text: full_text, score: None })
}

fn extract_review_issues(text: &str) -> String {
    text.lines()
        .filter(|l| l.contains("分") || l.contains("问题") || l.contains("建议"))
        .take(5)
        .collect::<Vec<_>>()
        .join("；")
}

fn format_project_context(contract: &context_service::ContextContract) -> String {
    let mut parts = Vec::with_capacity(3);
    if !contract.project_title.is_empty() {
        parts.push(format!("小说：《{}》", contract.project_title));
    }
    if !contract.characters.is_empty() {
        let chars: Vec<String> = contract.characters.iter().map(|c| {
            let mut info = format!("- {}（{}）", c.name, c.personality);
            if !c.background.is_empty() { info.push_str(&format!("：{}", c.background)); }
            info
        }).collect();
        parts.push(format!("\n主要角色：\n{}", chars.join("\n")));
    }
    if !contract.worldview.is_empty() {
        let wv: Vec<String> = contract.worldview.iter()
            .map(|w| format!("- [{}] {}：{}", w.category, w.title, w.content))
            .collect();
        parts.push(format!("\n世界观设定：\n{}", wv.join("\n")));
    }
    parts.join("\n")
}

fn send_progress(on_progress: &Channel<WorkflowProgress>, step: &WorkflowStep, message: impl Into<String>, text: impl Into<String>) {
    let _ = on_progress.send(WorkflowProgress {
        step_index: 0, step_count: 0,
        step_type: step.step_type.clone(),
        step_label: step_label(&step.step_type),
        status: "completed".to_string(),
        message: message.into(), text: text.into(), done: false,
    });
}

fn send_progress_str(on_progress: &Channel<WorkflowProgress>, step_type: &str, step_label: &str, message: String, text: &str) {
    let _ = on_progress.send(WorkflowProgress {
        step_index: 0, step_count: 0,
        step_type: step_type.to_string(),
        step_label: step_label.to_string(),
        status: "completed".to_string(),
        message, text: text.to_string(), done: false,
    });
}
