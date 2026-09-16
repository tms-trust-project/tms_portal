use url::Url;
use crate::db::allowed_redirects_dao::{db_get_allowed_redirect, db_get_allowed_redirect_by_client_name};
use anyhow::{Context, Result};
use sqlx::PgTransaction;

pub async fn check_allowed_redirects<'a>( tx: &mut PgTransaction<'a>, client_id:&String, url:&String) -> Result<()> {
    // Check the allowed redirect, but ignore query params
    let mut bareUrl = Url::parse(url)?;
    bareUrl.set_query(None);
    let _ = db_get_allowed_redirect(tx, &client_id, &bareUrl.to_string()).await
        .with_context(||format!("Requested redirect url: {0} is not found for client id: {1}",  url, client_id))?;
    Ok(())
}

pub async fn check_allowed_redirects_by_client_name<'a>( tx: &mut PgTransaction<'a>, client_name:&String, url:&String) -> Result<()> {
    // Check the allowed redirect, but ignore query params
    let mut bareUrl = Url::parse(url)?;
    bareUrl.set_query(None);
    let _ = db_get_allowed_redirect_by_client_name(tx, &client_name, &bareUrl.to_string()).await
        .with_context(||format!("Requested redirect url: {0} is not found for client name: {1}",  url, client_name))?;
    Ok(())
}
