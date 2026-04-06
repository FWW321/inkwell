use crate::db::models::{CharacterFaction, CharacterFactionWithNames, CharacterRelation, CharacterRelationWithNames, Faction};
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::{RecordId, ToSql};

pub async fn list_relations(db: &Db, project_id: &str) -> AppResult<Vec<CharacterRelationWithNames>> {
    db.query(
        "SELECT id, project, \
         in.id AS char_a_id, in.name AS char_a_name, \
         out.id AS char_b_id, out.name AS char_b_name, \
         relationship_type, description, \
         start_chapter.id AS start_chapter_id, start_chapter.title AS start_chapter_title, \
         end_chapter.id AS end_chapter_id, end_chapter.title AS end_chapter_title, \
         created_at \
         FROM character_relation \
         WHERE project = $pid \
         ORDER BY created_at DESC"
    )
    .bind(("pid", RecordId::new("project", project_id)))
    .await?.take::<Vec<CharacterRelationWithNames>>(0).map_err(Into::into)
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
    let has_start = start_chapter_id.is_some();
    let has_end = end_chapter_id.is_some();

    let mut q = db.query(
        "RELATE type::record('character', $char_a)->character_relation->type::record('character', $char_b) \
         SET project = type::record('project', $pid), \
         relationship_type = $rel_type, \
         description = $desc, \
         start_chapter = if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter = if $has_end { type::record('outline_node', $end_ch) } else { NONE }"
    )
    .bind(("pid", project_id.to_string()))
    .bind(("char_a", char_a_id.to_string()))
    .bind(("char_b", char_b_id.to_string()))
    .bind(("rel_type", relationship_type.to_string()))
    .bind(("desc", description.to_string()))
    .bind(("has_start", has_start))
    .bind(("has_end", has_end));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    q.await?.take::<Option<CharacterRelation>>(0)?
        .ok_or_else(|| AppError::Internal("create relation failed".into()))
}

pub async fn update_relation(
    db: &Db,
    id: &str,
    relationship_type: &str,
    description: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterRelation> {
    let has_start = start_chapter_id.is_some();
    let has_end = end_chapter_id.is_some();

    let mut q = db.query(
        "UPDATE type::record($id) MERGE { \
         relationship_type = $rel_type, \
         description = $desc, \
         start_chapter = if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter = if $has_end { type::record('outline_node', $end_ch) } else { NONE } \
         }"
    )
    .bind(("id", RecordId::new("character_relation", id)))
    .bind(("rel_type", relationship_type.to_string()))
    .bind(("desc", description.to_string()))
    .bind(("has_start", has_start))
    .bind(("has_end", has_end));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    q.await?.take::<Option<CharacterRelation>>(0)?
        .ok_or_else(|| AppError::NotFound("关系不存在".to_string()))
}

pub async fn delete_relation(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<CharacterRelation> = db.delete(("character_relation", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound("关系不存在".to_string()));
    }
    Ok(())
}

pub async fn list_factions(db: &Db, project_id: &str) -> AppResult<Vec<CharacterFactionWithNames>> {
    db.query(
        "SELECT id, project, \
         in.id AS character_id, in.name AS character_name, \
         out.id AS faction_id, out.name AS faction_name, \
         role, \
         start_chapter.id AS start_chapter_id, start_chapter.title AS start_chapter_title, \
         end_chapter.id AS end_chapter_id, end_chapter.title AS end_chapter_title, \
         created_at \
         FROM character_faction \
         WHERE project = $pid \
         ORDER BY created_at DESC"
    )
    .bind(("pid", RecordId::new("project", project_id)))
    .await?.take::<Vec<CharacterFactionWithNames>>(0).map_err(Into::into)
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
    let has_start = start_chapter_id.is_some();
    let has_end = end_chapter_id.is_some();

    let mut q = db.query(
        "BEGIN; \
         LET $faction = (SELECT id FROM faction WHERE project = $pid AND name = $fname LIMIT 1); \
         LET $fid = IF $faction { $faction.id } ELSE { (CREATE faction CONTENT { project: type::record('project', $pid), name: $fname }).id }; \
         RELATE type::record('character', $char_id)->character_faction->$fid \
         SET project = type::record('project', $pid), \
         role = $role, \
         start_chapter = if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter = if $has_end { type::record('outline_node', $end_ch) } else { NONE }; \
         COMMIT"
    )
    .bind(("pid", project_id.to_string()))
    .bind(("char_id", character_id.to_string()))
    .bind(("fname", faction_name.to_string()))
    .bind(("role", role.to_string()))
    .bind(("has_start", has_start))
    .bind(("has_end", has_end));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    q.await?.check()?
        .take::<Option<CharacterFaction>>(3)?
        .ok_or_else(|| AppError::Internal("create faction failed".into()))
}

pub async fn update_faction(
    db: &Db,
    id: &str,
    faction_name: &str,
    role: &str,
    start_chapter_id: Option<&str>,
    end_chapter_id: Option<&str>,
) -> AppResult<CharacterFaction> {
    let has_start = start_chapter_id.is_some();
    let has_end = end_chapter_id.is_some();

    let existing: Option<CharacterFaction> = db.select(("character_faction", id)).await?;
    let edge = existing.ok_or_else(|| AppError::NotFound("势力归属不存在".to_string()))?;
    let project_id = edge.project.key.to_sql();

    let mut q = db.query(
        "BEGIN; \
         LET $faction = (SELECT id FROM faction WHERE project = type::record('project', $pid) AND name = $fname LIMIT 1); \
         LET $fid = IF $faction { $faction.id } ELSE { (CREATE faction CONTENT { project: type::record('project', $pid), name: $fname }).id }; \
         UPDATE type::record($id) MERGE { \
         out = $fid, \
         role = $role, \
         start_chapter = if $has_start { type::record('outline_node', $start_ch) } else { NONE }, \
         end_chapter = if $has_end { type::record('outline_node', $end_ch) } else { NONE } \
         }; \
         COMMIT"
    )
    .bind(("id", RecordId::new("character_faction", id)))
    .bind(("pid", project_id))
    .bind(("fname", faction_name.to_string()))
    .bind(("role", role.to_string()))
    .bind(("has_start", has_start))
    .bind(("has_end", has_end));
    if let Some(sc) = start_chapter_id {
        q = q.bind(("start_ch", sc.to_string()));
    }
    if let Some(ec) = end_chapter_id {
        q = q.bind(("end_ch", ec.to_string()));
    }

    q.await?.check()?
        .take::<Option<CharacterFaction>>(3)?
        .ok_or_else(|| AppError::NotFound("势力归属不存在".to_string()))
}

pub async fn delete_faction(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<CharacterFaction> = db.delete(("character_faction", id)).await?;
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
