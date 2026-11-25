use anyhow::Result;

use crate::obj_model::identity_provider::IdentityProvider;

#[async_trait::async_trait]
pub trait AuthorizationCodeExchangeGateway: Send + Sync {
    async fn exchange(&self, idp: &IdentityProvider, input: ExchangeInput) -> Result<OAuthTokenSet>;
}

pub struct ExchangeInput {
    pub code: String,
    pub callback_url: String,
}

#[derive(Debug)]
pub struct OAuthTokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
}