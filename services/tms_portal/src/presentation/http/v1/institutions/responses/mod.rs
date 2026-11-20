use crate::obj_model;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub federated_identity_config: FederatedIdentityConfig,
}

impl From<obj_model::institutions::Institution> for Institution {
    fn from(value: obj_model::institutions::Institution) -> Self {
        Institution {
            id: value.id,
            name: value.name,
            enabled: value.enabled,
            federated_identity_config: value.federated_identity_config.into(),
        }
    }
}

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct FederatedIdentityConfig {
    pub globus: Option<GlobusConfiguration>,
}

impl From<obj_model::institutions::FederatedIdentityConfig> for FederatedIdentityConfig {
    fn from(value: obj_model::institutions::FederatedIdentityConfig) -> Self {
        FederatedIdentityConfig {
            globus: value.globus.map(GlobusConfiguration::from),
        }
    }
}

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct GlobusConfiguration {
    pub redirect_idp_domain: String,
    pub idp_logout_url: String,
}

impl From<obj_model::institutions::GlobusConfiguration> for GlobusConfiguration {
    fn from(value: obj_model::institutions::GlobusConfiguration) -> Self {
        GlobusConfiguration {
            redirect_idp_domain: value.redirect_idp_domain,
            idp_logout_url: value.idp_logout_url,
        }
    }
}
