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

#[derive(Debug, Clone, Serialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ipc: IpcError = self.into();
        ipc.serialize(serializer)
    }
}

impl From<&AppError> for IpcError {
    fn from(e: &AppError) -> Self {
        match e {
            AppError::NotFound(m) => IpcError {
                code: "NOT_FOUND".into(),
                message: m.clone(),
            },
            AppError::Validation(m) => IpcError {
                code: "VALIDATION".into(),
                message: m.clone(),
            },
            AppError::Ai(m) => IpcError {
                code: "AI_ERROR".into(),
                message: m.clone(),
            },
            AppError::Database(d) => IpcError {
                code: "DB_ERROR".into(),
                message: format!("数据库错误: {}", d),
            },
            AppError::Internal(_) => IpcError {
                code: "INTERNAL".into(),
                message: "服务内部错误".into(),
            },
            AppError::Serialization(_) => IpcError {
                code: "INTERNAL".into(),
                message: "数据序列化失败".into(),
            },
            AppError::Io(_) => IpcError {
                code: "INTERNAL".into(),
                message: "IO 错误".into(),
            },
        }
    }
}

impl From<AppError> for IpcError {
    fn from(e: AppError) -> Self {
        IpcError::from(&e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
