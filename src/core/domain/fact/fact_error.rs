use thiserror::Error;

pub type FactResult<T> = Result<T, FactError>;

#[derive(Error, Debug)]
pub enum FactError {
    #[error("Fact not found")]
    FactNotFound,
    #[error("Fact already exists and is identical (idempotent duplicate)")]
    FactAlreadyExists,
    #[error("Invalid fact: {0}")]
    InvalidFact(String),
    #[error("Fact operation failed: {0}")]
    FactOperationFailed(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Fact conflict detected")]
    FactConflict,
}
