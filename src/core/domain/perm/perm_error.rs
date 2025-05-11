use thiserror::Error;

pub type Perm<T> = Result<T, PermError>;

#[derive(Error, Debug)]
pub enum PermError
{
    #[error("Permission not found")]
    PermNotFound,

    #[error("Perms Relationship not found")]
    PermRelationShipNotFound,

    #[error("Perms Relationship not created")]
    PermRelationShipNotCreated, 
    
    #[error("Perm already exists")]
    PermAlreadyExist,
    
    #[error("New permission isn't created in a document")]
    PermIsNotCreatedDocument,

    #[error("Id not found")]
    IdNotFound,

    #[error("Mongo error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    #[error("Database query error")]
    DatabaseQueryError,

    #[error("Perm document parse error")]
    PermDocParseError,

    #[error("Permission Isn't Deleted")]
    PermNotDeleted,

    #[error("Permission Doc Not Updated")]
    PermDocNotUpdated,

    #[error("Permission Document Isn't Created")]
    PermDocumentNotCreated
}