use chrono::Utc;
use sqlx::{PgPool, PgTransaction};
use chrono::Duration;
use tms_lib::utils::service_error::ServiceError::{BadRequest, NotFound};
use crate::db::client_dao::{db_get_client_by_id, db_get_client_by_name};
use crate::db::delegations_dao::{db_delete_delegation, db_get_delegations, db_insert_delegation};
use crate::db::resource_provider_logins_dao::{db_get_resource_provider_login};
use crate::obj_model::delegation::Delegation;
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{token_matches_client, SecurityContext};

pub async fn add_delegation(
    db_pool: &PgPool, security_context: &SecurityContext, client_name:&String, rp_id:&String, rp_account:&String
) -> anyhow::Result<Delegation> {
    let tms_identity = &security_context.tms_identity;
    let configuration = Configuration::get(&db_pool).await?;
    let expiration = &configuration.delegation_policy_config.get_delegation_expiration()?;
    let mut tx = db_pool.begin().await?;

    // First check that the rp_id/rp_account are linked, enabled, and not expired
    let _ = match ensure_recent_rp_login(&mut tx, tms_identity, rp_account, rp_id, configuration.delegation_policy_config.delegation_max_mins_since_login).await {
        Ok(recent_login) => Some(recent_login),
        Err(e) => return Err(e.into()),
    };

    let client = db_get_client_by_name(&mut tx, &client_name).await?;
    let client_id = client.client_id;
    if ! token_matches_client(security_context, &client_id) {
        return Err(BadRequest(
            format!("Token client '{}' is not allowed to delegate for client id: '{}' name: '{}'",
                    security_context.client_id, &client_id, client_name)).into());
    }

    let delegation = db_insert_delegation(&mut tx, &client_id, rp_account,
                                          &expiration, tms_identity, rp_id).await?;
    tx.commit().await?;
    Ok(delegation)
}

async fn ensure_recent_rp_login<'a>(tx: &mut PgTransaction<'a>, tms_identity:&String,
                                    rp_account:&String, rp_id:&String, max_mins_since_login:i64
) -> anyhow::Result<()> {
    let last_allowed_login = Utc::now() - Duration::minutes(max_mins_since_login);
    if let Some(_) = db_get_resource_provider_login(tx, tms_identity,
                                                         rp_account, rp_id, last_allowed_login).await? {
        return Ok(());
    } else {
        return Err(NotFound("User must authenticate to delegate".to_string()).into());
    }
}

pub async fn get_delegations(
    db_pool: &PgPool, security_context: &SecurityContext, client_name:&Option<String>) -> anyhow::Result<Vec<Delegation>> {
    let mut tx = db_pool.begin().await?;
    let mut client_id_for_delegations = None;

    match client_name {
        // must be tms client to get all delegations - if no client id is specified, and the token is for
        // a client other than the tms portal client, set the client id to what's in the token
        None => {
            if ! security_context.is_tms_client {
                let client = db_get_client_by_id(&mut tx, &security_context.client_id).await?;
                client_id_for_delegations = Some(client.client_id)
            }
        }

        // Since we have a client name, it must match what's in the token or the token must be for the
        // tms portal (which is allowed to view all delegations for a user)
        Some(client_name) => {
            let client = db_get_client_by_name(&mut tx, &client_name).await?;
            let client_id = client.client_id;

            if ! token_matches_client(security_context, &client_id) {
                return Err(BadRequest(
                    format!("Token client '{}' is not allowed to delegate for client id: '{}' name: '{}'",
                            &security_context.client_id, &client_id, &client_name)).into());
            }
        }
    }

    let delegations = db_get_delegations(&mut tx, &client_id_for_delegations,
                                         &security_context.tms_identity).await?;
    tx.commit().await?;
    Ok(delegations)
}
pub async fn delete_delegation(
    db_pool: &PgPool, security_context: &SecurityContext, client_name:&String, delegation_id:i32) -> anyhow::Result<Delegation> {
    let tms_identity = &security_context.tms_identity;
    // let configuration = Configuration::get(&db_pool).await?;
    let mut tx = db_pool.begin().await?;

    // I feel like we don't need a recent auth for this ...  but I could be wrong
    // // First check that the rp_id/rp_account are linked, enabled, and not expired
    // let _ = match ensure_recent_rp_login(&mut tx, tms_identity, rp_account, rp_id, configuration.delegation_policy_config.delegation_max_mins_since_login).await {
    //     Ok(recent_login) => Some(recent_login),
    //     Err(e) => return Err(e.into()),
    // };

    let client = db_get_client_by_name(&mut tx, &client_name).await?;
    let client_id = client.client_id;
    if ! token_matches_client(security_context, &client_id) {
        return Err(BadRequest(
            format!("Token client '{}' is not allowed to delegate for client id: '{}' name: '{}'",
                    security_context.client_id, &client_id, client_name)).into());
    }

    let delegation = db_delete_delegation(&mut tx, tms_identity, &client_id, delegation_id).await?;
    tx.commit().await?;
    Ok(delegation)
}
