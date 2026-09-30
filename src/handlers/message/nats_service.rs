use std::env;
use async_nats::Client;
use serde::Serialize;
use tracing::{info, error};
use futures_util::StreamExt;
use std::sync::Arc;
use crate::context::Context;
use crate::core::operation::membership_ops::MembershipOps;
use crate::core::domain::membership::membership_commands::MembershipCommand;

#[derive(Serialize)]
pub struct UserMessage {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub event_type: String,
}

pub struct NatsService {
    client: Client,
}

impl NatsService {
    pub fn new(client: Client) -> Self {
        Self { client }
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
