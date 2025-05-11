use thiserror::Error;
pub type UserResult<T> = Result<T, UserError>;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    
    #[error("User already exists")]
    AlreadyExists,
    
    #[error("User not authorized")]
    Unauthorized,
    
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Mongo error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Incorrect Format Email")]
    IncorrectFormatEmail,
    
    #[error("Email is used")]   
    EmailIsUsed,

    #[error("User doc not updated")]
    UserDocNotUpdated,

    #[error("User doc isn't created")]   
    UserDocumentNotCreated,

    #[error("Invalid user id")]  
    InvalidUserId,

    #[error("Hash password error")]
    HashPasswordError,
    
    #[error("Perm error")]
    PermError,
    
    #[error("Auth error")]
    AuthError

}