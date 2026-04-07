use crate::error::AppResult;
use surrealdb::Surreal;
use surrealdb::engine::local::SurrealKv;

use surrealdb::engine::local::Db as SurrealDb;

pub type Db = Surreal<SurrealDb>;

pub struct AppState {
    db: Db,
    http: reqwest::Client,
}

impl AppState {
    pub async fn new() -> AppResult<Self> {
        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("inkwell");

        std::fs::create_dir_all(&db_path)?;

        let db = Surreal::new::<SurrealKv>(db_path.to_string_lossy().as_ref()).await?;
        db.use_ns("inkwell").use_db("inkwell").await?;
        init_schema(&db).await?;

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!(e)))?;

        Ok(Self { db, http })
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}

async fn init_schema(db: &Db) -> AppResult<()> {
    db.query(r#"
        DEFINE TABLE project SCHEMAFULL;
        DEFINE FIELD title ON project TYPE string;
        DEFINE FIELD description ON project TYPE string DEFAULT '';
        DEFINE FIELD author ON project TYPE string DEFAULT '';
        DEFINE FIELD language ON project TYPE string DEFAULT '';
        DEFINE FIELD tags ON project TYPE string DEFAULT '';
        DEFINE FIELD status ON project TYPE string DEFAULT 'ongoing'
            ASSERT $value IN ['ongoing', 'completed', 'hiatus'];
        DEFINE FIELD cover_url ON project TYPE option<string>;
        DEFINE FIELD created_at ON project TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON project TYPE datetime DEFAULT time::now();

        DEFINE TABLE outline_node SCHEMAFULL;
        DEFINE FIELD project ON outline_node TYPE record<project>;
        DEFINE INDEX idx_outline_node_project ON outline_node FIELDS project;
        DEFINE FIELD parent ON outline_node TYPE option<record<outline_node>>;
        DEFINE INDEX idx_outline_node_project_parent ON outline_node FIELDS project, parent;
        DEFINE FIELD node_type ON outline_node TYPE string
            ASSERT $value IN ['volume', 'chapter', 'scene'];
        DEFINE INDEX idx_outline_node_project_type ON outline_node FIELDS project, node_type;
        DEFINE FIELD title ON outline_node TYPE string;
        DEFINE FIELD sort_order ON outline_node TYPE int DEFAULT 0;
        DEFINE FIELD content_json ON outline_node TYPE any DEFAULT {};
        DEFINE FIELD word_count ON outline_node TYPE int DEFAULT 0;
        DEFINE FIELD status ON outline_node TYPE string DEFAULT 'draft'
            ASSERT $value IN ['draft', 'in_progress', 'complete', 'revised'];
        DEFINE FIELD diff_original ON outline_node TYPE option<string>;
        DEFINE FIELD diff_new ON outline_node TYPE option<string>;
        DEFINE FIELD diff_mode ON outline_node TYPE option<string>;
        DEFINE FIELD created_at ON outline_node TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON outline_node TYPE datetime DEFAULT time::now();

        DEFINE TABLE character SCHEMAFULL;
        DEFINE FIELD project ON character TYPE record<project>;
        DEFINE INDEX idx_character_project ON character FIELDS project;
        DEFINE FIELD name ON character TYPE string;
        DEFINE FIELD aliases ON character TYPE any DEFAULT [];
        DEFINE FIELD avatar_url ON character TYPE option<string>;
        DEFINE FIELD description ON character TYPE string DEFAULT '';
        DEFINE FIELD personality ON character TYPE string DEFAULT '';
        DEFINE FIELD background ON character TYPE string DEFAULT '';
        DEFINE FIELD race ON character TYPE string DEFAULT '';
        DEFINE FIELD model ON character TYPE option<record<ai_model>>;
        DEFINE FIELD created_at ON character TYPE datetime DEFAULT time::now();

        DEFINE TABLE worldview_entry SCHEMAFULL;
        DEFINE FIELD project ON worldview_entry TYPE record<project>;
        DEFINE INDEX idx_worldview_entry_project ON worldview_entry FIELDS project;
        DEFINE FIELD category ON worldview_entry TYPE string DEFAULT '';
        DEFINE FIELD title ON worldview_entry TYPE string;
        DEFINE FIELD content ON worldview_entry TYPE string DEFAULT '';
        DEFINE FIELD created_at ON worldview_entry TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON worldview_entry TYPE option<datetime>;

        DEFINE TABLE ai_model SCHEMAFULL;
        DEFINE FIELD name ON ai_model TYPE string;
        DEFINE INDEX idx_ai_model_name ON ai_model FIELDS name UNIQUE;
        DEFINE FIELD api_key ON ai_model TYPE string DEFAULT '';
        DEFINE FIELD model ON ai_model TYPE string DEFAULT '';
        DEFINE FIELD base_url ON ai_model TYPE string DEFAULT 'https://api.openai.com/v1';
        DEFINE FIELD is_default ON ai_model TYPE bool DEFAULT false;
        DEFINE FIELD created_at ON ai_model TYPE datetime DEFAULT time::now();

        DEFINE TABLE ai_agent SCHEMAFULL;
        DEFINE FIELD name ON ai_agent TYPE string;
        DEFINE INDEX idx_ai_agent_name ON ai_agent FIELDS name UNIQUE;
        DEFINE FIELD model ON ai_agent TYPE option<record<ai_model>>;
        DEFINE FIELD system_prompt ON ai_agent TYPE string DEFAULT '';
        DEFINE FIELD temperature ON ai_agent TYPE float DEFAULT 0.8;
        DEFINE FIELD is_default ON ai_agent TYPE bool DEFAULT false;
        DEFINE FIELD created_at ON ai_agent TYPE datetime DEFAULT time::now();

        DEFINE TABLE ai_message SCHEMAFULL;
        DEFINE FIELD project ON ai_message TYPE record<project>;
        DEFINE INDEX idx_ai_message_project ON ai_message FIELDS project;
        DEFINE FIELD role ON ai_message TYPE string
            ASSERT $value IN ['user', 'assistant', 'system'];
        DEFINE FIELD content ON ai_message TYPE string;
        DEFINE FIELD created_at ON ai_message TYPE datetime DEFAULT time::now();

        DEFINE TABLE narrative_session SCHEMAFULL;
        DEFINE FIELD project ON narrative_session TYPE record<project>;
        DEFINE INDEX idx_narrative_session_project ON narrative_session FIELDS project;
        DEFINE FIELD title ON narrative_session TYPE string DEFAULT '';
        DEFINE FIELD scene ON narrative_session TYPE string DEFAULT '';
        DEFINE FIELD atmosphere ON narrative_session TYPE string DEFAULT '';
        DEFINE FIELD timeline_id ON narrative_session TYPE string DEFAULT 'main';
        DEFINE FIELD strand ON narrative_session TYPE string DEFAULT 'quest'
            ASSERT $value IN ['quest', 'fire', 'constellation'];
        DEFINE FIELD status ON narrative_session TYPE string DEFAULT 'active'
            ASSERT $value IN ['active', 'paused', 'ended'];
        DEFINE FIELD created_at ON narrative_session TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON narrative_session TYPE datetime DEFAULT time::now();

        DEFINE TABLE narrative_beat SCHEMAFULL;
        DEFINE FIELD session ON narrative_beat TYPE record<narrative_session>;
        DEFINE INDEX idx_narrative_beat_session ON narrative_beat FIELDS session;
        DEFINE INDEX idx_narrative_beat_session_order ON narrative_beat FIELDS session, sort_order;
        DEFINE FIELD beat_type ON narrative_beat TYPE string
            ASSERT $value IN ['narration', 'character_action', 'scene_change', 'author_intervention'];
        DEFINE FIELD character ON narrative_beat TYPE option<record<character>>;
        DEFINE FIELD character_name ON narrative_beat TYPE string DEFAULT '';
        DEFINE FIELD content ON narrative_beat TYPE string DEFAULT '';
        DEFINE FIELD metadata ON narrative_beat TYPE any DEFAULT {};
        DEFINE FIELD sort_order ON narrative_beat TYPE int DEFAULT 0;
        DEFINE FIELD timeline_id ON narrative_beat TYPE string DEFAULT 'main';
        DEFINE FIELD strand ON narrative_beat TYPE string DEFAULT 'quest'
            ASSERT $value IN ['quest', 'fire', 'constellation'];
        DEFINE FIELD hook_type ON narrative_beat TYPE option<string>;
        DEFINE FIELD hook_strength ON narrative_beat TYPE option<string>;
        DEFINE FIELD micro_payoffs ON narrative_beat TYPE any DEFAULT [];
        DEFINE FIELD created_at ON narrative_beat TYPE datetime DEFAULT time::now();

        DEFINE TABLE narrative_event SCHEMAFULL;
        DEFINE FIELD beat ON narrative_event TYPE record<narrative_beat>;
        DEFINE INDEX idx_narrative_event_beat ON narrative_event FIELDS beat;
        DEFINE FIELD session ON narrative_event TYPE record<narrative_session>;
        DEFINE INDEX idx_narrative_event_session ON narrative_event FIELDS session;
        DEFINE FIELD event_type ON narrative_event TYPE string;
        DEFINE FIELD character ON narrative_event TYPE option<record<character>>;
        DEFINE FIELD character_name ON narrative_event TYPE string DEFAULT '';
        DEFINE FIELD summary ON narrative_event TYPE string DEFAULT '';
        DEFINE FIELD detail ON narrative_event TYPE any DEFAULT {};
        DEFINE FIELD created_at ON narrative_event TYPE datetime DEFAULT time::now();

        DEFINE TABLE writing_review SCHEMAFULL;
        DEFINE FIELD session ON writing_review TYPE record<narrative_session>;
        DEFINE INDEX idx_writing_review_session ON writing_review FIELDS session;
        DEFINE FIELD beat ON writing_review TYPE record<narrative_beat>;
        DEFINE FIELD dimension ON writing_review TYPE string;
        DEFINE FIELD score ON writing_review TYPE float DEFAULT 0.0;
        DEFINE FIELD passed ON writing_review TYPE bool DEFAULT true;
        DEFINE FIELD issues ON writing_review TYPE any DEFAULT [];
        DEFINE FIELD summary ON writing_review TYPE string DEFAULT '';
        DEFINE FIELD created_at ON writing_review TYPE datetime DEFAULT time::now();

        DEFINE TABLE faction SCHEMAFULL;
        DEFINE FIELD project ON faction TYPE record<project>;
        DEFINE INDEX idx_faction_project ON faction FIELDS project;
        DEFINE FIELD name ON faction TYPE string;

        DEFINE TABLE character_state TYPE RELATION IN character OUT narrative_beat SCHEMAFULL;
        DEFINE FIELD emotion ON character_state TYPE string DEFAULT '';
        DEFINE FIELD location ON character_state TYPE string DEFAULT '';
        DEFINE FIELD knowledge ON character_state TYPE string DEFAULT '';
        DEFINE FIELD physical_state ON character_state TYPE string DEFAULT '';
        DEFINE FIELD created_at ON character_state TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_cs_in ON character_state FIELDS in;
        DEFINE INDEX idx_cs_out ON character_state FIELDS out;

        DEFINE TABLE character_relation TYPE RELATION IN character OUT character SCHEMAFULL;
        DEFINE FIELD project ON character_relation TYPE record<project>;
        DEFINE INDEX idx_character_relation_project ON character_relation FIELDS project;
        DEFINE FIELD relationship_type ON character_relation TYPE string DEFAULT '';
        DEFINE FIELD description ON character_relation TYPE string DEFAULT '';
        DEFINE FIELD start_chapter ON character_relation TYPE option<record<outline_node>>;
        DEFINE FIELD end_chapter ON character_relation TYPE option<record<outline_node>>;
        DEFINE FIELD created_at ON character_relation TYPE datetime DEFAULT time::now();

        DEFINE TABLE character_faction TYPE RELATION IN character OUT faction SCHEMAFULL;
        DEFINE FIELD project ON character_faction TYPE record<project>;
        DEFINE INDEX idx_character_faction_project ON character_faction FIELDS project;
        DEFINE FIELD role ON character_faction TYPE string DEFAULT '';
        DEFINE FIELD start_chapter ON character_faction TYPE option<record<outline_node>>;
        DEFINE FIELD end_chapter ON character_faction TYPE option<record<outline_node>>;
        DEFINE FIELD created_at ON character_faction TYPE datetime DEFAULT time::now();

        DEFINE TABLE workflow SCHEMAFULL;
        DEFINE FIELD name ON workflow TYPE string;
        DEFINE FIELD description ON workflow TYPE string DEFAULT '';
        DEFINE FIELD is_preset ON workflow TYPE bool DEFAULT false;
        DEFINE FIELD is_default ON workflow TYPE bool DEFAULT false;
        DEFINE FIELD step_count ON workflow TYPE int DEFAULT 0;
        DEFINE FIELD created_at ON workflow TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON workflow TYPE datetime DEFAULT time::now();

        DEFINE TABLE workflow_step SCHEMAFULL;
        DEFINE FIELD workflow ON workflow_step TYPE record<workflow>;
        DEFINE INDEX idx_workflow_step_workflow ON workflow_step FIELDS workflow;
        DEFINE INDEX idx_workflow_step_workflow_order ON workflow_step FIELDS workflow, sort_order;
        DEFINE FIELD sort_order ON workflow_step TYPE int DEFAULT 0;
        DEFINE FIELD step_type ON workflow_step TYPE string;
        DEFINE FIELD agent ON workflow_step TYPE option<record<ai_agent>>;
        DEFINE FIELD condition ON workflow_step TYPE option<object>;
        DEFINE FIELD config ON workflow_step TYPE object DEFAULT {};
        DEFINE FIELD enabled ON workflow_step TYPE bool DEFAULT true;
    "#).await?;

    Ok(())
}
