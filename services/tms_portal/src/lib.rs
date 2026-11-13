pub mod config;
pub mod infra;
pub mod obj_model;
pub mod ports;
pub mod presentation;
pub mod services;
pub mod utils;

use sqlx::PgPool;
use url::Url;

use crate::services::delegation_service::DelegationService;
use crate::utils::configuration::Configuration;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Configuration,
    pub delegation_service: DelegationService,
}

pub fn database_url() -> anyhow::Result<String> {
    let database_host = std::env::var("TMS_PORTAL_DB_HOST")
        .map_err(|_| anyhow::anyhow!("TMS_PORTAL_DB_HOST must be set"))?;
    let database_port = std::env::var("TMS_PORTAL_DB_PORT").unwrap_or(String::from("5432"));
    let database_name = std::env::var("TMS_PORTAL_DB_NAME").unwrap_or(String::from("tms_db"));
    let database_user = std::env::var("TMS_PORTAL_DB_USER").unwrap_or(String::from("tms"));
    let database_password =
        std::env::var("TMS_PORTAL_DB_PASSWORD").expect("TMS_PORTAL_DB_PASSWORD must be set");

    let database_url_string = format!(
        "postgres://{0}:{1}@{2}:{3}/{4}",
        &database_user, &database_password, &database_host, &database_port, &database_name
    );

    // just parsing the db url to determine if it seems valid.  We actually use database_url_string.
    let _database_url = Url::parse(database_url_string.as_str()).map_err(|e| {
        anyhow::anyhow!(format!(
            "The database url {0} is not valid (error: {1})",
            &database_url_string, e
        ))
    })?;

    Ok(database_url_string)
}
