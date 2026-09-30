use thiserror::Error;

pub type MembershipResult<T> = Result<T, MembershipError>;

#[derive(Error, Debug)]
pub enum MembershipError {
    #[error("Membership not found")]
    MembershipNotFound,
    #[error("Membership already exists")]
    MembershipAlreadyExists,
    #[error("Invalid membership")]
    InvalidMembership,
    #[error("Membership operation failed")]
    MembershipOperationFailed,
}