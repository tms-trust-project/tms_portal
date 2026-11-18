use crate::obj_model::oauth::IssuedToken;
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait IssuedTokenRepository: Send + Sync {
    async fn get(&self, token: &str) -> anyhow::Result<IssuedToken>;

    async fn delete(&self, token: &str) -> anyhow::Result<IssuedToken>;

    async fn revoke(&self, token: &str) -> anyhow::Result<IssuedToken>;

    async fn insert(
        &self,
        token: &str,
        expiration: &DateTime<Utc>,
    ) -> anyhow::Result<IssuedToken>;

    async fn cleanup(&self, expired_before: &DateTime<Utc>) -> anyhow::Result<u64>;
}
