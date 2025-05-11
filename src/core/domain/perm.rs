use serde::{Deserialize, Serialize};
use crate::core::domain::perm::perm_error::PermError;
use crate::data::access::perms_repo::MongoPermRepo;
use crate::utils::domains_ids::PermID;

pub mod perm_cat;
pub mod perm_repo;
pub mod perm_type;
pub mod perm_error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perm
{
    pub _id:         Option<PermID>,
    pub name:        String,
    pub description: String,
}

#[derive(Clone)]
pub struct PermEntity<'a>
{
    props: Perm,
    repo:  &'a MongoPermRepo,
}

impl<'a> PermEntity<'a>
{
    pub async fn new(new_perm: Perm, repo: &'a MongoPermRepo) -> Self
    {
        Self {
            repo,
            props: Perm {
                _id: None,
                name: new_perm.name,
                description: new_perm.description,
            }
        }
    }

    pub async fn create(self) -> Result<Perm, PermError>
    {
        self.repo.create(self.props).await
    }
}