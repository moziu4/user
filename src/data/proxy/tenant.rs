use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{TenantID, OrganizationID, PlanID};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationType {
    SelfService,
    #[serde(alias = "deeplinking", alias = "deplinking", alias = "invitation")]
    Deplinking,
}

impl Default for RegistrationType {
    fn default() -> Self {
        Self::SelfService
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TenantState {
    Active,
    Inactive,
    Suspended,
}

impl Default for TenantState {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TenantConfiguration {
    #[serde(default)]
    pub settings: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TenantFeatures {
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Menu {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tenant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<TenantID>,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub organization_id: Option<OrganizationID>,
    #[serde(default, alias = "plan", skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<PlanID>,
    #[serde(default)]
    pub configuration: TenantConfiguration,
    #[serde(default)]
    pub state: TenantState,
    #[serde(default)]
    pub registration_type: RegistrationType,
    #[serde(default)]
    pub features: TenantFeatures,
    #[serde(default)]
    pub default_language: String,
    #[serde(default)]
    pub available_languages: Vec<String>,
    #[serde(default)]
    pub menus: Vec<Menu>,
}
