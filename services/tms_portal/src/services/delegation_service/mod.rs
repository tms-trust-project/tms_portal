use std::sync::Arc;

use crate::obj_model::configuration::DelegationPolicyConfig;
use crate::obj_model::delegation::Delegation;
use crate::ports::client::ClientRepository;
use crate::ports::delegation::DelegationRepository;
use crate::ports::resource_provider_login::ResourceProviderLoginRepository;
use crate::utils::jwt_utils::SecurityContext;
use chrono::Duration;
use chrono::Utc;
use tms_lib::utils::service_error::ServiceError::{BadRequest, Unauthorized};

#[derive(Clone)]
pub struct DelegationService {
    client_repo: Arc<dyn ClientRepository>,
    delegation_repo: Arc<dyn DelegationRepository>,
    resource_provider_login_repo: Arc<dyn ResourceProviderLoginRepository>,
    delegation_policy: DelegationPolicyConfig,
}

impl DelegationService {
    pub fn new(
        client_repo: Arc<dyn ClientRepository>,
        delegation_repo: Arc<dyn DelegationRepository>,
        resource_provider_login_repo: Arc<dyn ResourceProviderLoginRepository>,
        delegation_policy: DelegationPolicyConfig,
    ) -> Self {
        Self { client_repo, delegation_repo, resource_provider_login_repo, delegation_policy }
    }

    pub async fn add_delegation(
        &self,
        ctx: &SecurityContext,
        client_name: &str,
        rp_id: &str,
        rp_account: &str,
    ) -> anyhow::Result<Delegation> {
        let client = self.client_repo.get_by_name(client_name).await?;

        let client_id = client.client_id;
        if !ctx.client_is_authorized(&client_id) {
            return Err(Unauthorized(format!(
                "Token client '{}' is not allowed to delegate for client id: '{}' name: '{}'",
                ctx.client_id, &client_id, client_name
            ))
            .into());
        }

        let maybe_recent_login = self
            .resource_provider_login_repo
            .find(
                &ctx.tms_identity,
                rp_account,
                rp_id,
                Utc::now()
                    - Duration::minutes(
                        self.delegation_policy
                            .delegation_max_mins_since_login
                            .clone(),
                    ),
                true,
            )
            .await?;

        let _recent_login = match maybe_recent_login {
            Some(recent_login) => recent_login,
            None => {
                return Err(Unauthorized("User must reauthenticate to delegate".to_string()).into());
            }
        };

        let delegation = self
            .delegation_repo
            .save(
                &client_id,
                rp_account,
                &self.delegation_policy.get_delegation_expiration()?,
                &ctx.tms_identity,
                rp_id,
            )
            .await?;

        Ok(delegation)
    }

    pub async fn list_delegations_for_client(
        &self,
        ctx: &SecurityContext,
        client_name: &str,
    ) -> anyhow::Result<Vec<Delegation>> {
        let client = self.client_repo.get_by_name(client_name).await?;

        if !ctx.client_is_authorized(&client.client_id) {
            return Err(Unauthorized(format!(
                "Token client '{}' is not allowed to list delegations for client id: '{}' name: '{}'",
                &ctx.client_id, &client.client_id, &client_name
            ))
            .into());
        }

        self.delegation_repo
            .list_by_client_id(&client.client_id, &ctx.tms_identity)
            .await
    }

    pub async fn list_delegations(&self, ctx: &SecurityContext) -> anyhow::Result<Vec<Delegation>> {
        // Return all delegations for the tms client
        if ctx.is_tms_client {
            return self.delegation_repo.list_all(&ctx.tms_identity).await;
        }

        // Return only the delegations of the requesting client_id
        self.delegation_repo
            .list_by_client_id(&ctx.client_id, &ctx.tms_identity)
            .await
    }

    pub async fn delete_delegation(
        &self,
        ctx: &SecurityContext,
        client_name: &str,
        delegation_id: i32,
    ) -> anyhow::Result<Delegation> {
        // I feel like we don't need a recent auth for this ...  but I could be wrong
        // // First check that the rp_id/rp_account are linked, enabled, and not expired
        // let _ = match ensure_recent_rp_login(&mut tx, tms_identity, rp_account, rp_id, configuration.delegation_policy_config.delegation_max_mins_since_login).await {
        //     Ok(recent_login) => Some(recent_login),
        //     Err(e) => return Err(e.into()),
        // };

        let client = self.client_repo.get_by_name(client_name).await?;
        if !ctx.client_is_authorized(&client.client_id) {
            return Err(BadRequest(format!(
                "Token client '{}' is not allowed to delegate for client id: '{}' name: '{}'",
                ctx.client_id, &client.client_id, client_name
            ))
            .into());
        }

        let delegation = self
            .delegation_repo
            .delete(&ctx.tms_identity, &client.client_id, delegation_id)
            .await?;

        Ok(delegation)
    }
}

#[cfg(test)]
#[path = "delegation_service.test.rs"]
mod delegation_service_test;

#[cfg(test)]
mod test_fixtures;
