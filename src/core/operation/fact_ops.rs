use chrono::Utc;
use mongodb::bson::{doc, from_document, oid::ObjectId};
use perms::UserID;
use tracing::info;
use crate::context::Context;
use crate::core::domain::fact::{
    fact_commands::FactCommand,
    fact_error::{FactError, FactResult},
    fact_repo::FactRepo,
    fact_type::{
        CreateFactDTO, FactStatus, FactType, UpdateFactStatusDTO, UserContactsDTO,
        UserContextDTO, UserFact, UserFactSummaryDTO, UserSummaryDTO,
    },
};
use crate::core::domain::user::user_type::Phone;
use crate::utils::domains_ids::FactID;

pub struct FactOps<'a> {
    context: &'a Context,
}

impl<'a> FactOps<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self { context }
    }

    fn repo(&self) -> &dyn FactRepo {
        self.context.fact_repo.as_ref()
    }

    /// Process a CreateFactDTO with idempotency deduplication and event publication
    pub async fn create_fact(&self, dto: CreateFactDTO, event_id: Option<String>) -> FactResult<UserFact> {
        // 1. Idempotency check: if identical fact from same source service exists, reuse it
        if let Some(existing) = self
            .repo()
            .find_exact_match(&dto.user_id, &dto.fact_type, &dto.value, &dto.source_service)
            .await?
        {
            info!(
                "Idempotent duplicate fact detected for user {:?}, type {:?}, source {}. Returning existing fact.",
                dto.user_id, dto.fact_type, dto.source_service
            );
            return Ok(existing);
        }

        // 2. If status is Active, optionally supersede previous active facts of unique types (like NIE, Passport, etc.)
        if dto.status == FactStatus::Active {
            match dto.fact_type {
                FactType::Nie
                | FactType::Passport
                | FactType::Nationality
                | FactType::DateOfBirth
                | FactType::Gender
                | FactType::ResidencePermit
                | FactType::ResidenceStatus
                | FactType::Tie
                | FactType::WorkPermit
                | FactType::VisaNumber => {
                    let _ = self
                        .repo()
                        .supersede_active_facts(&dto.user_id, &dto.fact_type, None)
                        .await;
                }
                _ => {}
            }
        }

        let now = Utc::now();
        let fact = UserFact {
            _id: None,
            user_id: dto.user_id,
            fact_type: dto.fact_type,
            value: dto.value,
            status: dto.status,
            verification: dto.verification,
            source_service: dto.source_service,
            source_tenant_id: dto.source_tenant_id,
            source_case_id: dto.source_case_id,
            evidence_reference: dto.evidence_reference,
            valid_from: dto.valid_from,
            valid_until: dto.valid_until,
            created_at: now,
            updated_at: now,
        };

        let created = self.repo().create(fact).await?;

        // 3. Publish NATS event if NATS is available
        if let Some(nats) = &self.context.nats_service {
            nats.publish_fact_created(&created, event_id).await;
        }

        Ok(created)
    }

    pub async fn update_status(
        &self,
        fact_id: &FactID,
        dto: UpdateFactStatusDTO,
        event_id: Option<String>,
    ) -> FactResult<UserFact> {
        let updated = self.repo().update_status(fact_id, dto.status.clone()).await?;

        if let Some(nats) = &self.context.nats_service {
            if dto.status == FactStatus::Revoked {
                nats.publish_fact_revoked(&updated, event_id).await;
            } else {
                nats.publish_fact_updated(&updated, event_id).await;
            }
        }

        Ok(updated)
    }

    pub async fn revoke_fact(&self, fact_id: &FactID, event_id: Option<String>) -> FactResult<UserFact> {
        let updated = self.repo().update_status(fact_id, FactStatus::Revoked).await?;

        if let Some(nats) = &self.context.nats_service {
            nats.publish_fact_revoked(&updated, event_id).await;
        }

        Ok(updated)
    }

    pub async fn get_facts_by_user(
        &self,
        user_id: &UserID,
        status_filter: Option<&FactStatus>,
    ) -> FactResult<Vec<UserFact>> {
        self.repo().find_by_user_id(user_id, status_filter).await
    }

    pub async fn get_facts_by_type(
        &self,
        user_id: &UserID,
        fact_type: &FactType,
    ) -> FactResult<Vec<UserFact>> {
        self.repo().find_by_user_id_and_type(user_id, fact_type).await
    }

    pub async fn get_fact_by_id(&self, fact_id: &FactID) -> FactResult<UserFact> {
        self.repo().find_by_id(fact_id).await
    }

    pub async fn execute_command(&self, command: FactCommand, event_id: Option<String>) -> FactResult<UserFact> {
        match command {
            FactCommand::CreateFact {
                user_id,
                fact_type,
                value,
                status,
                verification,
                source_service,
                source_tenant_id,
                source_case_id,
                evidence_reference,
                valid_from,
                valid_until,
            } => {
                let dto = CreateFactDTO {
                    user_id,
                    fact_type,
                    value,
                    status,
                    verification,
                    source_service,
                    source_tenant_id,
                    source_case_id,
                    evidence_reference,
                    valid_from,
                    valid_until,
                };
                self.create_fact(dto, event_id).await
            }
            FactCommand::UpdateStatus { fact_id, status } => {
                self.update_status(&fact_id, UpdateFactStatusDTO { status }, event_id).await
            }
            FactCommand::RevokeFact { fact_id } => {
                self.revoke_fact(&fact_id, event_id).await
            }
        }
    }

    /// Retrieve minimal privilege context DTO for external consumers (e.g. ai-service)
    pub async fn get_user_context(&self, user_id: &UserID) -> FactResult<UserContextDTO> {
        let user = self
            .context
            .user_repo
            .fetch_by_id(user_id.clone())
            .await
            .map_err(|_| FactError::UserNotFound)?;

        // Try to fetch primary phone from phones collection
        let user_oid = ObjectId::from(user_id.clone());
        let phone_coll = self.context.get_collection("phones");
        let phone_filter = doc! { "user_id": user_oid, "status": "Active" };
        let phone_str = if let Ok(Some(phone_doc)) = phone_coll.find_one(phone_filter).await {
            if let Ok(phone) = from_document::<Phone>(phone_doc) {
                Some(phone.number)
            } else {
                None
            }
        } else {
            None
        };

        // Fetch active facts
        let active_facts = self
            .repo()
            .find_by_user_id(user_id, Some(&FactStatus::Active))
            .await?;

        let facts_summary = active_facts
            .into_iter()
            .map(|f| UserFactSummaryDTO {
                id: f._id.map(|id| id.to_string()),
                fact_type: f.fact_type,
                value: f.value,
                status: f.status,
                verification: f.verification,
                valid_until: f.valid_until,
            })
            .collect();

        Ok(UserContextDTO {
            user: UserSummaryDTO {
                id: user._id.map(|id| id.to_string()).unwrap_or_default(),
                username: user.username,
                name: user.name,
                surname_1: user.surname_1,
                surname_2: user.surname_2,
            },
            contacts: UserContactsDTO {
                email: user.email,
                phone: phone_str,
            },
            facts: facts_summary,
        })
    }
}
