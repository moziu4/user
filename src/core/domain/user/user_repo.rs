use async_trait::async_trait;


use crate::core::domain::membership::Membership;
use crate::core::domain::user::user_type::{Phone, Address};
use crate::utils::domains_ids::{TenantID, AgencyID};
use crate::core::domain::user::user_error::UserError;
use crate::core::domain::user::User;
use perms::UserID;

#[async_trait]
pub trait UserRepo
{
    async fn create(&self, new_user: User) -> Result<User, UserError>;
    async fn fetch_all(&self) -> Result<Vec<User>, UserError>;
    async fn fetch_all_actives(&self) -> Result<Vec<User>, UserError>;
    async fn fetch_by_id(&self, id: UserID) -> Result<User, UserError>;
    async fn fetch_by_email(&self, email: String) -> Result<User, UserError>;
    async fn save(&self, user: User) -> Result<User, UserError>;
    async fn delete(&self, id: UserID) -> Result<(), UserError>;

    async fn create_membership(&self, membership: Membership) -> Result<Membership, UserError>;
    async fn fetch_memberships_by_user(&self, user_id: UserID) -> Result<Vec<Membership>, UserError>;
    
    async fn create_phone(&self, phone: Phone) -> Result<Phone, UserError>;
    async fn create_address(&self, address: Address) -> Result<Address, UserError>;

    async fn deactivate_tenant_membership(&self, user_id: UserID, tenant_id: TenantID) -> Result<(), UserError>;
    async fn deactivate_agency_membership(&self, user_id: UserID, agency_id: AgencyID) -> Result<(), UserError>;
    async fn anonymize_user_in_tenant(&self, user_id: UserID, tenant_id: TenantID) -> Result<(), UserError>;
}
