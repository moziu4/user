use std::env;
use std::sync::Arc;

use mongodb::{Client, Collection};
use mongodb::bson::Document;
use crate::data::access::{
    auth_repo::MongoAuthRepo,
    perms_repo::MongoPermRepo,
    user_repo::MongoUserRepo,
};


#[derive(Clone)]
pub struct Context
{

    pub client:     Arc<Client>,
    pub user_repo:  Arc<MongoUserRepo>,
    pub auth_repo:  Arc<MongoAuthRepo>,
    pub perm_repo: Arc<MongoPermRepo>,

}


impl Context
{
    pub fn new(client: Client) -> Self
    {
        let arc_client = Arc::new(client);
        let db_name = env::var("MONGO_DATABASE").expect("Var MONGO_DATABASE no definida");
        
        let user_collection = arc_client.database(&db_name).collection("users");
        let auth_collection = arc_client.database(&db_name).collection("auth");
        let perm_collection = arc_client.database(&db_name).collection("perm");
        
        Self { client:     arc_client.clone(),
                  user_repo:  Arc::new(MongoUserRepo::new(user_collection)),
                  auth_repo:  Arc::new(MongoAuthRepo::new(auth_collection)),
                  perm_repo: Arc::new(MongoPermRepo::new(perm_collection)), }
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

    pub fn get_collection(&self, collection: &str) -> Collection<Document>
    {
        let db_name = env::var("MONGO_DATABASE").expect("Var MONGO_DATABASE no definida");
        self.client
            .database(&db_name)
            .collection(collection)
    }

}
