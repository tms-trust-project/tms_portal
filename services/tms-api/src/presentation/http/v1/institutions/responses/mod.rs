use crate::obj_model;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub idp_config: IdpConfig,
}

impl From<obj_model::institutions::Institution> for Institution {
    fn from(value: obj_model::institutions::Institution) -> Self {
        Institution {
            id: value.id,
            name: value.name,
            enabled: value.enabled,
            idp_config: value.idp_config.into(),
        }
    }
}

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct IdpConfig {
    pub globus: Option<GlobusConfiguration>,
}

impl From<obj_model::institutions::IdpConfig> for IdpConfig {
    fn from(value: obj_model::institutions::IdpConfig) -> Self {
        IdpConfig { globus: value.globus.map(GlobusConfiguration::from) }
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
