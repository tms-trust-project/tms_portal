use jsonwebtoken::jwk::JwkSet;
use reqwest::Client;
use anyhow::Result;

use crate::ports::jwks_gateway::JwksGateway;

pub struct ReqwestJwksGateway {
    http_client: Client
}

#[async_trait::async_trait]
impl JwksGateway for ReqwestJwksGateway {
    async fn get_public_key(&self, pub_key_url: &str) -> Result<JwkSet> {
        let jwks_string = self.http_client
            .get(pub_key_url)
            .send()
            .await?
            .text()
            .await?;
        
        serde_json::from_str(jwks_string.as_str()).map_err(|error| error.into())
    }
}

impl ReqwestJwksGateway {
    pub fn new() -> Self {
        Self {
            http_client: Client::new()
        }
    }
}