use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{OrganizationID, TenantID};
use crate::core::domain::membership::MembershipStatus;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateMembershipStatusDTO {
    pub status: MembershipStatus,
}
