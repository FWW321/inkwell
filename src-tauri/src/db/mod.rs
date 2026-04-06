pub mod models;

pub use models::*;

use surrealdb::types::{ToSql, Value};

pub fn get_created_id(v: &Value) -> String {
    match v {
        Value::Object(map) => match map.get("id") {
            Some(Value::RecordId(rid)) => rid.key.to_sql(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}
