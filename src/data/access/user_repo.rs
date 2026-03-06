use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection};
use mongodb::bson::{from_document, to_document};
use perms::UserID;
use crate::core::domain::user::{user_repo::UserRepo, User};
use crate::core::domain::membership::Membership;
use crate::core::domain::user::user_type::{Phone, Address};
use crate::utils::domains_ids::{MembershipID, PhoneID, AddressID, TenantID, AgencyID};
use crate::core::domain::user::user_error::UserError;


#[derive(Clone, Debug)]
pub struct MongoUserRepo
{
    collection:          Collection<Document>,
    memberships_coll:    Collection<Document>,
    phones_coll:         Collection<Document>,
    addresses_coll:      Collection<Document>,
}

impl MongoUserRepo
{
    pub fn new(
        collection:          Collection<Document>,
        memberships_coll:    Collection<Document>,
        phones_coll:         Collection<Document>,
        addresses_coll:      Collection<Document>,
    ) -> Self
    {
        Self { 
            collection,
            memberships_coll,
            phones_coll,
            addresses_coll,
        }
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
                username:      new_user.username.clone(),
                email:         new_user.email.clone(),
                name:          new_user.name.clone(),
                status:        new_user.status,
                deleted_at:    new_user.deleted_at,
                anonymized_at: new_user.anonymized_at,
                gpd_erased:    new_user.gpd_erased,
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
        let filter = doc! { "_id": ObjectId::from(id) };
        let update = doc! { "$set": { "status": "Deleted", "deleted_at": mongodb::bson::DateTime::now() } };
        
        let result = self.collection.update_one(filter, update).await
            .map_err(|_| UserError::UserDocNotUpdated)?;

        if result.matched_count == 0 {
            return Err(UserError::UserNotFound);
        }
        Ok(())
    }

    pub async fn create_membership(&self, mut membership: Membership) -> Result<Membership, UserError> {
        if membership._id.is_none() {
            membership._id = Some(MembershipID::new());
        }
        let doc = to_document(&membership).map_err(|_| UserError::UserDocumentNotCreated)?;
        self.memberships_coll.insert_one(doc).await?;
        Ok(membership)
    }

