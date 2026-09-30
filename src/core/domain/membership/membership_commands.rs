use serde::{Deserialize, Serialize};
use crate::core::domain::membership::MembershipStatus;
use crate::utils::domains_ids::{OrganizationID, TenantID};
use perms::UserID;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum MembershipCommand {
    CreateTenant {
        user_id: UserID,
        tenant_id: TenantID,
        role_id: u32,
    },
    CreateOrganization {
        user_id: UserID,
        organization_id: OrganizationID,
        role_id: u32,
    },
    UpdateStatus {
        status: MembershipStatus,
    },
    DeactivateUserFromOrganization {
        user_id: UserID,
        organization_id: OrganizationID,
    },
}
