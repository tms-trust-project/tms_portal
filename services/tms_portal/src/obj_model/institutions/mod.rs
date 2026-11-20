#[cfg(test)]
pub mod test_fixtures;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub federated_identity_config: FederatedIdentityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FederatedIdentityConfig {
    pub globus: Option<GlobusConfiguration>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobusConfiguration {
    pub redirect_idp_domain: String,
    pub idp_logout_url: String,
}
