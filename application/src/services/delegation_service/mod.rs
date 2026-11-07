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
        Self {
            client_repo,
            delegation_repo,
            resource_provider_login_repo,
            delegation_policy,
        }
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
        if !ctx.token_matches_client(&client_id) {
            return Err(BadRequest(format!(
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
            None => return Err(Unauthorized("User must reauthenticate to delegate".to_string()).into())
        };

        let delegation = self.delegation_repo
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

    pub async fn get_delegations(
        &self,
        ctx: &SecurityContext,
        client_name: &Option<String>,
    ) -> anyhow::Result<Vec<Delegation>> {
        let mut client_id_for_delegations = None;

        match client_name {
            // must be tms client to get all delegations - if no client id is specified, and the token is for
            // a client other than the tms portal client, set the client id to what's in the token
            None => {
                if !ctx.is_tms_client {
                    let client = self.client_repo.get_by_id(&ctx.client_id).await?;
                    client_id_for_delegations = Some(client.client_id)
                }
            }

            // Since we have a client name, it must match what's in the token or the token must be for the
            // tms portal (which is allowed to view all delegations for a user)
            Some(client_name) => {
                let client = self.client_repo.get_by_name(client_name).await?;
                let client_id = client.client_id;

                if !ctx.token_matches_client(&client_id) {
                    return Err(BadRequest(format!(
                        "Token client '{}' is not allowed to delegate for client id: '{}' name: '{}'",
                        &ctx.client_id, &client_id, &client_name
                    ))
                    .into());
                }
            }
        }

        let delegations = match client_id_for_delegations {
            Some(client_id) => {
                self.delegation_repo
                    .list_by_client_id(&client_id, &ctx.tms_identity)
                    .await?
            }
            None => self.delegation_repo.list_all(&ctx.tms_identity).await?,
        };

        Ok(delegations)
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
        if !ctx.token_matches_client(&client.client_id) {
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