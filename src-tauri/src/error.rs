use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("PM3 not found on any port")]
    DeviceNotFound,
    #[error("PM3 command failed: {0}")]
    CommandFailed(String),
    #[error("No card detected")]
    NoCardFound,
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}
