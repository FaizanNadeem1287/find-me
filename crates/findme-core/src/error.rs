//! Strongly typed error definitions for findme-core

use std::path::PathBuf;
use thiserror::Error;

/// Core engine error types
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Filesystem notification error: {0}")]
    Notify(String),

    #[error("Invalid search query syntax in '{query}': {reason}")]
    QuerySyntax { query: String, reason: String },

    #[error("Regex compilation error for '{pattern}': {source}")]
    Regex {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error("Index storage corruption or serialization failure: {0}")]
    Persistence(String),

    #[error("Path '{0}' not found or inaccessible")]
    PathNotFound(PathBuf),

    #[error("Engine channel closed unexpectedly: {0}")]
    ChannelClosed(String),
}

/// Specialized Result alias for findme-core operations
pub type Result<T> = std::result::Result<T, EngineError>;
