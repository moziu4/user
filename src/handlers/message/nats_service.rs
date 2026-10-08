use std::env;
use async_nats::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use futures_util::StreamExt;
use std::sync::Arc;
use crate::context::Context;
use crate::core::operation::membership_ops::MembershipOps;
use crate::core::operation::fact_ops::FactOps;
use crate::core::domain::membership::membership_commands::MembershipCommand;
use crate::core::domain::fact::{
    fact_commands::FactEventMessage,
    fact_type::{CreateFactDTO, UserFact},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMessage {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub event_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerificationEmailEvent {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvitationDeeplinkEvent {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deeplink_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}

pub struct NatsService {
    client: Client,
}

impl NatsService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn publish_verification_email(&self, event: &VerificationEmailEvent) {
        let subject = "user.verification.email_requested";
        match serde_json::to_vec(event) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject.to_string(), payload.into()).await {
                    error!("Error publishing verification email to NATS subject {}: {:?}", subject, e);
                } else {
                    info!("Verification email event published to NATS subject {}", subject);
                }
            }
            Err(e) => {
                error!("Error serializing verification email event for NATS: {:?}", e);
            }
        }
    }

    pub async fn publish_invitation_deeplink(&self, event: &InvitationDeeplinkEvent) {
        let subject = "user.invitation.deeplink_requested";
        match serde_json::to_vec(event) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject.to_string(), payload.into()).await {
                    error!("Error publishing invitation deeplink to NATS subject {}: {:?}", subject, e);
                } else {
                    info!("Invitation deeplink event published to NATS subject {}", subject);
                }
            }
            Err(e) => {
                error!("Error serializing invitation deeplink event for NATS: {:?}", e);
            }
        }
    }

    pub async fn publish_user_event(&self, user_id: String, username: String, email: String, event_type: &str) {
        let subject = format!("user.{}", event_type);
        let message = UserMessage {
            user_id,
            username,
            email,
            event_type: event_type.to_string(),
        };

        match serde_json::to_vec(&message) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject.clone(), payload.into()).await {
                    error!("Error publishing message to NATS subject {}: {:?}", subject, e);
                } else {
                    info!("Message published to NATS subject {}", subject);
                }
            }
            Err(e) => {
                error!("Error serializing message for NATS: {:?}", e);
            }
        }
    }

    pub async fn publish_membership_command(&self, command: &MembershipCommand) {
        let subject = "membership.command";
        match serde_json::to_vec(command) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject.to_string(), payload.into()).await {
                    error!("Error publishing command to NATS: {:?}", e);
                } else {
                    info!("Membership command published to NATS");
                }
            }
            Err(e) => {
                error!("Error serializing membership command: {:?}", e);
            }
        }
    }

    pub async fn publish_membership_command_with_id(&self, id: &str, command: &MembershipCommand) {
        let subject = format!("membership.command.{}", id);
        match serde_json::to_vec(command) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject, payload.into()).await {
                    error!("Error publishing command to NATS: {:?}", e);
                } else {
                    info!("Membership command published to NATS");
                }
            }
            Err(e) => {
                error!("Error serializing membership command: {:?}", e);
            }
        }
    }

    pub async fn publish_fact_created(&self, fact: &UserFact, event_id: Option<String>) {
        self.publish_fact_event("user.fact.created", fact, event_id).await;
    }

    pub async fn publish_fact_updated(&self, fact: &UserFact, event_id: Option<String>) {
        self.publish_fact_event("user.fact.updated", fact, event_id).await;
    }

    pub async fn publish_fact_revoked(&self, fact: &UserFact, event_id: Option<String>) {
        self.publish_fact_event("user.fact.revoked", fact, event_id).await;
    }

    async fn publish_fact_event(&self, subject: &str, fact: &UserFact, event_id: Option<String>) {
        let event_msg = FactEventMessage {
            event_id,
            event: subject.to_string(),
            user_id: fact.user_id.clone(),
            fact_type: fact.fact_type.clone(),
            value: fact.value.clone(),
            fact_id: fact._id.clone(),
            status: fact.status.clone(),
            verification: fact.verification.clone(),
            source_service: fact.source_service.clone(),
            source_tenant_id: fact.source_tenant_id.clone(),
            source_case_id: fact.source_case_id.clone(),
            evidence_reference: fact.evidence_reference.clone(),
            valid_from: fact.valid_from,
            valid_until: fact.valid_until,
            timestamp: Some(chrono::Utc::now()),
        };

        match serde_json::to_vec(&event_msg) {
            Ok(payload) => {
                if let Err(e) = self.client.publish(subject.to_string(), payload.into()).await {
                    error!("Error publishing fact event to NATS subject {}: {:?}", subject, e);
                } else {
                    info!("Fact event published to NATS subject {}", subject);
                }
            }
            Err(e) => {
                error!("Error serializing fact event for NATS: {:?}", e);
            }
        }
    }

    pub async fn subscribe_fact_events(client: Client, context: Arc<Context>) {
        let subject = "user.fact.>";
        match client.subscribe(subject).await {
            Ok(mut subscriber) => {
                info!("Subscribed to NATS subject: {}", subject);
                while let Some(msg) = subscriber.next().await {
                    info!("Received NATS fact event on subject: {}", msg.subject);
                    let ops = FactOps::new(&context);

                    // Try deserializing as FactEventMessage first
                    if let Ok(event) = serde_json::from_slice::<FactEventMessage>(&msg.payload) {
                        info!("Parsed FactEventMessage: {:?}", event);
                        let event_id = event.event_id.clone();
                        let dto = CreateFactDTO {
                            user_id: event.user_id,
                            fact_type: event.fact_type,
                            value: event.value,
                            status: event.status,
                            verification: event.verification,
                            source_service: event.source_service,
                            source_tenant_id: event.source_tenant_id,
                            source_case_id: event.source_case_id,
                            evidence_reference: event.evidence_reference,
                            valid_from: event.valid_from,
                            valid_until: event.valid_until,
                        };

                        if msg.subject.ends_with(".created") || event.event == "user.fact.created" {
                            let _ = ops.create_fact(dto, event_id).await;
                        } else if msg.subject.ends_with(".revoked") || event.event == "user.fact.revoked" {
                            if let Some(fid) = event.fact_id {
                                let _ = ops.revoke_fact(&fid, event_id).await;
                            }
                        } else {
                            let _ = ops.create_fact(dto, event_id).await;
                        }
                    } else if let Ok(dto) = serde_json::from_slice::<CreateFactDTO>(&msg.payload) {
                        info!("Parsed CreateFactDTO from NATS: {:?}", dto);
                        let _ = ops.create_fact(dto, None).await;
                    } else {
                        warn!("Could not deserialize NATS fact message payload on {}", msg.subject);
                    }
                }
            }
            Err(e) => {
                error!("Failed to subscribe to NATS subject {}: {:?}", subject, e);
            }
        }
    }

    pub async fn subscribe_membership_events(client: Client, context: Arc<Context>) {
        let subject = "membership.>";
        match client.subscribe(subject).await {
            Ok(mut subscriber) => {
                info!("Subscribed to NATS subject: {}", subject);
                while let Some(msg) = subscriber.next().await {
                    if let Ok(command) = serde_json::from_slice::<MembershipCommand>(&msg.payload) {
                        info!("Received NATS membership command: {:?}", command);
                        let membership_repo = context.get_collection("memberships");
                        let mongo_repo = crate::data::access::membership_repo::MongoMembershipRepo::new(membership_repo);
                        let ops = MembershipOps::new(&mongo_repo, &context);

                        let subject_parts: Vec<&str> = msg.subject.split('.').collect();
                        let membership_id = if subject_parts.len() > 2 {
                            Some(subject_parts[2])
                        } else {
                            None
                        };

                        let _ = ops.execute_command(membership_id, command).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to subscribe to NATS subject {}: {:?}", subject, e);
            }
        }
    }
}

pub async fn connect_nats() -> Option<Client> {
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    info!("Intentando conectar a NATS en {}", nats_url);
    
    match async_nats::connect(&nats_url).await {
        Ok(client) => {
            info!("Conectado con éxito a NATS en {}", nats_url);
            Some(client)
        }
        Err(e) => {
            error!("Fallo al conectar a NATS en {}: {:?}. Asegúrate de que NATS_URL esté configurado correctamente.", nats_url, e);
            None
        }
    }
}
