use chrono::{DateTime, Utc};
use perms::UserID;
use serde::{Deserialize, Serialize};
use crate::core::domain::fact::fact_type::{FactStatus, FactType, FactVerification};
use crate::utils::domains_ids::{FactID, TenantID};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum FactCommand {
    CreateFact {
        user_id: UserID,
        fact_type: FactType,
        value: serde_json::Value,
        #[serde(default = "default_active_status")]
        status: FactStatus,
        #[serde(default = "default_unverified")]
        verification: FactVerification,
        source_service: String,
        source_tenant_id: Option<TenantID>,
        source_case_id: Option<String>,
        evidence_reference: Option<String>,
        valid_from: Option<DateTime<Utc>>,
        valid_until: Option<DateTime<Utc>>,
    },
    UpdateStatus {
        fact_id: FactID,
        status: FactStatus,
    },
    RevokeFact {
        fact_id: FactID,
    },
}

fn default_active_status() -> FactStatus {
    FactStatus::Active
}

fn default_unverified() -> FactVerification {
    FactVerification::Unverified
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FactEventMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub event: String, // "user.fact.created", "user.fact.updated", "user.fact.revoked"
    pub user_id: UserID,
    pub fact_type: FactType,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_id: Option<FactID>,
    pub status: FactStatus,
    pub verification: FactVerification,
    pub source_service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tenant_id: Option<TenantID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_case_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}
