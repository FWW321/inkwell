use serde::{Deserialize, Deserializer, Serializer};
use surrealdb::types::{RecordId, RecordIdKey};

pub mod rid {
    use super::*;

    pub fn serialize<S: Serializer>(rid: &RecordId, s: S) -> Result<S::Ok, S::Error> {
        match &rid.key {
            RecordIdKey::String(k) => s.serialize_str(k),
            RecordIdKey::Uuid(k) => s.serialize_str(&k.to_string()),
            RecordIdKey::Number(n) => s.serialize_str(&n.to_string()),
            _ => s.serialize_str(&format!("{:?}", rid.key)),
        }
    }

    #[derive(Deserialize)]
    struct IdParts {
        table: String,
        key: serde_json::Value,
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<RecordId, D::Error> {
        let value: serde_json::Value = Deserialize::deserialize(d)?;
        match value {
            serde_json::Value::String(key_str) => {
                Ok(RecordId::new("placeholder", key_str.as_str()))
            }
            serde_json::Value::Object(map) => {
                let parts: IdParts = serde_json::from_value(serde_json::Value::Object(map))
                    .map_err(serde::de::Error::custom)?;
                let key = match parts.key {
                    serde_json::Value::String(s) => RecordIdKey::String(s),
                    serde_json::Value::Number(n) => {
                        RecordIdKey::Number(n.as_i64().ok_or_else(|| {
                            serde::de::Error::custom("invalid record id key number")
                        })?)
                    }
                    other => RecordIdKey::String(other.to_string()),
                };
                Ok(RecordId {
                    table: parts.table.into(),
                    key,
                })
            }
            _ => Err(serde::de::Error::custom(
                "expected string or object for RecordId",
            )),
        }
    }
}

pub mod opt_rid {
    use super::*;

    pub fn serialize<S: Serializer>(rid: &Option<RecordId>, s: S) -> Result<S::Ok, S::Error> {
        match rid {
            Some(r) => rid::serialize(r, s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<RecordId>, D::Error> {
        let opt: Option<serde_json::Value> = Option::deserialize(d)?;
        match opt {
            None => Ok(None),
            Some(value) => {
                let rid: RecordId =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Some(rid))
            }
        }
    }
}