    pub async fn fetch_memberships_by_user(&self, user_id: UserID) -> Result<Vec<Membership>, UserError> {
        let filter = doc! { "user_id": ObjectId::from(user_id) };
        let mut cursor = self.memberships_coll.find(filter).await?;
        let mut memberships = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            let m: Membership = from_document(doc).map_err(|_| UserError::UserNotFound)?;
            memberships.push(m);
        }
        Ok(memberships)
    }

    pub async fn create_phone(&self, mut phone: Phone) -> Result<Phone, UserError> {
        if phone._id.is_none() {
            phone._id = Some(PhoneID::new());
        }
        let doc = to_document(&phone).map_err(|_| UserError::UserDocumentNotCreated)?;
        self.phones_coll.insert_one(doc).await?;
        Ok(phone)
    }

    pub async fn create_address(&self, mut address: Address) -> Result<Address, UserError> {
        if address._id.is_none() {
            address._id = Some(AddressID::new());
        }
        let doc = to_document(&address).map_err(|_| UserError::UserDocumentNotCreated)?;
        self.addresses_coll.insert_one(doc).await?;
        Ok(address)
    }

    pub async fn deactivate_tenant_membership(&self, user_id: UserID, tenant_id: TenantID) -> Result<(), UserError> {
        let filter = doc! { "user_id": ObjectId::from(user_id), "tenants.tenant_id": ObjectId::from(tenant_id) };
        let update = doc! { "$set": { "tenants.$.status": "Inactive" } };
        let result = self.memberships_coll.update_one(filter, update).await?;
        if result.matched_count == 0 {
            return Err(UserError::UserNotFound);
        }
        Ok(())
    }

    pub async fn deactivate_agency_membership(&self, user_id: UserID, agency_id: AgencyID) -> Result<(), UserError> {
        let filter = doc! { "user_id": ObjectId::from(user_id), "agencies.agency_id": ObjectId::from(agency_id) };
        let update = doc! { "$set": { "agencies.$.status": "Inactive" } };
        let result = self.memberships_coll.update_one(filter, update).await?;
        if result.matched_count == 0 {
            return Err(UserError::UserNotFound);
        }
        Ok(())
    }

    pub async fn anonymize_user_in_tenant(&self, user_id: UserID, tenant_id: TenantID) -> Result<(), UserError> {
        // 1. Eliminar teléfonos asociados al usuario y tenant
        let phone_filter = doc! { "user_id": ObjectId::from(user_id.clone()), "tenant_id": ObjectId::from(tenant_id.clone()) };
        self.phones_coll.delete_many(phone_filter).await?;

        // 2. Eliminar direcciones asociadas al usuario y tenant
        let address_filter = doc! { "user_id": ObjectId::from(user_id.clone()), "tenant_id": ObjectId::from(tenant_id.clone()) };
        self.addresses_coll.delete_many(address_filter).await?;

        // 3. Obtener membresía para verificar otros tenants/agencies
        let memberships = self.fetch_memberships_by_user(user_id.clone()).await?;
        let mut total_active_memberships = 0;

        for m in &memberships {
            total_active_memberships += m.tenants.iter().filter(|t| t.status == crate::core::domain::membership::MembershipStatus::Active && t.tenant_id != tenant_id).count();
            total_active_memberships += m.agencies.iter().filter(|a| a.status == crate::core::domain::membership::MembershipStatus::Active).count();
        }

        // 4. Actualizar o eliminar la membresía del tenant
        let filter = doc! { "user_id": ObjectId::from(user_id.clone()) };
        let update = doc! { "$pull": { "tenants": { "tenant_id": ObjectId::from(tenant_id) } } };
        self.memberships_coll.update_one(filter, update).await?;

        // 5. Si no quedan más membresías activas, anonimizar el User globalmente
        if total_active_memberships == 0 {
            let user_filter = doc! { "_id": ObjectId::from(user_id.clone()) };
            let user_update = doc! {
                "$set": {
                    "username": format!("anonymized_{}", user_id),
                    "email": format!("{}@anonymized.com", user_id),
                    "name": "Anonymized",
                    "status": "Anonymized",
                    "anonymized_at": mongodb::bson::DateTime::now(),
                }
            };
            self.collection.update_one(user_filter, user_update).await?;
        }

        Ok(())
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
    async fn create (&self, new_user: User) -> Result<User, UserError> {
        self.create(new_user).await
    }
    async fn fetch_all(&self) -> Result<Vec<User>, UserError> {
        self.fetch_all().await
    }
    async fn fetch_all_actives(&self) -> Result<Vec<User>, UserError> {
        self.fetch_all_actives().await
    }
    async fn fetch_by_id(&self, id: UserID) -> Result<User, UserError> {
        self.fetch_by_id(id).await
    }
    async fn fetch_by_email(&self, email: String) -> Result<User, UserError> {
        self.fetch_by_email(email).await
    }
    async fn save (&self, user: User) -> Result<User, UserError> {
        self.save(user).await
    }
    async fn delete (&self, id: UserID) -> Result<(), UserError> {
        self.delete(id).await
    }
    async fn create_membership(&self, membership: Membership) -> Result<Membership, UserError> {
        self.create_membership(membership).await
    }
    async fn fetch_memberships_by_user(&self, user_id: UserID) -> Result<Vec<Membership>, UserError> {
        self.fetch_memberships_by_user(user_id).await
    }
    async fn create_phone(&self, phone: Phone) -> Result<Phone, UserError> {
        self.create_phone(phone).await
    }
    async fn create_address(&self, address: Address) -> Result<Address, UserError> {
        self.create_address(address).await
    }
    async fn deactivate_tenant_membership(&self, user_id: UserID, tenant_id: TenantID) -> Result<(), UserError> {
        self.deactivate_tenant_membership(user_id, tenant_id).await
    }
    async fn deactivate_agency_membership(&self, user_id: UserID, agency_id: AgencyID) -> Result<(), UserError> {
        self.deactivate_agency_membership(user_id, agency_id).await
    }
    async fn anonymize_user_in_tenant(&self, user_id: UserID, tenant_id: TenantID) -> Result<(), UserError> {
        self.anonymize_user_in_tenant(user_id, tenant_id).await
    }
}