use log::debug;
use reqwest::Client;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::obj_model::identity_provider::IdentityProvider;
use crate::ports::authorization_code_gateway::{AuthorizationCodeExchangeGateway, ExchangeInput, OAuthTokenSet};

pub struct ReqwestAuthorizationCodeExchangeGateway {
    http_client: Client,
}

#[async_trait::async_trait]
impl AuthorizationCodeExchangeGateway for ReqwestAuthorizationCodeExchangeGateway {
    async fn exchange(&self, idp: &IdentityProvider, input: ExchangeInput) -> Result<OAuthTokenSet> {
        let resp = self.http_client
            .post(&idp.oauth2_token_url)
            .form(&[
                ("grant_type", &"authorization_code".to_string()),
                ("redirect_uri", &input.callback_url),
                ("code", &input.code),
            ])
            .basic_auth(idp.client_id.clone(), Some(idp.client_secret.clone()))
            .send()
            .await
            .context("Error getting response body")?
            .text()
            .await
            .context("Error getting response body")?;

        Ok(serde_json::from_str::<AuthorizationCodeResponse>(&resp)
            .context("Error deserializing token response body")?
            .into())
    }
}

impl ReqwestAuthorizationCodeExchangeGateway {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new()
        }
    }
}

impl From<AuthorizationCodeResponse> for OAuthTokenSet {
    fn from(value: AuthorizationCodeResponse) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            id_token: value.id_token,
            token_type: value.token_type,
            expires_in: value.expires_in,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
}

/*
Exchanges an auth code for an auth token.  The parameter <R> is the structure
that it is deserialized into.
 */
pub async fn get_token_for_provider<R>(
    idp: &IdentityProvider,
    callback_url: &str,
    code: &str,
) -> Result<R>
where
    R: DeserializeOwned,
{
    debug!("exchange_code_for_token called");
    let form_params = [
        ("grant_type", &"authorization_code".to_string()),
        ("redirect_uri", &String::from(callback_url)),
        ("code", &code.to_owned()),
    ];
    debug!("Form params: {:?}", form_params);
    let client = reqwest::Client::new();
    let response = client
        .post(&idp.oauth2_token_url)
        .form(&form_params)
        .basic_auth(idp.client_id.clone(), Some(idp.client_secret.clone()))
        .send()
        .await
        .context("Error getting response body")?;
    
    debug!("Response from exchange code: {:?}", response);

    let token_string = response
        .text()
        .await
        .context("Error getting response body")?;

    serde_json::from_str::<R>(&token_string).context("Error deserializing token response body")
}