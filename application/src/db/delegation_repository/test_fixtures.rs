use std::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::{
    obj_model::delegation::{Delegation, test_fixtures::DelegationBuilder},
    ports::delegation::DelegationRepository,
};

pub struct MockDelegationRepo {
    delegations: Mutex<Vec<Delegation>>,
    error: Option<String>,
}

impl MockDelegationRepo {
    pub fn new(initial_state: Vec<Delegation>) -> Self {
        Self { delegations: Mutex::new(initial_state), error: None }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    fn configured_error(&self) -> Option<anyhow::Error> {
        self.error.as_ref().map(|error| anyhow::Error::msg(error.clone()))
    }

    fn lock_delegations(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Vec<Delegation>>> {
        self.delegations.lock().map_err(|_| anyhow::Error::msg("Mock delegation repository lock poisoned"))
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
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        let mut delegations = self.lock_delegations()?;
        let next_id = delegations.iter().map(|delegation| delegation.id).max().unwrap_or(0) + 1;
        let delegation = DelegationBuilder::new()
            .with_id(next_id)
            .with_client_id(client_id.into())
            .with_rp_account(rp_account.into())
            .with_expires_at(*expires_at)
            .with_tms_identity(tms_identity.into())
            .with_rp_id(rp_id.into())
            .build();

        delegations.push(delegation.clone());

        Ok(delegation)
    }

    async fn list_all(&self, tms_identity: &str) -> anyhow::Result<Vec<Delegation>> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        let delegations = self.lock_delegations()?;

        Ok(delegations.iter().filter(|delegation| delegation.tms_identity == tms_identity).cloned().collect())
    }

    async fn list_by_client_id(&self, client_id: &str, tms_identity: &str) -> anyhow::Result<Vec<Delegation>> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        let delegations = self.lock_delegations()?;

        Ok(delegations
            .iter()
            .filter(|delegation| delegation.client_id == client_id && delegation.tms_identity == tms_identity)
            .cloned()
            .collect())
    }

    async fn delete(&self, tms_identity: &str, client_id: &str, delegation_id: i32) -> anyhow::Result<Delegation> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        let mut delegations = self.lock_delegations()?;
        let index = delegations
            .iter()
            .position(|delegation| {
                delegation.id == delegation_id
                    && delegation.client_id == client_id
                    && delegation.tms_identity == tms_identity
            })
            .ok_or_else(|| anyhow::Error::msg("Delegation not found"))?;

        Ok(delegations.remove(index))
    }
}
