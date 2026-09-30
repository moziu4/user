use crate::core::domain::membership::{
    Membership,
    membership_repo::MembershipRepo,
    membership_error::{MembershipError, MembershipResult},
    membership_commands::MembershipCommand,
    MembershipStatus,
};
use crate::data::access::membership_repo::MongoMembershipRepo;
use crate::context::Context;

pub struct MembershipOps<'a> {
    repo: &'a MongoMembershipRepo,
    context: &'a Context,
}

impl<'a> MembershipOps<'a> {
    pub fn new(repo: &'a MongoMembershipRepo, context: &'a Context) -> Self {
        Self {
            repo,
            context,
        }
    }

    pub async fn execute_command(&self, membership_id: Option<&str>, command: MembershipCommand) -> MembershipResult<Membership> {
        match command {
            MembershipCommand::CreateTenant { user_id, tenant_id, role_id } => {
                let membership = Membership::new_tenant(user_id.clone(), tenant_id.clone(), role_id);
                let created = self.repo.create(membership, self.context).await?;

                if let Some(nats) = &self.context.nats_service {
                    let msg = MembershipCommand::CreateTenant { user_id, tenant_id, role_id };
                    nats.publish_membership_command(&msg).await;
                }

                Ok(created)
            }
            MembershipCommand::CreateOrganization { user_id, organization_id, role_id } => {
                let membership = Membership::new_organization(user_id.clone(), organization_id.clone(), role_id);
                let created = self.repo.create(membership, self.context).await?;

                if let Some(nats) = &self.context.nats_service {
                    let msg = MembershipCommand::CreateOrganization { user_id, organization_id, role_id };
                    nats.publish_membership_command(&msg).await;
                }

                Ok(created)
            }
            MembershipCommand::UpdateStatus { status } => {
                let id = membership_id.ok_or(MembershipError::InvalidMembership)?;
                let status_str = match status {
                    MembershipStatus::Active => "Active",
                    MembershipStatus::Inactive => "Inactive",
                    MembershipStatus::Suspended => "Suspended",
                };
                self.repo.update_status(self.context, id, status_str).await?;

                if let Some(nats) = &self.context.nats_service {
                    let msg = MembershipCommand::UpdateStatus { status };
                    nats.publish_membership_command_with_id(id, &msg).await;
                }

                self.repo.get_membership(self.context, id).await
            }
            MembershipCommand::DeactivateUserFromOrganization { user_id, organization_id } => {
                self.repo.deactivate_organization_and_tenants(self.context, user_id.clone(), organization_id.clone()).await?;

                if let Some(nats) = &self.context.nats_service {
                    let msg = MembershipCommand::DeactivateUserFromOrganization { user_id, organization_id };
                    nats.publish_membership_command(&msg).await;
                }

                // Devolvemos una membresía vacía o buscamos una relacionada para cumplir con el tipo de retorno
                Err(MembershipError::MembershipNotFound)
            }
        }
    }

    pub async fn get_membership_for_user(&self, user_id: &str) -> MembershipResult<Vec<Membership>> {
        self.repo.get_membership_for_user(self.context, user_id).await
    }

    pub async fn get_membership(&self, membership_id: &str) -> MembershipResult<Membership> {
        self.repo.get_membership(self.context, membership_id).await
    }
}
