use chrono::{DateTime, Utc};

use crate::obj_model::delegation::Delegation;

#[async_trait::async_trait]
pub trait DelegationRepository: Send + Sync {
    async fn save(
        &self,
        client_id: &str,
        rp_account: &str,
        expires_at: &DateTime<Utc>,
        tms_identity: &str,
        rp_id: &str,
    ) -> anyhow::Result<Delegation>;

    async fn list_all(&self, tms_identity: &str) -> anyhow::Result<Vec<Delegation>>;

    async fn list_by_client_id(&self, client_id: &str, tms_identity: &str) -> anyhow::Result<Vec<Delegation>>;

    async fn delete(&self, tms_identity: &str, client_id: &str, delegation_id: i32) -> anyhow::Result<Delegation>;
}
