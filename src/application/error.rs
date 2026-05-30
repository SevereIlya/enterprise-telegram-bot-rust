use crate::domain::error::DomainError;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn message_error(&self) -> &'static str {
        match self {
            AppError::Database(_) => "Some message",
            AppError::Config(_) => "Some message",
            AppError::Domain(domain_err) => domain_err.message_error(),
            AppError::MaxRetriesExceeded(_) => "Some message",
            AppError::Internal(_) => "Some message",
        }
    }
}
