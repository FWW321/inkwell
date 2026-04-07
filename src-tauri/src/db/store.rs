use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use surrealdb::types::SurrealValue;

use crate::error::{AppError, AppResult};
use crate::state::Db;

macro_rules! exec_query {
    ($db:expr, $sql:expr, $binds:expr) => {{
        let mut q = $db.query($sql);
        for (key, val) in $binds {
            q = q.bind((key, val));
        }
        q.await?.check()?
    }};
}

pub struct Store<'a>(&'a Db);

impl<'a> Store<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self(db)
    }

    pub fn db(&self) -> &Db {
        self.0
    }

    pub async fn get<T: DeserializeOwned + SurrealValue>(
        &self,
        table: &str,
        id: &str,
    ) -> AppResult<T> {
        self.0
            .select((table, id))
            .await?
            .ok_or_else(|| AppError::NotFound(format!("{table} not found: {id}")))
    }

    pub async fn delete(&self, table: &str, id: &str) -> AppResult<()> {
        let deleted: Option<serde_json::Value> = self.0.delete((table, id)).await?;
        if deleted.is_none() {
            return Err(AppError::NotFound(format!("{table} not found: {id}")));
        }
        Ok(())
    }

    pub async fn count(&self, table: &str) -> AppResult<i64> {
        let sql = format!("SELECT VALUE count() FROM (SELECT * FROM {table}) GROUP BY ALL");
        let mut resp = self.0.query(sql).await?.check()?;
        let result: Option<i64> = resp.take::<Option<i64>>(0)?;
        Ok(result.unwrap_or(0))
    }

    pub async fn is_empty(&self, table: &str) -> AppResult<bool> {
        Ok(self.count(table).await? == 0)
    }

    pub fn find<T: DeserializeOwned + SurrealValue>(&'a self, table: &str) -> Find<'a, T> {
        Find {
            db: self.0,
            table: table.to_owned(),
            wheres: Vec::new(),
            binds: Vec::new(),
            order: None,
            limit: None,
            projection: None,
            fetch: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn content(&'a self, table: &str) -> Content<'a> {
        Content {
            db: self.0,
            table: table.to_owned(),
            merge_id: None,
            fields: Vec::new(),
            touch: false,
        }
    }

    pub fn update(&'a self, table: &str, id: &str) -> Update<'a> {
        Update {
            db: self.0,
            table: table.to_owned(),
            id: id.to_owned(),
            sets: Vec::new(),
            touch: false,
        }
    }
}

pub struct Find<'a, T> {
    db: &'a Db,
    table: String,
    wheres: Vec<String>,
    binds: Vec<(String, serde_json::Value)>,
    order: Option<String>,
    limit: Option<usize>,
    projection: Option<String>,
    fetch: Vec<String>,
    _phantom: PhantomData<T>,
}

impl<'a, T: DeserializeOwned + SurrealValue> Find<'a, T> {
    pub fn filter_eq(mut self, field: &str, value: impl serde::Serialize) -> Self {
        let idx = self.wheres.len();
        let param = format!("__w{idx}");
        let val = serde_json::to_value(value).unwrap_or_default();
        self.wheres.push(format!("{field} = ${param}"));
        self.binds.push((param, val));
        self
    }

    pub fn filter_ref(mut self, field: &str, ref_table: &str, id: &str) -> Self {
        let idx = self.wheres.len();
        let param = format!("__w{idx}");
        self.wheres
            .push(format!("{field} = type::record('{ref_table}', ${param})"));
        self.binds
            .push((param, serde_json::Value::String(id.to_owned())));
        self
    }

    pub fn filter_is_none(mut self, field: &str) -> Self {
        self.wheres.push(format!("{field} IS NONE"));
        self
    }

    pub fn order(mut self, expr: &str) -> Self {
        self.order = Some(expr.to_owned());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn project(mut self, expr: &str) -> Self {
        self.projection = Some(expr.to_owned());
        self
    }

    pub fn fetch(mut self, fields: &[&str]) -> Self {
        self.fetch = fields.iter().map(|s| (*s).to_owned()).collect();
        self
    }

    pub async fn all(self) -> AppResult<Vec<T>> {
        let (sql, binds) = self.build_sql();
        let mut resp = exec_query!(self.db, &sql, binds);
        Ok(resp.take::<Vec<T>>(0)?)
    }

    pub async fn one(self) -> AppResult<T> {
        let (sql, binds) = self.build_sql();
        let mut resp = exec_query!(self.db, &sql, binds);
        let result: Option<T> = resp.take::<Option<T>>(0)?;
        result.ok_or_else(|| AppError::NotFound(format!("{} not found", self.table)))
    }

    pub async fn count(self) -> AppResult<i64> {
        let mut sql = format!("SELECT VALUE count() FROM (SELECT * FROM {})", self.table);
        if !self.wheres.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.wheres.join(" AND ")));
        }
        let mut resp = exec_query!(self.db, &sql, self.binds);
        let result: Option<i64> = resp.take::<Option<i64>>(0)?;
        Ok(result.unwrap_or(0))
    }

    fn build_sql(&self) -> (String, Vec<(String, serde_json::Value)>) {
        let select = self.projection.as_deref().unwrap_or("*");
        let where_clause = if self.wheres.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.wheres.join(" AND "))
        };
        let mut sql = format!("SELECT {select} FROM {}{where_clause}", self.table);
        if let Some(ref o) = self.order {
            sql.push_str(&format!(" ORDER BY {o}"));
        }
        if let Some(l) = self.limit {
            sql.push_str(&format!(" LIMIT {l}"));
        }
        if !self.fetch.is_empty() {
            sql.push_str(&format!(" FETCH {}", self.fetch.join(", ")));
        }
        (sql, self.binds.clone())
    }
}

