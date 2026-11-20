use crate::obj_model::institutions::{FederatedIdentityConfig, Institution};

pub struct InstitutionBuilder {
    id: Option<String>,
    name: Option<String>,
    enabled: Option<bool>,
    federated_identity_config: Option<FederatedIdentityConfig>,
}

impl InstitutionBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            enabled: None,
            federated_identity_config: None,
        }
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

    pub fn with_federated_identity_config(
        mut self,
        federated_identity_config: FederatedIdentityConfig,
    ) -> Self {
        self.federated_identity_config = Some(federated_identity_config);
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
            federated_identity_config: self
                .federated_identity_config
                .clone()
                .unwrap_or(FederatedIdentityConfig { globus: None }),
        }
    }
}
