use thiserror::Error;
use tokio::task::JoinError;

/// Application error types
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("GDB error: {0}")]
    GDBError(String),

    #[error("GDB timeout")]
    GDBTimeout,

    #[error("GDB busy")]
    GDBBusy,

    #[error("GDB quit")]
    GDBQuit,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Program has exited. {0}")]
    ProgramExited(String),

    #[error("Program is not running. {0}")]
    ProgramNotRunning(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Parse Json error: {0}")]
    ParseJsonError(#[from] serde_json::error::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Task join error: {0}")]
    JoinError(#[from] JoinError),
}

/// Application result type
pub type AppResult<T> = Result<T, AppError>;
