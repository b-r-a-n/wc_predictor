//! CLI error types.

use std::path::PathBuf;

/// CLI errors.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Team not found: {0}")]
    TeamNotFound(String),

    #[error("Invalid data file: {0}")]
    InvalidDataFile(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tournament validation failed: {0}")]
    TournamentValidation(#[from] wc_core::tournament::TournamentError),
}

pub type Result<T> = std::result::Result<T, CliError>;
