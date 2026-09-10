use tms_lib::utils::service_error::ServiceError::BadRequest;
use anyhow::{anyhow, Result};
use sqlx::postgres::PgRow;
use sqlx::{query, Error, PgTransaction, Row};
use crate::obj_model::allowed_redirects::AllowedRedirect;

impl From<&PgRow> for AllowedRedirect {
    fn from(row: &PgRow) -> Self {
        AllowedRedirect {
            uri: row.get("uri"),
            client_id: row.get("client_id"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

pub async fn db_get_allowed_redirect<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: &String,
    redirect_uri: &String,
) -> Result<AllowedRedirect> {
    match query(
        "SELECT uri, client_id, created, updated FROM allowed_redirects WHERE client_id = $1 and uri = $2",
    )
        .bind(client_id)
        .bind(redirect_uri)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AllowedRedirect::from(&row)),
        Err(Error::RowNotFound) => Err(BadRequest("Invalid redirect uri".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
pub async fn db_get_allowed_redirect_by_client_name<'a>(
    tx: &mut PgTransaction<'a>,
    client_name: &String,
    redirect_uri: &String,
) -> Result<AllowedRedirect> {
    match query(
        "SELECT ar.uri, ar.client_id, ar.created, ar.updated FROM allowed_redirects ar INNER JOIN clients cl ON ar.client_id = cl.client_id WHERE cl.name = $1 and ar.uri = $2",
    )
        .bind(client_name)
        .bind(redirect_uri)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AllowedRedirect::from(&row)),
        Err(Error::RowNotFound) => Err(BadRequest(format!("Invalid uri for client name: {}", client_name)).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
