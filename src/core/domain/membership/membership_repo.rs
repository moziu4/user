use async_trait::async_trait;
use crate::core::domain::membership::{Membership, membership_error::MembershipResult, MembershipStatus};
use crate::context::Context;
use crate::utils::domains_ids::{OrganizationID, TenantID};
use perms::UserID;

#[async_trait]
pub trait MembershipRepo {
    async fn create(&self, membership: Membership, context: &Context) -> MembershipResult<Membership>;
    async fn get_membership(&self, context: &Context, membership_id: &str) -> MembershipResult<Membership>;
    async fn get_membership_for_user(&self, context: &Context, user_id: &str) -> MembershipResult<Vec<Membership>>;
    async fn update_status(&self, context: &Context, membership_id: &str, status: &str) -> MembershipResult<()>;
    async fn deactivate_organization_and_tenants(&self, context: &Context, user_id: UserID, organization_id: OrganizationID) -> MembershipResult<()>;
}