enum FieldValue {
    Plain(serde_json::Value),
    Ref {
        table: String,
        id: String,
    },
    OptRef {
        table: String,
        has: bool,
        id: String,
    },
}

struct Field {
    name: String,
    value: FieldValue,
}

pub struct Content<'a> {
    db: &'a Db,
    table: String,
    merge_id: Option<String>,
    fields: Vec<Field>,
    touch: bool,
}

impl<'a> Content<'a> {
    pub fn field(mut self, name: &str, value: impl serde::Serialize) -> Self {
        self.fields.push(Field {
            name: name.to_owned(),
            value: FieldValue::Plain(serde_json::to_value(value).unwrap_or_default()),
        });
        self
    }

    pub fn ref_id(mut self, name: &str, ref_table: &str, id: &str) -> Self {
        self.fields.push(Field {
            name: name.to_owned(),
            value: FieldValue::Ref {
                table: ref_table.to_owned(),
                id: id.to_owned(),
            },
        });
        self
    }

    pub fn opt_ref(mut self, name: &str, ref_table: &str, id: Option<&str>) -> Self {
        self.fields.push(Field {
            name: name.to_owned(),
            value: FieldValue::OptRef {
                table: ref_table.to_owned(),
                has: id.is_some(),
                id: id.map(|s| s.to_owned()).unwrap_or_default(),
            },
        });
        self
    }

    pub fn touch(mut self) -> Self {
        self.touch = true;
        self
    }

    pub fn merge_mode(mut self, id: &str) -> Self {
        self.merge_id = Some(id.to_owned());
        self
    }

    pub async fn exec<T: DeserializeOwned + SurrealValue>(self) -> AppResult<T> {
        let (sql, binds) = self.build_sql();
        let mut resp = exec_query!(self.db, &sql, binds);
        let result: Option<T> = resp.take::<Option<T>>(0)?;
        result.ok_or_else(|| AppError::Internal(anyhow::anyhow!("{} operation failed", self.table)))
    }

