pub mod membership_error;
pub mod membership_repo;
pub mod membership_type;
pub mod membership_commands;

use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{MembershipID, OrganizationID, TenantID};
use perms::UserID;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MembershipStatus {
    Active,
    Inactive,
    Pending,
    Unverified,
    Suspended,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MembershipTarget {
    Organization(OrganizationID),
    Tenant(TenantID),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Membership {
    pub _id:       Option<MembershipID>,
    pub user_id:   UserID,
    pub target:    MembershipTarget,
    pub role_id:   u32,
    pub status:    MembershipStatus,
}

impl Membership {
    pub fn new_tenant(user_id: UserID, tenant_id: TenantID, role_id: u32) -> Self {
        Self::new_tenant_with_status(user_id, tenant_id, role_id, MembershipStatus::Active)
    }

    pub fn new_tenant_with_status(user_id: UserID, tenant_id: TenantID, role_id: u32, status: MembershipStatus) -> Self {
        Self {
            _id: None,
            user_id,
            target: MembershipTarget::Tenant(tenant_id),
            role_id,
            status,
        }
    }

    pub fn new_organization(user_id: UserID, organization_id: OrganizationID, role_id: u32) -> Self {
        Self::new_organization_with_status(user_id, organization_id, role_id, MembershipStatus::Active)
    }

    pub fn new_organization_with_status(user_id: UserID, organization_id: OrganizationID, role_id: u32, status: MembershipStatus) -> Self {
        Self {
            _id: None,
            user_id,
            target: MembershipTarget::Organization(organization_id),
            role_id,
            status,
        }
    }
}
