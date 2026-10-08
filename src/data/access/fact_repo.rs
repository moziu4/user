use async_trait::async_trait;
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, from_document, to_bson, to_document, Document},
    Collection,
};
use mongodb::bson::oid::ObjectId;
use perms::UserID;
use crate::core::domain::fact::{
    fact_error::{FactError, FactResult},
    fact_repo::FactRepo,
    fact_type::{FactStatus, FactType, UserFact},
};
use crate::utils::domains_ids::FactID;

#[derive(Clone, Debug)]
pub struct MongoFactRepo {
    collection: Collection<Document>,
}

impl MongoFactRepo {
    pub fn new(collection: Collection<Document>) -> Self {
        Self { collection }
    }
}

#[async_trait]
impl FactRepo for MongoFactRepo {
    async fn create(&self, mut fact: UserFact) -> FactResult<UserFact> {
        if fact._id.is_none() {
            fact._id = Some(FactID::new());
        }
        let now = Utc::now();
        fact.created_at = now;
        fact.updated_at = now;

        let doc = to_document(&fact)
            .map_err(|e| FactError::FactOperationFailed(format!("Serialization error: {}", e)))?;
        
        self.collection
            .insert_one(doc)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo insert error: {}", e)))?;

        Ok(fact)
    }

    async fn find_by_id(&self, fact_id: &FactID) -> FactResult<UserFact> {
        let obj_id: ObjectId = (*fact_id).clone().into();
        let filter = doc! { "_id": obj_id };

        let doc = self.collection
            .find_one(filter)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo find error: {}", e)))?
            .ok_or(FactError::FactNotFound)?;

        let fact: UserFact = from_document(doc)
            .map_err(|e| FactError::InvalidFact(format!("Deserialization error: {}", e)))?;
        Ok(fact)
    }

    async fn find_by_user_id(&self, user_id: &UserID, status_filter: Option<&FactStatus>) -> FactResult<Vec<UserFact>> {
        let user_oid = ObjectId::from(user_id.clone());
        let mut filter = doc! { "user_id": user_oid };

        if let Some(status) = status_filter {
            let status_bson = to_bson(status)
                .map_err(|e| FactError::FactOperationFailed(format!("Status bson error: {}", e)))?;
            filter.insert("status", status_bson);
        }

        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo find error: {}", e)))?;

        let mut facts = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Cursor error: {}", e)))?
        {
            if let Ok(fact) = from_document::<UserFact>(doc) {
                facts.push(fact);
            }
        }

        Ok(facts)
    }

    async fn find_by_user_id_and_type(&self, user_id: &UserID, fact_type: &FactType) -> FactResult<Vec<UserFact>> {
        let user_oid = ObjectId::from(user_id.clone());
        let fact_type_bson = to_bson(fact_type)
            .map_err(|e| FactError::FactOperationFailed(format!("FactType bson error: {}", e)))?;
        
        let filter = doc! {
            "user_id": user_oid,
            "fact_type": fact_type_bson
        };

        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo find error: {}", e)))?;

        let mut facts = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Cursor error: {}", e)))?
        {
            if let Ok(fact) = from_document::<UserFact>(doc) {
                facts.push(fact);
            }
        }

        Ok(facts)
    }

    async fn find_exact_match(
        &self,
        user_id: &UserID,
        fact_type: &FactType,
        value: &serde_json::Value,
        source_service: &str,
    ) -> FactResult<Option<UserFact>> {
        let user_oid = ObjectId::from(user_id.clone());
        let fact_type_bson = to_bson(fact_type)
            .map_err(|e| FactError::FactOperationFailed(format!("FactType bson error: {}", e)))?;
        let value_bson = to_bson(value)
            .map_err(|e| FactError::FactOperationFailed(format!("Value bson error: {}", e)))?;

        let filter = doc! {
            "user_id": user_oid,
            "fact_type": fact_type_bson,
            "value": value_bson,
            "source_service": source_service,
        };

        let doc = self.collection
            .find_one(filter)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo find error: {}", e)))?;

        match doc {
            Some(d) => {
                let fact = from_document::<UserFact>(d)
                    .map_err(|e| FactError::InvalidFact(format!("Deserialization error: {}", e)))?;
                Ok(Some(fact))
            }
            None => Ok(None),
        }
    }

    async fn update_status(&self, fact_id: &FactID, status: FactStatus) -> FactResult<UserFact> {
        let obj_id: ObjectId = (*fact_id).clone().into();
        let filter = doc! { "_id": obj_id };
        let status_bson = to_bson(&status)
            .map_err(|e| FactError::FactOperationFailed(format!("Status bson error: {}", e)))?;
        let now_bson = to_bson(&Utc::now())
            .map_err(|e| FactError::FactOperationFailed(format!("Time bson error: {}", e)))?;

        let update = doc! {
            "$set": {
                "status": status_bson,
                "updated_at": now_bson
            }
        };

        let result = self.collection
            .update_one(filter.clone(), update)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo update error: {}", e)))?;

        if result.matched_count == 0 {
            return Err(FactError::FactNotFound);
        }

        self.find_by_id(fact_id).await
    }

    async fn supersede_active_facts(
        &self,
        user_id: &UserID,
        fact_type: &FactType,
        except_id: Option<&FactID>,
    ) -> FactResult<usize> {
        let user_oid = ObjectId::from(user_id.clone());
        let fact_type_bson = to_bson(fact_type)
            .map_err(|e| FactError::FactOperationFailed(format!("FactType bson error: {}", e)))?;
        let active_status_bson = to_bson(&FactStatus::Active)
            .map_err(|e| FactError::FactOperationFailed(format!("Status bson error: {}", e)))?;
        let superseded_status_bson = to_bson(&FactStatus::Superseded)
            .map_err(|e| FactError::FactOperationFailed(format!("Status bson error: {}", e)))?;
        let now_bson = to_bson(&Utc::now())
            .map_err(|e| FactError::FactOperationFailed(format!("Time bson error: {}", e)))?;

        let mut filter = doc! {
            "user_id": user_oid,
            "fact_type": fact_type_bson,
            "status": active_status_bson,
        };

        if let Some(eid) = except_id {
            let obj_id: ObjectId = (*eid).clone().into();
            filter.insert("_id", doc! { "$ne": obj_id });
        }

        let update = doc! {
            "$set": {
                "status": superseded_status_bson,
                "updated_at": now_bson
            }
        };

        let result = self.collection
            .update_many(filter, update)
            .await
            .map_err(|e| FactError::FactOperationFailed(format!("Mongo update many error: {}", e)))?;

        Ok(result.modified_count as usize)
    }
}
