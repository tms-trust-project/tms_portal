pub mod config;
pub mod infra;
pub mod obj_model;
pub mod ports;
pub mod presentation;
pub mod services;
pub mod utils;
pub mod constants;

use axum::extract::FromRef;
use services::client_service::ClientService;
use services::institution_service::InstitutionService;
use services::login_service::LoginService;
use sqlx::PgPool;
use url::Url;

use crate::services::delegation_service::DelegationService;
use crate::utils::configuration::Configuration;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Configuration,
    pub client_service: ClientService,
    pub delegation_service: DelegationService,
    pub institution_service: InstitutionService,
    pub login_service: LoginService,
}

impl FromRef<AppState> for DelegationService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.delegation_service.clone()
    }
}

pub fn database_url() -> anyhow::Result<String> {
    let database_host = std::env::var("TMS_PORTAL_DB_HOST")
        .map_err(|_| anyhow::anyhow!("TMS_PORTAL_DB_HOST must be set"))?;
    let database_port = std::env::var("TMS_PORTAL_DB_PORT").unwrap_or(String::from("5432"));
    let database_name = std::env::var("TMS_PORTAL_DB_NAME").unwrap_or(String::from("tms_db"));
    let database_user = std::env::var("TMS_PORTAL_DB_USER").unwrap_or(String::from("tms"));
    let database_password =
        std::env::var("TMS_PORTAL_DB_PASSWORD").expect("TMS_PORTAL_DB_PASSWORD must be set");

    build_database_url(
        &database_user,
        &database_password,
        &database_host,
        &database_port,
        &database_name,
    )
}

/*
The Postgres connection url.  It holds the password, so an error about it shows the url
with the password masked (it ends up in the startup output and the logs).
 */
fn build_database_url(
    user: &str,
    password: &str,
    host: &str,
    port: &str,
    name: &str,
) -> anyhow::Result<String> {
    let database_url_string = format!("postgres://{user}:{password}@{host}:{port}/{name}");

    // just parsing the db url to determine if it seems valid.  We actually use database_url_string.
    let _database_url = Url::parse(database_url_string.as_str()).map_err(|e| {
        anyhow::anyhow!(
            "The database url postgres://{user}:***@{host}:{port}/{name} is not valid (error: {e})"
        )
    })?;

    Ok(database_url_string)
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod lib_test;
