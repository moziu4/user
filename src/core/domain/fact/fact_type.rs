use chrono::{DateTime, Utc};
use perms::UserID;
use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{FactID, TenantID};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactType {
    // Identity
    Nie,
    Passport,
    Nationality,
    DateOfBirth,
    Gender,

    // Immigration
    ResidencePermit,
    ResidenceStatus,
    Tie,
    WorkPermit,
    VisaNumber,

    // Administrative
    MunicipalRegistration,
    SocialSecurityNumber,
    TaxIdentification,

    // Custom / Other
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for FactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactType::Nie => write!(f, "NIE"),
            FactType::Passport => write!(f, "PASSPORT"),
            FactType::Nationality => write!(f, "NATIONALITY"),
            FactType::DateOfBirth => write!(f, "DATE_OF_BIRTH"),
            FactType::Gender => write!(f, "GENDER"),
            FactType::ResidencePermit => write!(f, "RESIDENCE_PERMIT"),
            FactType::ResidenceStatus => write!(f, "RESIDENCE_STATUS"),
            FactType::Tie => write!(f, "TIE"),
            FactType::WorkPermit => write!(f, "WORK_PERMIT"),
            FactType::VisaNumber => write!(f, "VISA_NUMBER"),
            FactType::MunicipalRegistration => write!(f, "MUNICIPAL_REGISTRATION"),
            FactType::SocialSecurityNumber => write!(f, "SOCIAL_SECURITY_NUMBER"),
            FactType::TaxIdentification => write!(f, "TAX_IDENTIFICATION"),
            FactType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for FactType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let upper = s.to_uppercase();
        let ft = match upper.as_str() {
            "NIE" => FactType::Nie,
            "PASSPORT" => FactType::Passport,
            "NATIONALITY" => FactType::Nationality,
            "DATE_OF_BIRTH" | "DATEOFBIRTH" | "BIRTH_DATE" => FactType::DateOfBirth,
            "GENDER" => FactType::Gender,
            "RESIDENCE_PERMIT" | "RESIDENCEPERMIT" => FactType::ResidencePermit,
            "RESIDENCE_STATUS" | "RESIDENCESTATUS" => FactType::ResidenceStatus,
            "TIE" => FactType::Tie,
            "WORK_PERMIT" | "WORKPERMIT" => FactType::WorkPermit,
            "VISA_NUMBER" | "VISANUMBER" => FactType::VisaNumber,
            "MUNICIPAL_REGISTRATION" | "MUNICIPALREGISTRATION" => FactType::MunicipalRegistration,
            "SOCIAL_SECURITY_NUMBER" | "SOCIALSECURITYNUMBER" => FactType::SocialSecurityNumber,
            "TAX_IDENTIFICATION" | "TAXIDENTIFICATION" => FactType::TaxIdentification,
            _ => FactType::Other(s.to_string()),
        };
        Ok(ft)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactStatus {
    Active,
    Superseded,
    Conflicting,
    Revoked,
}

impl std::fmt::Display for FactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactStatus::Active => write!(f, "ACTIVE"),
            FactStatus::Superseded => write!(f, "SUPERSEDED"),
            FactStatus::Conflicting => write!(f, "CONFLICTING"),
            FactStatus::Revoked => write!(f, "REVOKED"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactVerification {
    Verified,
    Unverified,
    PendingVerification,
    Rejected,
}

impl std::fmt::Display for FactVerification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactVerification::Verified => write!(f, "VERIFIED"),
            FactVerification::Unverified => write!(f, "UNVERIFIED"),
            FactVerification::PendingVerification => write!(f, "PENDING_VERIFICATION"),
            FactVerification::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserFact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<FactID>,
    pub user_id: UserID,
    pub fact_type: FactType,
    pub value: serde_json::Value,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateFactDTO {
    pub user_id: UserID,
    pub fact_type: FactType,
    pub value: serde_json::Value,
    #[serde(default = "default_active_status")]
    pub status: FactStatus,
    #[serde(default = "default_unverified")]
    pub verification: FactVerification,
    pub source_service: String,
    pub source_tenant_id: Option<TenantID>,
    pub source_case_id: Option<String>,
    pub evidence_reference: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

fn default_active_status() -> FactStatus {
    FactStatus::Active
}

fn default_unverified() -> FactVerification {
    FactVerification::Unverified
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateFactStatusDTO {
    pub status: FactStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserFactSummaryDTO {
    pub id: Option<String>,
    pub fact_type: FactType,
    pub value: serde_json::Value,
    pub status: FactStatus,
    pub verification: FactVerification,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSummaryDTO {
    pub id: String,
    pub username: String,
    pub name: String,
    pub surname_1: Option<String>,
    pub surname_2: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserContactsDTO {
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserContextDTO {
    pub user: UserSummaryDTO,
    pub contacts: UserContactsDTO,
    pub facts: Vec<UserFactSummaryDTO>,
}
