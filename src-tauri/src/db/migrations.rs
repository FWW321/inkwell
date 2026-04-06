use crate::error::AppResult;
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            author TEXT NOT NULL DEFAULT '',
            language TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'ongoing' CHECK(status IN ('ongoing', 'completed', 'hiatus')),
            cover_url TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS outline_nodes (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            parent_id TEXT,
            node_type TEXT NOT NULL CHECK(node_type IN ('volume', 'chapter')),
            title TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            content_json TEXT NOT NULL DEFAULT '[]',
            word_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(parent_id) REFERENCES outline_nodes(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            avatar_url TEXT,
            description TEXT NOT NULL DEFAULT '',
            personality TEXT NOT NULL DEFAULT '',
            background TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS worldview_entries (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            category TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS ai_models (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            api_key TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            base_url TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(base_url, api_key, model),
            UNIQUE(name)
        );

        CREATE TABLE IF NOT EXISTS ai_agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            model_id TEXT NOT NULL REFERENCES ai_models(id) ON DELETE CASCADE,
            system_prompt TEXT NOT NULL DEFAULT '',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(name)
        );

        CREATE TABLE IF NOT EXISTS ai_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        ",
    )?;

    let has_project_new_cols: bool = conn
        .prepare("SELECT author FROM projects LIMIT 0")
        .is_ok();

    if !has_project_new_cols {
        conn.execute_batch(
            "ALTER TABLE projects ADD COLUMN author TEXT NOT NULL DEFAULT '';
             ALTER TABLE projects ADD COLUMN language TEXT NOT NULL DEFAULT '';
             ALTER TABLE projects ADD COLUMN tags TEXT NOT NULL DEFAULT '';
             ALTER TABLE projects ADD COLUMN status TEXT NOT NULL DEFAULT 'ongoing';",
        )?;
    }

    let has_diff_cols: bool = conn
        .prepare("SELECT diff_original FROM outline_nodes LIMIT 0")
        .is_ok();

    if !has_diff_cols {
        conn.execute_batch(
            "ALTER TABLE outline_nodes ADD COLUMN diff_original TEXT;
             ALTER TABLE outline_nodes ADD COLUMN diff_new TEXT;
             ALTER TABLE outline_nodes ADD COLUMN diff_mode TEXT;",
        )?;
    }

    let has_ai_agents: bool = conn
        .prepare("SELECT id FROM ai_agents LIMIT 0")
        .is_ok();

    if !has_ai_agents {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ai_agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                model_id TEXT NOT NULL REFERENCES ai_models(id) ON DELETE CASCADE,
                system_prompt TEXT NOT NULL DEFAULT '',
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(name)
            );",
        )?;
    }

    Ok(())
}
