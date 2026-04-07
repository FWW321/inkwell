pub mod models;
pub mod store;

pub use models::*;
pub use store::Store;

use crate::error::{AppError, AppResult};
use surrealdb::types::{RecordIdKey, ToSql, Value};

pub fn created_id(v: &Value) -> AppResult<String> {
    match v {
        Value::Object(map) => match map.get("id") {
            Some(Value::RecordId(rid)) => Ok(rid.key.to_sql()),
            _ => Err(AppError::Internal(anyhow::anyhow!(
                "created record missing id field"
            ))),
        },
        _ => Err(AppError::Internal(anyhow::anyhow!(
            "expected object for created record"
        ))),
    }
}
