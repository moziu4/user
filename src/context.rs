use std::env;
use std::sync::Arc;

use mongodb::{Client, Collection};
use mongodb::bson::Document;
use crate::data::access::{
    auth_repo::MongoAuthRepo,
    fact_repo::MongoFactRepo,
    perms_repo::MongoPermRepo,
    user_repo::MongoUserRepo,
};
use crate::handlers::message::nats_service::NatsService;

#[derive(Clone)]
pub struct Context
{

    pub client:     Arc<Client>,
    pub user_repo:  Arc<MongoUserRepo>,
    pub auth_repo:  Arc<MongoAuthRepo>,
    pub perm_repo: Arc<MongoPermRepo>,
    pub fact_repo: Arc<MongoFactRepo>,
    pub nats_service: Option<Arc<NatsService>>,
    pub http_client: reqwest::Client,
}


impl Context
{
    pub fn new(client: Client, nats_client: Option<async_nats::Client>) -> Self
    {
        let arc_client = Arc::new(client);
        let db_name = env::var("MONGO_DATABASE").expect("Var MONGO_DATABASE no definida");
        
        let user_collection = arc_client.database(&db_name).collection("users");
        let membership_collection = arc_client.database(&db_name).collection("memberships");
        let phone_collection = arc_client.database(&db_name).collection("phones");
        let address_collection = arc_client.database(&db_name).collection("addresses");
        let auth_collection = arc_client.database(&db_name).collection("auth");
        let perm_collection = arc_client.database(&db_name).collection("perm");
        let fact_collection = arc_client.database(&db_name).collection("facts");

        let nats_service = nats_client.map(|c| Arc::new(NatsService::new(c)));
        let http_client = reqwest::Client::new();
        
        Self { client:     arc_client.clone(),
                  user_repo:  Arc::new(MongoUserRepo::new(
                      user_collection,
                      membership_collection,
                      phone_collection,
                      address_collection
                  )),
                  auth_repo:  Arc::new(MongoAuthRepo::new(auth_collection)),
                  perm_repo: Arc::new(MongoPermRepo::new(perm_collection)),
                  fact_repo: Arc::new(MongoFactRepo::new(fact_collection)),
                  nats_service,
                  http_client,
        }
    }

    pub async fn get_tenant(&self, tenant_id: &str) -> crate::error::ServiceResult<crate::data::proxy::tenant::Tenant> {
        crate::data::proxy::tenant_proxy::TenantProxy::get_tenant_by_id(&self.http_client, tenant_id).await
    }

    pub fn get_user_repo(&self) -> Arc<MongoUserRepo>
    {
        Arc::clone(&self.user_repo)
    }

    pub fn get_auth_repo(&self) -> Arc<MongoAuthRepo>
    {
        Arc::clone(&self.auth_repo)
    }

    pub fn get_perm_repo(&self) -> Arc<MongoPermRepo>
    {
        Arc::clone(&self.perm_repo)
    }

    pub fn get_fact_repo(&self) -> Arc<MongoFactRepo>
    {
        Arc::clone(&self.fact_repo)
    }

    pub fn get_collection(&self, collection: &str) -> Collection<Document>
    {
        let db_name = env::var("MONGO_DATABASE").expect("Var MONGO_DATABASE no definida");
        self.client
            .database(&db_name)
            .collection(collection)
    }

}
