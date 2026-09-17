use std::sync::Arc;
use std::time::Duration;
use log::error;
use sqlx::PgPool;
use tokio::time::{interval_at, Instant};
use crate::db::config_dao::{db_get_delegation_policy_config, db_get_http_config, db_get_jwt_config, db_get_oauth_config, db_get_runtime_config};
use crate::obj_model::configuration::{DelegationPolicyConfig, HttpConfig, JwtConfig, OAuthConfig, RuntimeConfig};

/// Read handle for the periodically-refreshed configuration cache. Always holds the
/// last-successfully-fetched `Configuration`; cheap to clone (Arc) and to hand to callers.
pub type ConfigCache = tokio::sync::watch::Receiver<Arc<Configuration>>;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub http_config: HttpConfig,
    pub oauth_config: OAuthConfig,
    pub jwt_config: JwtConfig,
    pub runtime_config: RuntimeConfig,
    pub delegation_policy_config: DelegationPolicyConfig,
}

impl Configuration {
    pub async fn get(db_pool:&PgPool) -> anyhow::Result<Configuration> {
        let mut tx = db_pool.begin().await?;
        let http_config = db_get_http_config(&mut tx).await?;
        let oauth_config = db_get_oauth_config(&mut tx).await?;
        let jwt_config = db_get_jwt_config(&mut tx).await?;
        let runtime_config = db_get_runtime_config(&mut tx).await?;
        let delegation_policy_config = db_get_delegation_policy_config(&mut tx).await?;
        tx.commit().await?;
        Ok(Configuration {
            http_config,
            oauth_config,
            jwt_config,
            runtime_config,
            delegation_policy_config,
        })
    }
}

// Fetch the configuration once, then spawn a background task that refetches it every
// `refresh_interval` and publishes the new value to the returned `ConfigCache`. If a refresh
// fetch fails, the error is logged and the previously cached value keeps being served.
pub async fn spawn_refresh_task(db_pool: PgPool, refresh_interval: Duration) -> anyhow::Result<ConfigCache> {
    let initial_config = Configuration::get(&db_pool).await?;
    let (sender, receiver) = tokio::sync::watch::channel(Arc::new(initial_config));

    tokio::spawn(async move {
        let mut interval = interval_at(Instant::now() + refresh_interval, refresh_interval);
        loop {
            interval.tick().await;
            match Configuration::get(&db_pool).await {
                Ok(new_config) => {
                    let _ = sender.send(Arc::new(new_config));
                }
                Err(error) => {
                    error!("Failed to refresh configuration, keeping previous value: {}", error);
                }
            }
        }
    });

    Ok(receiver)
}