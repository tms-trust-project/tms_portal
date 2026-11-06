use chrono::{DateTime, Utc};

use crate::{
    obj_model::delegation::{Delegation, test_fixtures::DelegationBuilder},
    ports::delegation::DelegationRepository,
};

pub struct MockDelegationRepo {
    return_type: MockDelegationReturnType,
}

impl MockDelegationRepo {
    pub fn new(return_type: MockDelegationReturnType) -> Self {
        Self { return_type }
    }
}

pub enum MockDelegationReturnType {
    Error,
    Delegation,
}

impl MockDelegationReturnType {
    pub fn delegation(
        client_id: Option<&str>,
        rp_account: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
        tms_identity: Option<&str>,
        rp_id: Option<&str>,
        delegation_id: Option<i32>,
    ) -> Delegation {
        let mut builder = DelegationBuilder::new();

        if let Some(client_id) = client_id {
            builder = builder.with_client_id(client_id.into());
        }

        if let Some(rp_account) = rp_account {
            builder = builder.with_rp_account(rp_account.into());
        }

        if let Some(expires_at) = expires_at {
            builder = builder.with_expires_at(expires_at);
        }

        if let Some(tms_identity) = tms_identity {
            builder = builder.with_tms_identity(tms_identity.into());
        }

        if let Some(rp_id) = rp_id {
            builder = builder.with_rp_id(rp_id.into());
        }

        if let Some(delegation_id) = delegation_id {
            builder = builder.with_id(delegation_id);
        }

        builder.build()
    }

    pub fn error(msg: Option<&str>) -> anyhow::Error {
        anyhow::Error::msg(msg.unwrap_or("Error").to_string())
    }
}

#[async_trait::async_trait]
impl DelegationRepository for MockDelegationRepo {
    async fn save(
        &self,
        client_id: &str,
        rp_account: &str,
        expires_at: &DateTime<Utc>,
        tms_identity: &str,
        rp_id: &str,
    ) -> anyhow::Result<Delegation> {
        match self.return_type {
            MockDelegationReturnType::Delegation => Ok(MockDelegationReturnType::delegation(
                Some(client_id),
                Some(rp_account),
                Some(*expires_at),
                Some(tms_identity),
                Some(rp_id),
                None,
            )),
            MockDelegationReturnType::Error => Err(MockDelegationReturnType::error(Some(
                "Failed to save delegation",
            ))),
        }
    }

    async fn list_all(&self, tms_identity: &str) -> anyhow::Result<Vec<Delegation>> {
        match self.return_type {
            MockDelegationReturnType::Delegation => Ok(vec![MockDelegationReturnType::delegation(
                None,
                None,
                None,
                Some(tms_identity),
                None,
                None,
            )]),
            MockDelegationReturnType::Error => Err(MockDelegationReturnType::error(Some(
                "Failed to list delegations",
            ))),
        }
    }

    async fn list_by_client_id(
        &self,
        client_id: &str,
        tms_identity: &str,
    ) -> anyhow::Result<Vec<Delegation>> {
        match self.return_type {
            MockDelegationReturnType::Delegation => Ok(vec![MockDelegationReturnType::delegation(
                Some(client_id),
                None,
                None,
                Some(tms_identity),
                None,
                None,
            )]),
            MockDelegationReturnType::Error => Err(MockDelegationReturnType::error(Some(
                "Failed to list delegations by client id",
            ))),
        }
    }

    async fn delete(
        &self,
        tms_identity: &str,
        client_id: &str,
        delegation_id: i32,
    ) -> anyhow::Result<Delegation> {
        match self.return_type {
            MockDelegationReturnType::Delegation => Ok(MockDelegationReturnType::delegation(
                Some(client_id),
                None,
                None,
                Some(tms_identity),
                None,
                Some(delegation_id),
            )),
            MockDelegationReturnType::Error => Err(MockDelegationReturnType::error(Some(
                "Failed to delete delegation",
            ))),
        }
    }
}
