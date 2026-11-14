pub mod identity_provider_dao;

use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::ports::identity_porvider::IdentityProviderRepository;
use anyhow::{Result, anyhow};
use sqlx::postgres::PgRow;
use sqlx::{Error, PgPool, PgTransaction, Row, query};
use std::collections::HashSet;
use std::str::FromStr;
use tms_lib::utils::service_error::{ServiceError::BadRequest, ServiceError::NotFound};

pub struct PostgresIdentityProviderRepository {
    db_pool: PgPool
}

#[async_trait::async_trait]
impl IdentityProviderRepository for PostgresIdentityProviderRepository {
    async fn get_login_providers(&self) -> Result<HashSet<IdentityProvider>> {
        let idp_query_result = query(
            "select id, name, client_id, client_secret, identity_redirect_url,
                         oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                         oauth2_public_key, scope, provider_type,
                         supports_login, supports_resources, resources_endpoint, created, updated
                         from identity_providers where supports_login = true",
        )
        .fetch_all(&self.db_pool)
        .await?;
    
        let mut idps: Vec<IdentityProvider> = vec![];
        for row in &idp_query_result {
            idps.push(IdentityProvider::try_from(row)?);
        }

        Ok(HashSet::from_iter(idps))
    }

    async fn get_login_provider_by_id(
        &self,
        id: &str,
    ) -> Result<IdentityProvider> {
        let row = query(
            "select id, name, client_id, client_secret, identity_redirect_url,
                         oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                         oauth2_public_key, scope, provider_type,
                         supports_login, supports_resources, resources_services_url, created, updated 
                         from identity_providers where id = $1 and supports_login = true",
        )
        .bind(id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Idp id {} not found", id)).into(),
            _ => anyhow::anyhow!(error),
        })?;
        IdentityProvider::try_from(&row)
    }
    
    async fn list_resource_providers(
        &self,
        tms_identity: &str,
        linked_only: &bool,
    ) -> Result<HashSet<IdentityProvider>> {
        let rp_query_result = match linked_only {
            true => {
                query(
                    "select ip.* from identity_providers as ip
                          INNER JOIN resource_provider_account_logins
                          AS rpal ON ip.id = rpal.rp_id where supports_resources = true
                          and tms_identity = $1",
                )
                .bind(tms_identity)
                .fetch_all(&self.db_pool)
                .await?
            }
            false => {
                query(
                    "select * from identity_providers
                         where supports_resources = true",
                )
                .fetch_all(&self.db_pool)
                .await?
            }
        };
    
        let mut rps: Vec<IdentityProvider> = vec![];
        for row in &rp_query_result {
            rps.push(IdentityProvider::try_from(row)?);
        }
        Ok(HashSet::from_iter(rps))
    }

    async fn get_resource_provider_by_id(
        &self,
        provider_id: &str,
    ) -> Result<IdentityProvider> {
        match query(
            "select * from identity_providers
                         where id = $1 and supports_resources = true",
        )
            .bind(provider_id)
            .fetch_one(&self.db_pool)
            .await
        {
            Ok(row) => Ok(IdentityProvider::try_from(&row)?),
            Err(Error::RowNotFound) => {
                Err(BadRequest("Resource provider not found".to_string()).into())
            }
            Err(error) => Err(anyhow!(error)),
        }
    }
}

// Identity providers can be for either resources or for logins.  There's a boolean for
// the support of each - supports_login, supports_resources.  I guess in retrospect it should
// have been named login_allowed and resources_allowed because it's not about support, but
// rather if we allow it.

impl TryFrom<&PgRow> for IdentityProvider {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("provider_type");
        Ok(IdentityProvider {
            id: row.get("id"),
            name: row.get("name"),
            client_id: row.get("client_id"),
            client_secret: row.get("client_secret"),
            identity_redirect_url: row.get("identity_redirect_url"),
            oauth2_token_url: row.get("oauth2_token_url"),
            oauth2_jwks_url: row.get("oauth2_jwks_url"),
            oidc_user_info_url: row.get("oidc_user_info_url"),
            oauth2_public_key: row.get("oauth2_public_key"),
            scope: row.get("scope"),
            identity_provider_type: IdentityProviderType::from_str(provider)?,
            supports_login: row.get("supports_login"),
            supports_resources: row.get("supports_resources"),
            resources_services_url: row.get("resources_services_url"),
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}
/*
Returns the list of identity providers that support login
 */

// #[cfg(test)]
// mod tests {
//     use crate::infra::persistence::identity_provider_repository::IdentityProviderType;
//     use std::str::FromStr;

//     #[test]
//     fn test_idp_type_from_str() {
//         assert!(IdentityProviderType::from_str("test").is_err());
//         assert_eq!(IdentityProviderType::from_str("Globus").unwrap(), IdentityProviderType::Globus);
//         assert_eq!(
//             IdentityProviderType::from_str("tacc_tapis").unwrap(),
//             IdentityProviderType::TaccTapis
//         );
//     }
// }
