use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{MembershipID, AgencyID, TenantID};
use perms::UserID;
use crate::core::domain::auth::auth_type::Role;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MembershipStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgencyMembership {
    pub agency_id: AgencyID,
    pub role_id: u32,
    pub status: MembershipStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenantMembership {
    pub tenant_id: TenantID,
    pub role_id: u32,
    pub status: MembershipStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Membership {
    pub _id:       Option<MembershipID>,
    pub user_id:   UserID,
    pub agencies:  Vec<AgencyMembership>,
    pub tenants:   Vec<TenantMembership>,
}

impl Membership {
    pub fn new(user_id: UserID) -> Self {
        Self {
            _id:       None,
            user_id,
            agencies:  Vec::new(),
            tenants:   Vec::new(),
        }
    }

    pub fn add_agency(&mut self, agency_id: AgencyID, role_id: u32) {
        self.agencies.push(AgencyMembership { agency_id, role_id, status: MembershipStatus::Active });
    }

    pub fn add_tenant(&mut self, tenant_id: TenantID, role_id: u32) {
        self.tenants.push(TenantMembership { tenant_id, role_id, status: MembershipStatus::Active });
    }

    pub fn with_agency(mut self, agency_id: AgencyID, role_id: u32) -> Self {
        self.add_agency(agency_id, role_id);
        self
    }

    pub fn with_tenant(mut self, tenant_id: TenantID, role_id: u32) -> Self {
        self.add_tenant(tenant_id, role_id);
        self
    }
}
