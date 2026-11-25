use crate::obj_model::institutions::{IdpConfig, Institution};

pub struct InstitutionBuilder {
    id: Option<String>,
    name: Option<String>,
    enabled: Option<bool>,
    idp_config: Option<IdpConfig>,
}

impl InstitutionBuilder {
    pub fn new() -> Self {
        Self { id: None, name: None, enabled: None, idp_config: None }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn with_idp_config(mut self, idp_config: IdpConfig) -> Self {
        self.idp_config = Some(idp_config);
        self
    }

    pub fn build(&self) -> Institution {
        Institution {
            id: self
                .id
                .clone()
                .unwrap_or_else(|| "test-institution-id".into()),
            name: self
                .name
                .clone()
                .unwrap_or_else(|| "Test Institution".into()),
            enabled: self.enabled.unwrap_or(true),
            idp_config: self
                .idp_config
                .clone()
                .unwrap_or(IdpConfig { globus: None }),
        }
    }
}
