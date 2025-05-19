
use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::{ bson::{doc, oid::ObjectId, Document}, Collection};
use mongodb::bson::{from_document, to_document};
use crate::core::domain::{
    auth::{
        auth_repo::AuthRepo,
    },
};
use crate::core::domain::auth::Auth;
use crate::core::domain::auth::auth_error::AuthError;
use crate::utils::domains_ids::AuthID;

#[derive(Clone)]
pub struct MongoAuthRepo {
    collection: Collection<Document>,
   
}

impl MongoAuthRepo {
    pub fn new(collection: Collection<Document>) -> Self {
       Self { collection }
    }

    pub async fn create (&self, mut new_auth: Auth) -> Result<Auth, AuthError>
    {
        if new_auth._id.is_none()
        {
            new_auth._id = Some(AuthID::new());
        }
        let collection = &self.collection;
        let auth_doc = to_document(&new_auth).map_err(|_| AuthError::AuthDocumentNotCreated)?;
        let insert_result = collection.insert_one(auth_doc).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id()
        {
            Ok(Auth { _id:               Some(AuthID::from_object_id(inserted_id)),
                user_id: new_auth.user_id,
                username: new_auth.username,
                email: new_auth.email,
                password: new_auth.password,
                roles: new_auth.roles,
                permissions: new_auth.permissions,
            })
        }
        else
        {
            Err(AuthError::AuthNotFound)
        }
    }
    pub async fn fetch_all(&self) -> Result<Vec<Auth>, AuthError>
    {
        let filter = doc! {};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| AuthError::AuthNotFound)?;

        let mut auths: Vec<Auth> = Vec::new();
        while let Some(auth_doc) = cursor.try_next()
            .await
            .map_err(|_| AuthError::AuthNotFound)?
        {
            let auth: Auth = from_document(auth_doc).map_err(|_| AuthError::AuthNotFound)?;
            auths.push(auth);
        }
        Ok(auths)
    }

    pub async fn fetch_all_actives(&self) -> Result<Vec<Auth>, AuthError>
    {
        let filter = doc! {"status": "Active"};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| AuthError::AuthNotFound)?;

        let mut auths: Vec<Auth> = Vec::new();
        while let Some(auth_doc) = cursor.try_next()
            .await
            .map_err(|_| AuthError::AuthNotFound)?
        {
            let auth: Auth = from_document(auth_doc).map_err(|_| AuthError::AuthNotFound)?;
            auths.push(auth);
        }
        Ok(auths)
    }

    pub async fn fetch_by_id(&self, id: AuthID) -> Result<Auth, AuthError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id)};
        let auth_doc = collection.find_one(filter)
            .await
            .map_err(|_| AuthError::AuthNotFound)?
            .ok_or(AuthError::AuthNotFound)?;

        let auth: Auth = from_document(auth_doc).map_err(|_| AuthError::AuthNotFound)?;
        Ok(auth)
    }

    pub async fn save (&self, auth: Auth) -> Result<Auth, AuthError>
    {
        let collection = &self.collection;
        if let Some(auth_id) = &auth._id
        {
            let auth_doc = to_document(&auth).map_err(|_| AuthError::AuthDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(auth_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": auth_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(AuthError::AuthNotFound)
                        }
                        else
                        {
                            Ok(auth)
                        }
                    },
                Err(_) => Err(AuthError::AuthDocNotUpdated),
            }
        }
        else
        {
            Err(AuthError::AuthNotFound)
        }
    }

    pub async fn delete (&self, id: AuthID) -> Result<(), AuthError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id) };
        let delete_result = collection.delete_one(filter).await;

        match delete_result
        {
            Ok(result) =>
                {
                    if result.deleted_count == 0
                    {
                        Err(AuthError::AuthNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(AuthError::AuthNotFound),
        }
    }

    pub async fn fetch_by_username(&self, username: String) -> Result<Auth, AuthError>
    {
        let collection = &self.collection;
        let filter = doc! { "username": username};
        let auth_doc = collection.find_one(filter)
            .await
            .map_err(|_| AuthError::AuthNotFound)?
            .ok_or(AuthError::AuthNotFound)?;

        let auth: Auth = from_document(auth_doc).map_err(|_| AuthError::AuthNotFound)?;
        Ok(auth)
    }

    pub async fn fetch_by_email(&self, email: String) -> Result<Auth, AuthError>
    {
        let collection = &self.collection;
        let filter = doc! { "email": email};
        let auth_doc = collection.find_one(filter)
            .await
            .map_err(|_| AuthError::AuthNotFound)?
            .ok_or(AuthError::AuthNotFound)?;

        let auth: Auth = from_document(auth_doc).map_err(|_| AuthError::AuthNotFound)?;
        Ok(auth)
    }
}


#[async_trait]
impl AuthRepo for MongoAuthRepo {
  
}
