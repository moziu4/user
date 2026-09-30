use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::{bson::{doc, oid::ObjectId, Document}, Collection};
use mongodb::bson::{from_document, to_document};
use crate::core::domain::membership::{Membership, membership_repo::MembershipRepo, membership_error::{MembershipError, MembershipResult}};
use crate::context::Context;
use crate::utils::domains_ids::{MembershipID, OrganizationID};
use perms::UserID;

#[derive(Clone, Debug)]
pub struct MongoMembershipRepo {
    collection: Collection<Document>,
}

impl MongoMembershipRepo {
    pub fn new(collection: Collection<Document>) -> Self {
        Self { collection }
    }
}

#[async_trait]
impl MembershipRepo for MongoMembershipRepo {
    async fn create(&self, mut membership: Membership, context: &Context) -> MembershipResult<Membership> {
        let coll = context.get_collection("memberships");
        if membership._id.is_none() {
            membership._id = Some(MembershipID::new());
        }
        let doc = to_document(&membership).map_err(|_| MembershipError::MembershipOperationFailed)?;
        coll.insert_one(doc).await.map_err(|_| MembershipError::MembershipOperationFailed)?;
        Ok(membership)
    }

    async fn get_membership(&self, context: &Context, membership_id: &str) -> MembershipResult<Membership> {
        let coll = context.get_collection("memberships");
        let obj_id = ObjectId::parse_str(membership_id).map_err(|_| MembershipError::InvalidMembership)?;
        let filter = doc! { "_id": obj_id };
        let doc = coll.find_one(filter).await.map_err(|_| MembershipError::MembershipNotFound)?
            .ok_or(MembershipError::MembershipNotFound)?;
        let membership: Membership = from_document(doc).map_err(|_| MembershipError::InvalidMembership)?;
        Ok(membership)
    }

    async fn get_membership_for_user(&self, context: &Context, user_id: &str) -> MembershipResult<Vec<Membership>> {
        let coll = context.get_collection("memberships");
        let user_oid = ObjectId::parse_str(user_id).map_err(|_| MembershipError::InvalidMembership)?;
        let filter = doc! { "user_id": user_oid };
        let mut cursor = coll.find(filter).await.map_err(|_| MembershipError::MembershipNotFound)?;
        let mut memberships = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|_| MembershipError::MembershipNotFound)? {
            let m: Membership = from_document(doc).map_err(|_| MembershipError::InvalidMembership)?;
            memberships.push(m);
        }
        Ok(memberships)
    }

    async fn update_status(&self, context: &Context, membership_id: &str, status: &str) -> MembershipResult<()> {
        let coll = context.get_collection("memberships");
        let obj_id = ObjectId::parse_str(membership_id).map_err(|_| MembershipError::InvalidMembership)?;
        let filter = doc! { "_id": obj_id };
        let update = doc! { "$set": { "status": status } };
        let result = coll.update_one(filter, update).await.map_err(|_| MembershipError::MembershipOperationFailed)?;
        if result.matched_count == 0 {
            return Err(MembershipError::MembershipNotFound);
        }
        Ok(())
    }

    async fn deactivate_organization_and_tenants(&self, context: &Context, user_id: UserID, organization_id: OrganizationID) -> MembershipResult<()> {
        let coll = context.get_collection("memberships");
        let user_oid = ObjectId::from(user_id);
        let org_oid = ObjectId::from(organization_id);

        // 1. Desactivar la membresía de la organización
        let org_filter = doc! { "user_id": user_oid, "target.Organization": org_oid };
        let update = doc! { "$set": { "status": "Inactive" } };
        coll.update_many(org_filter, update.clone()).await.map_err(|_| MembershipError::MembershipOperationFailed)?;

        // 2. Desactivar también los tenants relacionados (si guardamos relación o si se gestiona en cascada)
        let tenant_filter = doc! { "user_id": user_oid, "target.Tenant": { "$exists": true } };
        coll.update_many(tenant_filter, update).await.map_err(|_| MembershipError::MembershipOperationFailed)?;

        Ok(())
    }
}