    fn build_sql(&self) -> (String, Vec<(String, serde_json::Value)>) {
        let mut parts: Vec<String> = Vec::new();
        let mut binds: Vec<(String, serde_json::Value)> = Vec::new();

        if let Some(id) = &self.merge_id {
            binds.push(("__id".to_owned(), serde_json::Value::String(id.clone())));
        }

        for field in &self.fields {
            let pname = format!("__f_{}", field.name);
            match &field.value {
                FieldValue::Plain(val) => {
                    binds.push((pname.clone(), val.clone()));
                    parts.push(format!("{}: ${pname}", field.name));
                }
                FieldValue::Ref { table, id } => {
                    parts.push(format!(
                        "{}: type::record('{}', '{}')",
                        field.name, table, id
                    ));
                }
                FieldValue::OptRef { table, has, id } => {
                    let has_p = format!("__fh_{}", field.name);
                    let id_p = pname;
                    binds.push((has_p.clone(), serde_json::Value::Bool(*has)));
                    binds.push((id_p.clone(), serde_json::Value::String(id.clone())));
                    parts.push(format!(
                        "{}: if ${has_p} {{ type::record('{table}', ${id_p}) }} else {{ NONE }}",
                        field.name
                    ));
                }
            }
        }

        if self.touch {
            parts.push("updated_at: time::now()".to_owned());
        }

        let body = parts.join(", ");
        let sql = if self.merge_id.is_some() {
            format!("UPDATE type::record($__id) MERGE {{ {body} }}")
        } else {
            format!("CREATE {} CONTENT {{ {body} }}", self.table)
        };
        (sql, binds)
    }
}

enum SetOp {
    Set(serde_json::Value),
    Expr(String),
    SetNone,
}

pub struct Update<'a> {
    db: &'a Db,
    table: String,
    id: String,
    sets: Vec<(String, SetOp)>,
    touch: bool,
}

impl<'a> Update<'a> {
    pub fn set(mut self, field: &str, value: impl serde::Serialize) -> Self {
        let val = serde_json::to_value(value).unwrap_or_default();
        self.sets.push((field.to_owned(), SetOp::Set(val)));
        self
    }

    pub fn set_expr(mut self, field: &str, expr: &str) -> Self {
        self.sets
            .push((field.to_owned(), SetOp::Expr(expr.to_owned())));
        self
    }

    pub fn set_none(mut self, field: &str) -> Self {
        self.sets.push((field.to_owned(), SetOp::SetNone));
        self
    }

    pub fn touch(mut self) -> Self {
        self.touch = true;
        self
    }

    pub async fn exec(self) -> AppResult<()> {
        let (sql, binds) = self.build_sql();
        exec_query!(self.db, &sql, binds);
        Ok(())
    }

    pub async fn get<T: DeserializeOwned + SurrealValue + 'static>(self) -> AppResult<T> {
        let table = self.table.clone();
        let id = self.id.clone();
        let db = self.db;
        self.exec().await?;
        Store(db).get(&table, &id).await
    }

    fn build_sql(&self) -> (String, Vec<(String, serde_json::Value)>) {
        let mut clauses: Vec<String> = Vec::new();
        let mut binds: Vec<(String, serde_json::Value)> = Vec::new();

        binds.push((
            "__id".to_owned(),
            serde_json::Value::String(self.id.clone()),
        ));

        for (i, (field, op)) in self.sets.iter().enumerate() {
            let param = format!("__s{i}");
            match op {
                SetOp::Set(val) => {
                    binds.push((param.clone(), val.clone()));
                    clauses.push(format!("{field} = ${param}"));
                }
                SetOp::Expr(expr) => {
                    clauses.push(format!("{field} = {expr}"));
                }
                SetOp::SetNone => {
                    clauses.push(format!("{field} = NONE"));
                }
            }
        }

        if self.touch {
            clauses.push("updated_at = time::now()".to_owned());
        }

        let sql = format!("UPDATE type::record($__id) SET {}", clauses.join(", "));
        (sql, binds)
    }
}
