use jsonwebtoken::jwk::JwkSet;
use anyhow::Result;

#[async_trait::async_trait]
pub trait JwksGateway: Send + Sync {
    async fn get_public_key(&self, pub_key_url: &str) -> Result<JwkSet>;
}