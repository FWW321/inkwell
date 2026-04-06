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

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<RecordId, D::Error> {
        let key_str: String = Deserialize::deserialize(d)?;
        Ok(RecordId::new("placeholder", key_str.as_str()))
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
        let opt: Option<String> = Deserialize::deserialize(d)?;
        Ok(opt.map(|k| RecordId::new("placeholder", k.as_str())))
    }
}
