use std::env;
use async_nats::Client;
use serde::Serialize;
use tracing::{info, error};

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
}

pub async fn connect_nats() -> Option<Client> {
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    // Si estamos en un contenedor de Docker, el host por defecto suele ser 'nats_service'
    // segun el docker-compose global.
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
