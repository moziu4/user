
use thiserror::Error;
pub type Auth<T> = Result<T, AuthError>;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Auth already exists")]
    AlreadyExists,

    #[error("Auth Username already exists")]
    AlreadyUsernameExists,

    #[error("Auth Email already exists")]
    AlreadyEmailExists,

    #[error("Auth isn't authorized")]
    Unauthorized,

    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Mongo error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    #[error("Auth not found")]
    AuthNotFound,
    
    #[error("Incorrect Format Email")]
    IncorrectFormatEmail,

    #[error("Email is used")]
    EmailIsUsed,

    #[error("Auth doc isn't updated")]
    AuthDocNotUpdated,

    #[error("Auth doc isn't created")]
    AuthDocumentNotCreated,
    
    #[error("Fail to create a token")]
    FailToCreateToken,
    
    #[error("Fail to decode token")]
    FailToDecodeToken,

    #[error("Incorrect Password")]
    IncorrectPassword
}

