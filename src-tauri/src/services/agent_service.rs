use crate::db::models::AiConfig;
use crate::error::AppResult;
use crate::state::Db;

pub struct AgentConfig {
    pub model_config: AiConfig,
    pub system_prompt: String,
    pub temperature: f32,
}

struct PresetAgent {
    name: &'static str,
    system_prompt: &'static str,
    temperature: f32,
}

const PRESETS: &[PresetAgent] = &[
    PresetAgent {
        name: "叙事者",
        temperature: 0.85,
        system_prompt: "你是一位专业的小说叙事者，负责推动剧情发展。你的职责是：描述场景变化和氛围、推动情节发展创造戏剧冲突、控制节奏在紧张和舒缓之间切换、为角色创造互动机会。只输出叙事内容，不要加任何标记或说明。",
    },
    PresetAgent {
        name: "角色扮演",
        temperature: 0.85,
        system_prompt: "你现在正在扮演小说中的角色「{character_name}」。你必须完全沉浸在这个角色中，按照角色的性格、背景和当前处境做出真实的反应。只输出角色的反应（对话用「」包裹、行动描述、心理活动），不要跳出角色，不要加任何元说明。",
    },
    PresetAgent {
        name: "质量审查",
        temperature: 0.3,
        system_prompt: "你是一位专业的网文质量审查员。你需要审查一段小说内容，并给出结构化的评价。\n\n请严格按照以下 JSON 格式回复（不要输出任何其他内容）：\n{\n  \"score\": 85,\n  \"summary\": \"一句话总结评价\",\n  \"issues\": [\n    {\n      \"severity\": \"high|medium|low\",\n      \"description\": \"问题描述\",\n      \"suggestion\": \"改进建议\"\n    }\n  ]\n}\n\n评分标准：\n- 90-100：优秀，无需修改\n- 70-89：良好，有小问题\n- 60-69：及格，有明显问题\n- 0-59：不及格，需要重写\n\nseverity 说明：\n- high：严重问题（事实矛盾、角色崩坏、节奏灾难）\n- medium：中等问题（逻辑跳跃、模式重复、情绪断裂）\n- low：小问题（措辞优化、描写不足）",
    },
    PresetAgent {
        name: "续写助手",
        temperature: 0.8,
        system_prompt: "你是一位专业的小说创作助手。你的任务是根据已有内容续写故事。请保持与前文一致的风格、语气和叙事视角。续写内容要自然流畅，与前文无缝衔接。直接输出续写内容，不要加任何说明或标记。",
    },
    PresetAgent {
        name: "改写助手",
        temperature: 0.8,
        system_prompt: "你是一位专业的文字编辑。用户会给你一段文字和一个改写指令，请根据指令改写这段文字。直接输出改写后的内容，不要加任何说明或标记。",
    },
    PresetAgent {
        name: "润色助手",
        temperature: 0.8,
        system_prompt: "你是一位专业的文学编辑。请润色用户提供的文字，提升表达质量，使之更加流畅优美。保持原文的核心意思不变，适当优化用词和句式。直接输出润色后的内容，不要加任何说明或标记。",
    },
    PresetAgent {
        name: "对话生成",
        temperature: 0.8,
        system_prompt: "你是一位擅长创作对话的小说家。根据提供的角色信息和场景描述，生成自然生动的角色对话。对话要符合每个角色的性格特点，推动情节发展。使用中文引号「」包裹对话内容，并标注说话者。直接输出对话内容，不要加任何说明或标记。",
    },
    PresetAgent {
        name: "写作顾问",
        temperature: 0.8,
        system_prompt: "你是 Inkwell 写作助手，一位专业的小说创作顾问。你可以帮助用户解决写作中的各种问题：情节构思、角色塑造、文笔提升、结构规划等。请给出具体、有建设性的建议。用中文回复。",
    },
    PresetAgent {
        name: "大纲生成",
        temperature: 0.7,
        system_prompt: "你是一位专业的小说大纲策划师。你的任务是根据小说概念和已有设定，规划结构清晰、节奏合理的大纲。你需要：分析概念的核心冲突和主题、规划合理的卷章结构、确保情节递进和节奏变化、兼顾角色成长线和世界观展开。严格按照要求的 JSON 格式输出，不要输出任何其他内容。",
    },
];

pub async fn seed_presets(db: &Db) -> AppResult<()> {
    for preset in PRESETS {
        let existing: Option<crate::db::models::AiAgent> = db
            .query("SELECT id FROM ai_agent WHERE name = $name LIMIT 1")
            .bind(("name", preset.name.to_string()))
            .await?
            .check()?
            .take::<Option<crate::db::models::AiAgent>>(0)?;

        if existing.is_none() {
            db.query(
                "CREATE ai_agent CONTENT { \
                 name: $name, \
                 system_prompt: $prompt, \
                 temperature: $temp, \
                 is_default: false \
                 }",
            )
            .bind(("name", preset.name.to_string()))
            .bind(("prompt", preset.system_prompt.to_string()))
            .bind(("temp", preset.temperature))
            .await?
            .check()?;
        }
    }

    Ok(())
}

pub async fn get_agent_config(db: &Db, agent_id: &str) -> AppResult<AgentConfig> {
    let agent = super::ai_service::get_agent(db, agent_id).await?;

    let model_config = match agent.model_id {
        Some(ref mid) => super::ai_service::get_model(db, mid).await?,
        None => super::ai_service::get_default_config(db).await?,
    };

    Ok(AgentConfig {
        model_config,
        system_prompt: agent.system_prompt,
        temperature: agent.temperature,
    })
}

pub async fn get_agent_by_name(db: &Db, name: &str) -> AppResult<AgentConfig> {
    let agents = super::ai_service::list_agents(db).await?;
    let agent = agents
        .into_iter()
        .find(|a| a.name == name)
        .ok_or_else(|| crate::error::AppError::NotFound(format!("助手「{}」不存在", name)))?;

    let model_config = match agent.model_id {
        Some(ref mid) => super::ai_service::get_model(db, mid).await?,
        None => super::ai_service::get_default_config(db).await?,
    };

    Ok(AgentConfig {
        model_config,
        system_prompt: agent.system_prompt,
        temperature: agent.temperature,
    })
}

pub async fn apply_character_template(db: &Db, character_name: &str) -> AppResult<AgentConfig> {
    let mut config = get_agent_by_name(db, "角色扮演").await?;
    config.system_prompt = config
        .system_prompt
        .replace("{character_name}", character_name);
    Ok(config)
}
