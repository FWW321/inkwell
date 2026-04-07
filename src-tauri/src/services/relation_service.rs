use crate::db::Store;
use crate::db::created_id;
use crate::db::models::{CharacterFaction, CharacterRelation, Faction};
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::{RecordId, Value};

const RELATION_SELECT: &str = "\
    SELECT id, project, \
         record::id(in) AS char_a_id, in.name AS char_a_name, \
         record::id(out) AS char_b_id, out.name AS char_b_name, \
         relationship_type, description, \
         if start_chapter != NONE { record::id(start_chapter) } else { NONE } AS start_chapter_id, start_chapter.title AS start_chapter_title, \
         if end_chapter != NONE { record::id(end_chapter) } else { NONE } AS end_chapter_id, end_chapter.title AS end_chapter_title, \
         created_at \
         FROM character_relation";

const FACTION_SELECT: &str = "\
    SELECT id, project, \
         record::id(in) AS character_id, in.name AS character_name, \
         record::id(out) AS faction_id, out.name AS faction_name, \
         role, \
         if start_chapter != NONE { record::id(start_chapter) } else { NONE } AS start_chapter_id, start_chapter.title AS start_chapter_title, \
         if end_chapter != NONE { record::id(end_chapter) } else { NONE } AS end_chapter_id, end_chapter.title AS end_chapter_title, \
         created_at \
         FROM character_faction";

pub async fn list_relations(db: &Db, project_id: &str) -> AppResult<Vec<CharacterRelation>> {
    db.query(RELATION_SELECT)
        .bind(("pid", RecordId::new("project", project_id)))
        .bind(("project", RecordId::new("project", project_id)))
        .await?
        .check()?
        .take::<Vec<CharacterRelation>>(0)
        .map_err(Into::into)
}

pub async fn create_relation(
    db: &Db,
    project_id: &str,
    char_a_id: &str,
    char_b_id: &str,
    relationship_type: &str,
    description: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterRelation> {
    let sql = "\
        RELATE type::record('character', $char_a)->character_relation->type::record('character', $char_b) \
         CONTENT { \
         project: type::record('project', $pid), \
         relationship_type: $rel_type, \
         description: $desc, \
         start_chapter: if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter: if $has_end { type::record('outline_node', $end_ch) } else { NONE } \
         }";

    let mut q = db
        .query(sql)
        .bind(("pid", project_id.to_string()))
        .bind(("char_a", char_a_id.to_string()))
        .bind(("char_b", char_b_id.to_string()))
        .bind(("rel_type", relationship_type.to_string()))
        .bind(("desc", description.to_string()))
        .bind(("has_start", start_chapter_id.is_some()))
        .bind(("has_end", end_chapter_id.is_some()));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    let created_id = q
        .await?
        .check()?
        .take::<Option<Value>>(0)?
        .map(|v| created_id(&v))
        .transpose()?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("create relation failed")))?;

    db.query(RELATION_SELECT)
        .bind((
            "rid",
            RecordId::new("character_relation", created_id.as_str()),
        ))
        .await?
        .check()?
        .take::<Option<CharacterRelation>>(0)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("fetch relation failed")))
}

pub async fn update_relation(
    db: &Db,
    id: &str,
    relationship_type: &str,
    description: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterRelation> {
    let sql = "\
        UPDATE type::record('character_relation', $rid) MERGE { \
         relationship_type: $rel_type, \
         description: $desc, \
         start_chapter: if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter: if $has_end { type::record('outline_node', $end_ch) } else { NONE } \
         }";

    let mut q = db
        .query(sql)
        .bind(("rid", id.to_string()))
        .bind(("rel_type", relationship_type.to_string()))
        .bind(("desc", description.to_string()))
        .bind(("has_start", start_chapter_id.is_some()))
        .bind(("has_end", end_chapter_id.is_some()));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    q.await?
        .check()?
        .take::<Option<Value>>(0)?
        .map(|_| ())
        .ok_or_else(|| AppError::NotFound("关系不存在".to_string()))?;

    db.query(RELATION_SELECT)
        .bind(("rid", RecordId::new("character_relation", id)))
        .await?
        .check()?
        .take::<Option<CharacterRelation>>(0)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("fetch relation failed")))
}

