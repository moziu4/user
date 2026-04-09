
use std::collections::HashMap;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::{bson::{doc, from_bson, Document}, Collection};
use mongodb::bson::{from_document, to_document};
use mongodb::bson::oid::ObjectId;
use tracing::log;
use crate::context::Context;
use crate::core::domain::auth::auth_type::Role;
use crate::core::domain::perm::{perm_repo::PermRepo, perm_type::{PermsRelationship}, Perm};
use crate::core::domain::perm::perm_error::PermError;
use crate::utils::domains_ids::PermID;

#[derive(Clone)]
pub struct MongoPermRepo
{
    collection: Collection<Document>,
}

impl MongoPermRepo
{
    pub fn new(collection: Collection<Document>) -> Self
    {
        Self { collection }
    }

    pub async fn create (&self, mut new_perm: Perm) -> Result<Perm, PermError>
    {
        if new_perm._id.is_none()
        {
            new_perm._id = Some(PermID::new());
        }
        let collection = &self.collection;
        let perm_doc = to_document(&new_perm).map_err(|_| PermError::PermDocumentNotCreated)?;
        let insert_result = collection.insert_one(perm_doc).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id()
        {
            Ok(Perm { _id:               Some(PermID::from_object_id(inserted_id)),
                name: new_perm.name,
                description: new_perm.description,
            })
        }
        else
        {
            Err(PermError::PermNotFound)
        }
    }
    pub async fn fetch_all(&self) -> Result<Vec<Perm>, PermError>
    {
        let filter = doc! {};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| PermError::PermNotFound)?;

        let mut perms: Vec<Perm> = Vec::new();
        while let Some(perm_doc) = cursor.try_next()
            .await
            .map_err(|_| PermError::PermNotFound)?
        {
            let perm: Perm = from_document(perm_doc).map_err(|_| PermError::PermNotFound)?;
            perms.push(perm);
        }
        Ok(perms)
    }

    pub async fn fetch_all_actives(&self) -> Result<Vec<Perm>, PermError>
    {
        let filter = doc! {"status": "Active"};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| PermError::PermNotFound)?;

        let mut perms: Vec<Perm> = Vec::new();
        while let Some(perm_doc) = cursor.try_next()
            .await
            .map_err(|_| PermError::PermNotFound)?
        {
            let perm: Perm = from_document(perm_doc).map_err(|_| PermError::PermNotFound)?;
            perms.push(perm);
        }
        Ok(perms)
    }

    pub async fn fetch_by_id(&self, id: PermID) -> Result<Perm, PermError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id)};
        let perm_doc = collection.find_one(filter)
            .await
            .map_err(|_| PermError::PermNotFound)?
            .ok_or(PermError::PermNotFound)?;

        let perm: Perm = from_document(perm_doc).map_err(|_| PermError::PermNotFound)?;
        Ok(perm)
    }

    pub async fn save (&self, perm: Perm) -> Result<Perm, PermError>
    {
        let collection = &self.collection;
        if let Some(perm_id) = &perm._id
        {
            let perm_doc = to_document(&perm).map_err(|_| PermError::PermDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(perm_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": perm_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(PermError::PermNotFound)
                        }
                        else
                        {
                            Ok(perm)
                        }
                    },
                Err(_) => Err(PermError::PermDocNotUpdated),
            }
        }
        else
        {
            Err(PermError::PermNotFound)
        }
    }

    pub async fn delete (&self, id: PermID) -> Result<(), PermError>
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
                        Err(PermError::PermNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(PermError::PermNotFound),
        }
    }
}

#[async_trait]
impl PermRepo for MongoPermRepo
{
    async fn create_perms_relationship(&self, perms_relationships: Vec<PermsRelationship>, context: &Context) -> Result<(), PermError>
    {
        let collection = context.get_collection("relationship");
        let docs: Vec<Document> = perms_relationships.into_iter()
            .map(|perms_relationship| {
                to_document(&perms_relationship).unwrap_or_else(|_| {
                    doc! {
                        "id": perms_relationship.id,
                        "role": perms_relationship.role as u32,
                        "perms": perms_relationship.perms,
                    }
                })
            })
            .collect();

        collection.insert_many(docs)
            .await
            .map_err(|_e| PermError::PermRelationShipNotCreated)?;
        Ok(())
    }

    async fn charge_permissions(&self, role_id: u32, context: &Context) -> Result<Vec<String>, PermError>
    {
        let collection_relationship: Collection<Document> = context.get_collection("relationship");

        // Intentamos buscar por ID numérico o por el nombre del rol (mapeando el ID al enum)
        let role = Role::from_id(role_id);
        let mut filter = doc! { "role": role_id };
        
        if let Some(r) = role {
            let role_name = format!("{:?}", r);
            filter = doc! {
                "$or": [
                    { "role": role_id },
                    { "role": role_name },
                    { "id": role_id } // A veces el catálogo usa 'id' en lugar de 'role' como identificador
                ]
            };
        }

        let result = collection_relationship.find_one(filter.clone())
            .await
            .map_err(|_e| { PermError::PermNotFound }
            )?;

        if let Some(document) = result
        {
            if let Some(perms_bson) = document.get("perms")
            {
                let perms: Vec<String> =
                    from_bson(perms_bson.clone()).map_err(|_e| {
                        PermError::PermNotFound
                    })?;

                return Ok(perms);
            } else {
                log::warn!("'perms' field not found in the relationship document: {:?}", document);
            }
        } else {
            log::warn!("No document found with filter: {:?}", filter);
        }

        Err(PermError::PermNotFound)
    }

    async fn get_perms_map(&self, context: &Context) -> Result<HashMap<String, u32>, PermError> {
        let coll = context.get_collection("perms");
        let mut cursor = coll.find(doc! {}).await
            .map_err(|_| PermError::PermNotFound)?;
        let mut map = HashMap::new();
        while let Some(doc) = cursor.try_next().await.map_err(|_| PermError::PermNotFound)? {
            if let (Ok(name), Ok(id)) = (doc.get_str("name"), doc.get_i32("id")) {
                map.insert(name.to_string(), id as u32);
            } else if let (Ok(name), Ok(id)) = (doc.get_str("name"), doc.get_i64("id")) {
                map.insert(name.to_string(), id as u32);
            }
        }
        Ok(map)
    }
}
