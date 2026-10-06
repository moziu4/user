use std::env;
use tracing::{info, warn, error};
use crate::data::proxy::tenant::{Tenant, RegistrationType};
use crate::error::{ServiceError, ServiceResult};

pub struct TenantProxy;

impl TenantProxy {
    pub async fn get_tenant_by_id(client: &reqwest::Client, tenant_id: &str) -> ServiceResult<Tenant> {
        let base_url = env::var("TENANT_SERVICE_URL").unwrap_or_else(|_| "http://localhost:4002".to_string());
        let url = format!("{}/api/tenant/{}", base_url, tenant_id);
        
        info!("Consultando tenant al servicio Tenant: {}", url);
        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Error al conectar con el microservicio Tenant en {}: {:?}", url, e);
                return Err(ServiceError::InternalServerError);
            }
        };

        if response.status().is_success() {
            match response.json::<Tenant>().await {
                Ok(data) => Ok(data),
                Err(e) => {
                    error!("Error al deserializar respuesta de Tenant: {:?}", e);
                    Err(ServiceError::InternalServerError)
                }
            }
        } else {
            warn!("Respuesta no exitosa ({}) al consultar tenant {}", response.status(), tenant_id);
            Err(ServiceError::InternalServerError)
        }
    }

    pub async fn get_registration_type(client: &reqwest::Client, tenant_id: &str) -> RegistrationType {
        match Self::get_tenant_by_id(client, tenant_id).await {
            Ok(tenant) => tenant.registration_type,
            Err(_) => RegistrationType::default(),
        }
    }
}