pub async fn delete_relation(db: &Db, id: &str) -> AppResult<()> {
    Store::new(db).delete("character_relation", id).await
}

pub async fn list_factions(db: &Db, project_id: &str) -> AppResult<Vec<CharacterFaction>> {
    db.query(FACTION_SELECT)
        .bind(("pid", RecordId::new("project", project_id)))
        .await?
        .check()?
        .take::<Vec<CharacterFaction>>(0)
        .map_err(Into::into)
}

pub async fn create_faction(
    db: &Db,
    project_id: &str,
    character_id: &str,
    faction_name: &str,
    role: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterFaction> {
    let sql = "\
        BEGIN; \
         LET $existing = (SELECT VALUE id FROM faction WHERE project = $pid AND name = $fname LIMIT 1); \
         LET $fid = IF $existing { $existing } ELSE { (CREATE faction CONTENT { project: type::record('project', $pid), name: $fname }).id }; \
         RELATE type::record('character', $cid)->character_faction->$fid \
         SET project = type::record('project', $pid), \
         role = $role, \
         start_chapter = if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter = if $has_end { type::record('outline_node', $end_ch) } else { NONE }; \
         COMMIT";

    let mut q = db
        .query(sql)
        .bind(("pid", project_id.to_string()))
        .bind(("cid", character_id.to_string()))
        .bind(("fname", faction_name.to_string()))
        .bind(("role", role.to_string()))
        .bind(("has_start", start_chapter_id.is_some()))
        .bind(("has_end", end_chapter_id.is_some()));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    let created_id = q
        .await?
        .check()?
        .take::<Option<Value>>(3)?
        .map(|v| created_id(&v))
        .transpose()?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("create faction failed")))?;

    db.query(FACTION_SELECT)
        .bind((
            "rid",
            RecordId::new("character_faction", created_id.as_str()),
        ))
        .await?
        .check()?
        .take::<Option<CharacterFaction>>(0)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("fetch faction failed")))
}

pub async fn update_faction(
    db: &Db,
    id: &str,
    faction_name: &str,
    role: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterFaction> {
    let existing: Option<Value> = db.select(("character_faction", id)).await?;
    if existing.is_none() {
        return Err(AppError::NotFound("势力归属不存在".to_string()));
    }

    let sql = "\
        BEGIN; \
         LET $existing = (SELECT VALUE id FROM faction WHERE name = $fname LIMIT 1); \
         LET $fid = IF $existing { $existing } ELSE { (CREATE faction CONTENT { name: $fname }).id }; \
         UPDATE type::record('character_faction', $rid) MERGE { \
         out: $fid, \
         role: $role, \
         start_chapter: if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter: if $has_end { type::record('outline_node', $end_ch) } else { NONE } \
         }; \
         COMMIT";

    let mut q = db
        .query(sql)
        .bind(("rid", id.to_string()))
        .bind(("fname", faction_name.to_string()))
        .bind(("role", role.to_string()))
        .bind(("has_start", start_chapter_id.is_some()))
        .bind(("has_end", end_chapter_id.is_some()));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    q.await?
        .check()?
        .take::<Option<Value>>(3)?
        .ok_or_else(|| AppError::NotFound("势力归属不存在".to_string()))?;

    db.query(FACTION_SELECT)
        .bind(("rid", RecordId::new("character_faction", id)))
        .await?
        .check()?
        .take::<Option<CharacterFaction>>(0)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("fetch faction failed")))
}

pub async fn delete_faction(db: &Db, id: &str) -> AppResult<()> {
    Store::new(db).delete("character_faction", id).await
}

pub async fn list_faction_names(db: &Db, project_id: &str) -> AppResult<Vec<Faction>> {
    Store::new(db)
        .find::<Faction>("faction")
        .project("id, project, name")
        .filter_ref("project", "project", project_id)
        .order("name")
        .all()
        .await
}
