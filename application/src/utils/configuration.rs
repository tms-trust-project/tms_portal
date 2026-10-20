use crate::obj_model::configuration::{
    DelegationPolicyConfig, HttpConfig, JwtConfig, OAuthConfig, RuntimeConfig, StateKey,
};
use crate::obj_model::keys::Key;
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
    #[allow(dead_code)]
    pub async fn get(_db_pool: &PgPool) -> anyhow::Result<Configuration> {
        unreachable!()
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let http_config = HttpConfig {
            base_url: std::env::var("TMS_PORTAL_BASE_URL")?,
            ..Default::default()
        };
        let oauth_config = OAuthConfig {
            login_oauth_provider: std::env::var("TMS_PORTAL_LOGIN_PROVIDER")?,
        };
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
