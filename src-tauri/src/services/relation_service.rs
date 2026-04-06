use crate::db::get_created_id;
use crate::db::models::{CharacterFaction, CharacterRelation, Faction};
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::{RecordId, ToSql, Value};

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
    db.query(format!("{RELATION_SELECT} WHERE project = $pid ORDER BY created_at DESC"))
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<CharacterRelation>>(0).map_err(Into::into)
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
    let start_ch_val = match start_chapter_id {
        Some(sc) => format!("type::record('outline_node', '{}')", sc),
        None => "NONE".to_string(),
    };
    let end_ch_val = match end_chapter_id {
        Some(ec) => format!("type::record('outline_node', '{}')", ec),
        None => "NONE".to_string(),
    };

    let sql = format!(
        "RELATE character:{a}->character_relation->character:{b} \
         CONTENT {{ \
         project: type::record('project', $pid), \
         relationship_type: $rel_type, \
         description: $desc, \
         start_chapter: {start_ch}, \
         end_chapter: {end_ch} \
         }}",
        a = char_a_id,
        b = char_b_id,
        start_ch = start_ch_val,
        end_ch = end_ch_val,
    );

    let created_id = db.query(&sql)
        .bind(("pid", project_id.to_string()))
        .bind(("rel_type", relationship_type.to_string()))
        .bind(("desc", description.to_string()))
        .await?.check()?.take::<Option<Value>>(0)?
        .map(|v| get_created_id(&v))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Internal("create relation failed".into()))?;

    db.query(format!("{RELATION_SELECT} WHERE id = $rid"))
        .bind(("rid", RecordId::new("character_relation", created_id.as_str())))
        .await?.check()?.take::<Option<CharacterRelation>>(0)?
        .ok_or_else(|| AppError::Internal("fetch relation failed".into()))
}

pub async fn update_relation(
    db: &Db,
    id: &str,
    relationship_type: &str,
    description: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterRelation> {
    let start_ch_val = match start_chapter_id {
        Some(sc) => format!("type::record('outline_node', '{}')", sc),
        None => "NONE".to_string(),
    };
    let end_ch_val = match end_chapter_id {
        Some(ec) => format!("type::record('outline_node', '{}')", ec),
        None => "NONE".to_string(),
    };

    let sql = format!(
        "UPDATE type::record('character_relation', '{id}') MERGE {{ \
         relationship_type: $rel_type, \
         description: $desc, \
         start_chapter: {start_ch}, \
         end_chapter: {end_ch} \
         }}",
        id = id,
        start_ch = start_ch_val,
        end_ch = end_ch_val,
    );

    db.query(&sql)
        .bind(("rel_type", relationship_type.to_string()))
        .bind(("desc", description.to_string()))
        .await?.check()?.take::<Option<Value>>(0)?
        .map(|_| ())
        .ok_or_else(|| AppError::NotFound("关系不存在".to_string()))?;

    db.query(format!("{RELATION_SELECT} WHERE id = $rid"))
        .bind(("rid", RecordId::new("character_relation", id)))
        .await?.check()?.take::<Option<CharacterRelation>>(0)?
        .ok_or_else(|| AppError::Internal("fetch relation failed".into()))
}

pub async fn delete_relation(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<Value> = db.delete(("character_relation", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound("关系不存在".to_string()));
    }
    Ok(())
}

pub async fn list_factions(db: &Db, project_id: &str) -> AppResult<Vec<CharacterFaction>> {
    db.query(format!("{FACTION_SELECT} WHERE project = $pid ORDER BY created_at DESC"))
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<CharacterFaction>>(0).map_err(Into::into)
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
    let start_ch_val = match start_chapter_id {
        Some(sc) => format!("type::record('outline_node', '{}')", sc),
        None => "NONE".to_string(),
    };
    let end_ch_val = match end_chapter_id {
        Some(ec) => format!("type::record('outline_node', '{}')", ec),
        None => "NONE".to_string(),
    };

    let sql = format!(
        "BEGIN; \
         LET $existing = (SELECT VALUE id FROM faction WHERE project = $pid AND name = $fname LIMIT 1); \
         LET $fid = IF $existing {{ $existing }} ELSE {{ (CREATE faction CONTENT {{ project: type::record('project', $pid), name: $fname }}).id }}; \
         RELATE character:{cid}->character_faction->$fid \
         SET project = type::record('project', $pid), \
         role = $role, \
         start_chapter = {start_ch}, \
         end_chapter = {end_ch}; \
         COMMIT",
        cid = character_id,
        start_ch = start_ch_val,
        end_ch = end_ch_val,
    );

    let created_id = db.query(&sql)
        .bind(("pid", project_id.to_string()))
        .bind(("fname", faction_name.to_string()))
        .bind(("role", role.to_string()))
        .await?.check()?.take::<Option<Value>>(3)?
        .map(|v| get_created_id(&v))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Internal("create faction failed".into()))?;

    db.query(format!("{FACTION_SELECT} WHERE id = $rid"))
        .bind(("rid", RecordId::new("character_faction", created_id.as_str())))
        .await?.check()?.take::<Option<CharacterFaction>>(0)?
        .ok_or_else(|| AppError::Internal("fetch faction failed".into()))
}

pub async fn update_faction(
    db: &Db,
    id: &str,
    faction_name: &str,
    role: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterFaction> {
    let start_ch_val = match start_chapter_id {
        Some(sc) => format!("type::record('outline_node', '{}')", sc),
        None => "NONE".to_string(),
    };
    let end_ch_val = match end_chapter_id {
        Some(ec) => format!("type::record('outline_node', '{}')", ec),
        None => "NONE".to_string(),
    };

    let existing: Option<Value> = db.select(("character_faction", id)).await?;
    if existing.is_none() {
        return Err(AppError::NotFound("势力归属不存在".to_string()));
    }

    let sql = format!(
        "BEGIN; \
         LET $existing = (SELECT VALUE id FROM faction WHERE name = $fname LIMIT 1); \
         LET $fid = IF $existing {{ $existing }} ELSE {{ (CREATE faction CONTENT {{ name: $fname }}).id }}; \
         UPDATE type::record('character_faction', '{id}') MERGE {{ \
         out: $fid, \
         role: $role, \
         start_chapter: {start_ch}, \
         end_chapter: {end_ch} \
         }}; \
         COMMIT",
        id = id,
        start_ch = start_ch_val,
        end_ch = end_ch_val,
    );

    db.query(&sql)
        .bind(("fname", faction_name.to_string()))
        .bind(("role", role.to_string()))
        .await?.check()?
        .take::<Option<Value>>(3)?
        .ok_or_else(|| AppError::NotFound("势力归属不存在".to_string()))?;

    db.query(format!("{FACTION_SELECT} WHERE id = $rid"))
        .bind(("rid", RecordId::new("character_faction", id)))
        .await?.check()?.take::<Option<CharacterFaction>>(0)?
        .ok_or_else(|| AppError::Internal("fetch faction failed".into()))
}

pub async fn delete_faction(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<Value> = db.delete(("character_faction", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound("势力归属不存在".to_string()));
    }
    Ok(())
}

pub async fn list_faction_names(db: &Db, project_id: &str) -> AppResult<Vec<Faction>> {
    db.query("SELECT id, project, name FROM faction WHERE project = $pid ORDER BY name")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<Faction>>(0).map_err(Into::into)
}
