
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
    // async fn insert_auth(
    //     &self,
    //     command: String,
    //     user_new: NewUser,
    //     user_id: Option<ObjectId>,
    //     perm: Vec<u32>,
    // ) -> Result<Auth, AuthError> {
    //     let auth_collection = self.get_collection();
    // 
    //     let hashed_password = hash(&user_new.password, DEFAULT_COST).map_err(|err| AuthError {
    //         message: format!(
    //             "Failed to hash \
    //                                                                                            password: {}",
    //             err
    //         ),
    //     })?;
    // 
    //     let role = match command.as_str() {
    //         "public" => Role::Visitor.to_string(),
    //         "private" => Role::SuperAdmin.to_string(),
    //         _ => Role::Visitor.to_string(),
    //     };
    // 
    //     let auth_doc = doc! {
    //         "user_id": user_id,
    //         "username": &user_new.username,
    //         "email": &user_new.email,
    //         "password": hashed_password.clone(),
    //         "roles": role.clone(),
    //         "permissions": perm.clone(),
    //     };
    // 
    //     let insert_result = auth_collection
    //         .insert_one(auth_doc.clone())
    //         .await
    //         .map_err(|err| AuthError {
    //             message: format!("Failed to insert user auth: {}", err),
    //         })?;
    // 
    //     let inserted_id = insert_result
    //         .inserted_id
    //         .as_object_id()
    //         .ok_or(AuthError {
    //             message: "Failed to retrieve inserted ID".to_string(),
    //         })?;
    // 
    //     let retrieved_doc = auth_collection
    //         .find_one(doc! { "_id": inserted_id })
    //         .await
    //         .map_err(|err| AuthError {
    //             message: format!("Failed to retrieve user auth: {}", err),
    //         })?;
    // 
    //     let document = retrieved_doc.ok_or(AuthError {
    //         message: "No document found with the provided ID".to_string(),
    //     })?;
    // 
    //     let auth = Auth {
    //         id: inserted_id,
    //         user_id: document
    //             .get_object_id("user_id")
    //             .map_err(|_| AuthError {
    //                 message: "Invalid or missing user_id".to_string(),
    //             })?
    //             .clone(),
    //         username: document
    //             .get_str("username")
    //             .map_err(|_| AuthError {
    //                 message: "Invalid or missing username".to_string(),
    //             })?
    //             .to_string(),
    //         email: document
    //             .get_str("email")
    //             .map_err(|_| AuthError {
    //                 message: "Invalid or missing email".to_string(),
    //             })?
    //             .to_string(),
    //         password: document
    //             .get_str("password")
    //             .map_err(|_| AuthError {
    //                 message: "Invalid or missing password".to_string(),
    //             })?
    //             .to_string(),
    //         roles: document
    //             .get_str("roles")
    //             .map_err(|_| AuthError {
    //                 message: "Invalid or missing roles".to_string(),
    //             })?
    //             .to_string()
    //             .parse()
    //             .unwrap(),
    //         permissions: Option::from(
    //             document
    //                 .get_array("permissions")
    //                 .map_err(|_| AuthError {
    //                     message: "Invalid or missing permissions".to_string(),
    //                 })
    //                 .and_then(|arr| {
    //                     arr.iter()
    //                         .map(|b| from_bson(b.clone()))
    //                         .collect::<Result<Vec<u32>, _>>()
    //                         .map_err(|_| AuthError {
    //                             message: "Failed to parse permissions".to_string(),
    //                         })
    //                 })?,
    //         ),
    //     };
    // 
    //     Ok(auth)
    // }
    // 
    // async fn do_login(&self, user_auth: AuthLogin) -> Result<String, AuthError> {
    //     let auth_collection = self.get_collection();
    //     let filter = doc! { "username": &user_auth.username };
    // 
    //     let auth_doc = auth_collection
    //         .find_one(filter)
    //         .await
    //         .map_err(|err| AuthError {
    //             message: format!("Failed to find user session: {}", err),
    //         })?;
    // 
    //     if auth_doc.is_none() {
    //         return Err(AuthError {
    //             message: "Invalid username or password".to_string(),
    //         });
    //     }
    // 
    //     let auth_doc = auth_doc.unwrap();
    // 
    //     let stored_password = auth_doc
    //         .get_str("password")
    //         .map_err(|_| AuthError {
    //             message: "Failed to retrieve password from database".to_string(),
    //         })?;
    // 
    //     let password_matches = verify(&user_auth.password, stored_password).map_err(|err| AuthError {
    //         message: format!(
    //             "Failed to verify \
    //              password: {}",
    //             err
    //         ),
    //     })?;
    // 
    //     if !password_matches {
    //         return Err(AuthError {
    //             message: "Invalid username or password".to_string(),
    //         });
    //     }
    //     // Obtener los permisos del documento de autenticación
    //     let permissions: Vec<u32> = auth_doc
    //         .get_array("permissions")
    //         .map_err(|_| AuthError {
    //             message: "Invalid or missing permissions".to_string(),
    //         })?
    //         .iter()
    //         .filter_map(|val| val.as_i32().map(|v| v as u32))
    //         .collect();
    //     let role = auth_doc
    //         .get_str("roles")
    //         .map_err(|_| AuthError {
    //             message: "Invalid or missing role".to_string(),
    //         })?
    //         .to_string();
    // 
    //     let start = SystemTime::now();
    //     let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    //     let exp = (since_the_epoch.as_secs() + 3600) as usize;
    // 
    //     let claims = Claims {
    //         sub: user_auth.username,
    //         role,
    //         exp,
    //         permissions,
    //     };
    // 
    //     let secret = env::var("SECRET_KEY").unwrap().to_string();
    //     let token =
    //         encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).map_err(|_| AuthError {
    //             message: "Failed to create token".to_string(),
    //         })?;
    // 
    //     Ok(token)
    // }
}
