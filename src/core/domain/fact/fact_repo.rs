use async_trait::async_trait;
use perms::UserID;
use crate::core::domain::fact::{
    fact_error::FactResult,
    fact_type::{FactStatus, FactType, UserFact},
};
use crate::utils::domains_ids::FactID;

#[async_trait]
pub trait FactRepo: Send + Sync {
    async fn create(&self, fact: UserFact) -> FactResult<UserFact>;
    async fn find_by_id(&self, fact_id: &FactID) -> FactResult<UserFact>;
    async fn find_by_user_id(&self, user_id: &UserID, status_filter: Option<&FactStatus>) -> FactResult<Vec<UserFact>>;
    async fn find_by_user_id_and_type(&self, user_id: &UserID, fact_type: &FactType) -> FactResult<Vec<UserFact>>;
    async fn find_exact_match(
        &self,
        user_id: &UserID,
        fact_type: &FactType,
        value: &serde_json::Value,
        source_service: &str,
    ) -> FactResult<Option<UserFact>>;
    async fn update_status(&self, fact_id: &FactID, status: FactStatus) -> FactResult<UserFact>;
    async fn supersede_active_facts(
        &self,
        user_id: &UserID,
        fact_type: &FactType,
        except_id: Option<&FactID>,
    ) -> FactResult<usize>;
}
