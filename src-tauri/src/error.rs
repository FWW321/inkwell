use serde::Serialize;
use surrealdb::Error as SurrealError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Ai(String),
    #[error("服务内部错误")]
    Internal(#[source] anyhow::Error),
    #[error(transparent)]
    Database(#[from] SurrealError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let msg: &str = match self {
            Self::NotFound(s) | Self::Validation(s) | Self::Ai(s) => s,
            Self::Internal(_) => "服务内部错误",
            Self::Database(e) => return serializer.serialize_str(&format!("数据库错误: {}", e)),
            Self::Serialization(e) => {
                return serializer.serialize_str(&format!("序列化错误: {}", e));
            }
            Self::Io(e) => return serializer.serialize_str(&format!("IO 错误: {}", e)),
        };
        serializer.serialize_str(msg)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
