use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct SerializableError {
    pub kind: String,
    pub message: String,
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Database(_) => "database",
            Self::Network(_) => "network",
            Self::Json(_) => "json",
            Self::Io(_) => "io",
            Self::InvalidInput(_) => "invalid_input",
            Self::NotFound(_) => "not_found",
            Self::Internal(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        SerializableError {
            kind: self.kind().to_string(),
            message: self.to_string(),
        }
        .serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_serializes_to_kind_and_message() {
        let err = AppError::InvalidInput("radius must be > 0".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"invalid_input\""));
        assert!(json.contains("radius must be > 0"));
    }
}
