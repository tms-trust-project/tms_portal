use chrono::{DateTime, Utc};

use crate::{
    obj_model::resources::{ResourceProviderLogin, test_fixtures::ResourceProviderLoginBuilder},
    ports::resource_provider_login::ResourceProviderLoginRepository,
};

pub struct MockResourceProviderLoginRepo {
    return_type: MockResourceProviderLoginReturnType,
}

impl MockResourceProviderLoginRepo {
    pub fn new(return_type: MockResourceProviderLoginReturnType) -> Self {
        Self { return_type }
    }
}

pub enum MockResourceProviderLoginReturnType {
    Error,
    None,
    ResourceProviderLogin,
}

impl MockResourceProviderLoginReturnType {
    pub fn resource_provider_login(
        tms_identity: Option<&str>,
        rp_account: Option<&str>,
        rp_id: Option<&str>,
        last_login: Option<DateTime<Utc>>,
        enabled: Option<bool>,
    ) -> ResourceProviderLogin {
        let mut builder = ResourceProviderLoginBuilder::new();

        if let Some(tms_identity) = tms_identity {
            builder = builder.with_tms_identity(tms_identity.into());
        }

        if let Some(rp_account) = rp_account {
            builder = builder.with_rp_account(rp_account.into());
        }

        if let Some(rp_id) = rp_id {
            builder = builder.with_rp_id(rp_id.into());
        }

        if let Some(last_login) = last_login {
            builder = builder.with_last_login(last_login);
        }

        if let Some(enabled) = enabled {
            builder = builder.with_enabled(enabled);
        }

        builder.build()
    }

    pub fn error(msg: Option<&str>) -> anyhow::Error {
        anyhow::Error::msg(msg.unwrap_or("Error").to_string())
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
        match self.return_type {
            MockResourceProviderLoginReturnType::ResourceProviderLogin => Ok(Some(
                MockResourceProviderLoginReturnType::resource_provider_login(
                    Some(tms_identity),
                    Some(rp_account),
                    Some(rp_id),
                    Some(minimum_last_login),
                    Some(enabled),
                ),
            )),
            MockResourceProviderLoginReturnType::None => Ok(None),
            MockResourceProviderLoginReturnType::Error => {
                Err(MockResourceProviderLoginReturnType::error(Some(
                    "Failed to find resource provider login",
                )))
            }
        }
    }
}
