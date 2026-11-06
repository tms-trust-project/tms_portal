use chrono::{DateTime, Utc};

use crate::{
    obj_model::resources::ResourceProviderLogin,
    ports::resource_provider_login::ResourceProviderLoginRepository,
};

pub struct MockResourceProviderLoginRepo {
    resource_provider_logins: Vec<ResourceProviderLogin>,
    error: Option<String>,
}

impl MockResourceProviderLoginRepo {
    pub fn new(initial_state: Vec<ResourceProviderLogin>) -> Self {
        Self {
            resource_provider_logins: initial_state,
            error: None,
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    fn configured_error(&self) -> Option<anyhow::Error> {
        self.error
            .as_ref()
            .map(|error| anyhow::Error::msg(error.clone()))
    }
}

#[async_trait::async_trait]
impl ResourceProviderLoginRepository for MockResourceProviderLoginRepo {
    async fn find(
        &self,
        tms_identity: &str,
        rp_account: &str,
        rp_id: &str,
        minimum_last_login: DateTime<Utc>,
        enabled: bool,
    ) -> anyhow::Result<Option<ResourceProviderLogin>> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        Ok(self
            .resource_provider_logins
            .iter()
            .find(|login| {
                login.tms_identity == tms_identity
                    && login.rp_account == rp_account
                    && login.rp_id == rp_id
                    && login.last_login >= minimum_last_login
                    && login.enabled == enabled
            })
            .cloned())
    }
}
