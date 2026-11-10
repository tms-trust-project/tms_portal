pub mod allowed_redirects_dao;

use crate::obj_model::allowed_redirects::AllowedRedirect;
use crate::ports::allowed_redirect::AllowedRedirectRepository;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row, query};
use tms_lib::utils::service_error::ServiceError::BadRequest;

pub struct PostgresAllowedRedirectRepository {
    db_pool: PgPool,
}

impl PostgresAllowedRedirectRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    fn from_row(row: &PgRow) -> AllowedRedirect {
        AllowedRedirect {
            uri: row.get("uri"),
            client_id: row.get("client_id"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

#[async_trait::async_trait]
impl AllowedRedirectRepository for PostgresAllowedRedirectRepository {
    async fn get_by_client_id_and_uri(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<AllowedRedirect> {
        let row = query(
            "SELECT uri, client_id, created, updated FROM allowed_redirects WHERE client_id = $1 and uri = $2",
        )
        .bind(client_id)
        .bind(redirect_uri)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => {
                BadRequest("Invalid redirect uri".to_string()).into()
            }
            _ => anyhow::anyhow!(error),
        })?;

        Ok(Self::from_row(&row))
    }

    async fn get_by_client_name_and_uri(
        &self,
        client_name: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<AllowedRedirect> {
        let row = query(
            "SELECT ar.uri, ar.client_id, ar.created, ar.updated FROM allowed_redirects ar INNER JOIN clients cl ON ar.client_id = cl.client_id WHERE cl.name = $1 and ar.uri = $2",
        )
        .bind(client_name)
        .bind(redirect_uri)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => {
                BadRequest(format!("Invalid uri for client name: {}", client_name)).into()
            }
            _ => anyhow::anyhow!(error),
        })?;

        Ok(Self::from_row(&row))
    }
}
