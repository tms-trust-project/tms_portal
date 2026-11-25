use crate::obj_model::configuration::{
    DelegationPolicyConfig, HttpConfig, JwtConfig, OAuthConfig, RuntimeConfig, StateKey,
};
use crate::obj_model::key::Key;
use crate::infra::persistence::configuration_repository::configuration_dao;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub http_config: HttpConfig,
    pub oauth_config: OAuthConfig,
    pub jwt_config: JwtConfig,
    pub runtime_config: RuntimeConfig,
    pub delegation_policy_config: DelegationPolicyConfig,
    pub state_key: StateKey,
}

impl Configuration {
    pub async fn get(db_pool: &PgPool) -> anyhow::Result<Configuration> {
        let mut tx = db_pool.begin().await?;
        let http_config = configuration_dao::db_get_http_config(&mut tx).await?;
        let oauth_config = configuration_dao::db_get_oauth_config(&mut tx).await?;
        let jwt_config = configuration_dao::db_get_jwt_config(&mut tx).await?;
        let runtime_config = configuration_dao::db_get_runtime_config(&mut tx).await?;
        let delegation_policy_config = configuration_dao::db_get_delegation_policy_config(&mut tx).await?;
        let state_key = configuration_dao::db_get_state_key_id(&mut tx).await?;
        tx.commit().await?;
        
        Ok(Configuration {
            http_config,
            oauth_config,
            jwt_config,
            runtime_config,
            delegation_policy_config,
            state_key
        })
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let http_config =
            HttpConfig { base_url: std::env::var("TMS_PORTAL_BASE_URL")?, ..Default::default() };
        let oauth_config =
            OAuthConfig { login_oauth_provider: std::env::var("TMS_PORTAL_LOGIN_PROVIDER")? };
        let signing_key_kid = std::env::var("TMS_PORTAL_SIGNING_KEY_ID")?;
        let mut jwt_config = JwtConfig {
            key: Key::from_env(
                &signing_key_kid,
                "TMS_PORTAL_SIGNING_KEY_PUB",
                "TMS_PORTAL_SIGNING_KEY_SEC",
            )?,
            signing_key_kid,
            ..Default::default()
        };
        if let Ok(v) = std::env::var("TMS_PORTAL_KEY_EXPIRATION_MINUTES") {
            jwt_config.default_expiration_minutes = v.parse()?
        };
        let runtime_config: RuntimeConfig = Default::default();
        let mut delegation_policy_config: DelegationPolicyConfig = Default::default();
        if let Ok(v) = std::env::var("TMS_PORTAL_DELEGATION_MAX_MINUTES") {
            delegation_policy_config.delegation_max_mins_since_login = v.parse()?
        };
        let state_key_kid = std::env::var("TMS_PORTAL_STATE_KEY_ID")?;
        let state_key = StateKey {
            key: Key::from_env(
                &state_key_kid,
                &std::env::var("TMS_PORTAL_STATE_KEY_PUB")?,
                &std::env::var("TMS_PORTAL_STATE_KEY_SEC")?,
            )?,
            kid: state_key_kid,
        };
        Ok(Configuration {
            http_config,
            oauth_config,
            jwt_config,
            runtime_config,
            delegation_policy_config,
            state_key,
        })
    }
}
