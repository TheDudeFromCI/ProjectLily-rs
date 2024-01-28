use rust_bert::RustBertError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum MemoryDBError {
    #[error("Failed to create model: {0}")]
    FailedToCreateModel(#[from] RustBertError),
    #[error("Wrong embedding size: expected: {expected}, actual: {actual}")]
    WrongEmbeddingSize { expected: usize, actual: usize },
    #[error("Failed to search KD Tree: {0}")]
    KdTreeError(#[from] kdtree::ErrorKind),
    #[error("Failed to spawn blocking task: {0}")]
    AsyncError(#[from] JoinError),
    #[error("Database URL not set in environment variables")]
    DatabaseUrlNotSet,
    #[error("Sqlite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("Unexpected database entry. Expected: `{expected}`, Actual: `{actual}`")]
    UnexpectedDatabaseEntry { expected: String, actual: String },
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
