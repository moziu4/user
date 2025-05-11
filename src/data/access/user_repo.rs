use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection};
use mongodb::bson::{from_document, to_document};
use crate::core::domain::user::{user_repo::UserRepo, User};
use crate::core::domain::user::user_error::UserError;
use crate::utils::domains_ids::UserID;

#[derive(Clone, Debug)]
pub struct MongoUserRepo
{
    collection: Collection<Document>,
}

impl MongoUserRepo
{
    pub fn new(collection: Collection<Document>) -> Self
    {
        Self { collection }
    }

    pub async fn create (&self, mut new_user: User) -> Result<User, UserError>
    {
        if new_user._id.is_none()
        {
            new_user._id = Some(UserID::new());
        }
        let collection = &self.collection;
        let user_doc = to_document(&new_user).map_err(|_| UserError::UserDocumentNotCreated)?;
        let insert_result = collection.insert_one(user_doc).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id()
        {
            Ok(User { _id:          Some(UserID::from_object_id(inserted_id)),
                username: new_user.username.clone(),
                email: new_user.email.clone(),
                
                name: new_user.name.clone(),
            })
        }
        else
        {
            Err(UserError::UserNotFound)
        }
    }
    pub async fn fetch_all(&self) -> Result<Vec<User>, UserError>
    {
        let filter = doc! {};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| UserError::UserNotFound)?;

        let mut users: Vec<User> = Vec::new();
        while let Some(user_doc) = cursor.try_next()
            .await
            .map_err(|_| UserError::UserNotFound)?
        {
            let user: User = from_document(user_doc).map_err(|_| UserError::UserNotFound)?;
            users.push(user);
        }
        Ok(users)
    }

    pub async fn fetch_all_actives(&self) -> Result<Vec<User>, UserError>
    {
        let filter = doc! {"status": "Active"};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| UserError::UserNotFound)?;

        let mut users: Vec<User> = Vec::new();
        while let Some(user_doc) = cursor.try_next()
            .await
            .map_err(|_| UserError::UserNotFound)?
        {
            let user: User = from_document(user_doc).map_err(|_| UserError::UserNotFound)?;
            users.push(user);
        }
        Ok(users)
    }

    pub async fn fetch_by_id(&self, id: UserID) -> Result<User, UserError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id)};
        let user_doc = collection.find_one(filter)
            .await
            .map_err(|_| UserError::UserNotFound)?
            .ok_or(UserError::UserNotFound)?;

        let user: User = from_document(user_doc).map_err(|_| UserError::UserNotFound)?;
        Ok(user)
    }

    pub async fn fetch_by_email(&self, email: String) -> Result<User, UserError>
    {
        let collection = &self.collection;
        let filter = doc! {"email": email};
        let user_doc = collection.find_one(filter)
            .await
            .map_err(|_| UserError::UserNotFound)?
            .ok_or(UserError::UserNotFound)?;

        let user: User = from_document(user_doc).map_err(|_| UserError::UserNotFound)?;
        Ok(user)
    }

    pub async fn save (&self, user: User) -> Result<User, UserError>
    {
        let collection = &self.collection;
        if let Some(user_id) = &user._id
        {
            let user_doc = to_document(&user).map_err(|_| UserError::UserDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(user_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": user_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(UserError::UserNotFound)
                        }
                        else
                        {
                            Ok(user)
                        }
                    },
                Err(_) => Err(UserError::UserDocNotUpdated),
            }
        }
        else
        {
            Err(UserError::UserNotFound)
        }
    }

    pub async fn delete (&self, id: UserID) -> Result<(), UserError>
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
                        Err(UserError::UserNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(UserError::UserNotFound),
        }
    }

    // fn document_to_user(document: Document) -> Result<User, UserError> {
    //     let user_id = document.get_object_id("_id").ok();
    //     let username = document.get_str("username").ok();
    //     let email = document.get_str("email").ok();
    //     let name = document.get_str("name").ok();
    // 
    //     if let (Some(user_id), Some(username), Some(email), Some(name)) =
    //         (user_id, username, email, name)
    //     {
    //         Ok(User {
    //             id: Some(user_id),
    //             username: username.to_string(),
    //             email: email.to_string(),
    //             name: name.to_string(),
    //         })
    //     } else {
    //         Err(UserError {
    //             message: "Incomplete user document found.".to_string(),
    //         })
    //     }
    // }

    
}

#[async_trait]
impl UserRepo for MongoUserRepo
{
    //     async fn insert_user(&self, user_new: UserNew) -> Result<User, UserError>
    //     {
    //         let user_collection = self.get_collection();
    // 
    // 
    //         if user_collection.find_one(doc! { "username": &user_new.username })
    //                           .await
    //                           .map_err(|err| UserError { message: format!("Failed to check username: {}", err), })?
    //                           .is_some()
    //         {
    //             return Err(UserError { message: "Username already exists".to_string(), });
    //         }
    // 
    //         // Verificar si el email ya existe
    //         if user_collection.find_one(doc! { "email": &user_new.email })
    //                           .await
    //                           .map_err(|err| UserError { message: format!("Failed to check email: {}", err), })?
    //                           .is_some()
    //         {
    //             return Err(UserError { message: "Email already exists".to_string(), });
    //         }
    // 
    //         // Verificar la validez del email
    //         if !user_new.email.contains('@')
    //         {
    //             return Err(UserError { message: "Invalid email format".to_string(), });
    //         }
    // 
    // 
    //         // Primero insertar el User y obtener su user_id
    //         let user_doc = doc! {
    //             "username": &user_new.username,
    //             "email": &user_new.email,
    //             "name": &user_new.name,
    //         };
    // 
    //         let user_result =
    //             user_collection.insert_one(user_doc)
    //                            .await
    //                            .map_err(|err| UserError { message: format!("Failed to insert user: {}", err), })?;
    // 
    //         let user_id = user_result.inserted_id
    //                                  .as_object_id()
    //                                  .ok_or(UserError { message: "Failed to retrieve inserted user ID.".to_string(), })?;
    // 
    //         let user = User { id:       Some(user_id),
    //                           username: user_new.username.clone(),
    //                           email:    user_new.email.clone(),
    //                           name:     user_new.name.clone(), };
    // 
    //         Ok(user)
    //     }
    // 
    // 
   
    // 
    // 
    //     async fn load_user_by_username(&self, username: String) -> Result<User, UserError>
    //     {
    //         let collection = self.get_collection();
    //         let filter = doc! { "username": username };
    // 
    //         let user_doc = collection.find_one(filter)
    //                                  .await
    //                                  .map_err(|err| UserError { message: format!("Failed to find user by username: {}",
    //                                                                              err), })?;
    // 
    //         if let Some(document) = user_doc
    //         {
    //             document_to_user(document)
    //         }
    //         else
    //         {
    //             Err(UserError { message: "User not found.".to_string() })
    //         }
    //     }
    // 
    
}