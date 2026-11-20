use crate::infra::auth::globus_token_provider::GlobusTokenProvider;
use crate::obj_model::configuration::{HttpConfig, StateKey};
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::obj_model::login::WhoAmIResponse;
use crate::ports::authorization_code_gateway::{AuthorizationCodeExchangeGateway, ExchangeInput};
use crate::ports::identity_porvider::IdentityProviderRepository;
use crate::ports::issued_token::IssuedTokenRepository;
use crate::ports::key::KeyRepository;
use crate::ports::tms_identity::TmsIdentityRepository;
use crate::ports::token_provider::TokenProvider;
use crate::utils::app_error::AppError;
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{SecurityContext, TmsTokenClaims, get_tms_token_claims};
use crate::utils::oauth2_authorization_code_utils::{
    OAuth2State, decode_access_token,
};
use crate::constants::{DEFAULT_ALGORITHM, TMS_ISSUER};
use anyhow::{Context, Result};
use jsonwebtoken::decode_header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tms_lib::utils::jwt_encoder::JwtEncoderBuilder;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use tms_lib::utils::service_error::{
    ServiceError,
    ServiceError::{BadRequest, Unauthorized},
};

#[derive(Clone)]
pub struct LoginService {
    state_key: StateKey,
    http_config: HttpConfig,
    identity_provider_repo: Arc<dyn IdentityProviderRepository>,
    issued_token_repo: Arc<dyn IssuedTokenRepository>,
    tms_identity_repo: Arc<dyn TmsIdentityRepository>,
    key_repo: Arc<dyn KeyRepository>,
    authorization_code_exchange_gateway: Arc<dyn AuthorizationCodeExchangeGateway>,
}

impl LoginService {
    pub fn new(
        state_key: StateKey,
        http_config: HttpConfig,
        identity_provider_repo: Arc<dyn IdentityProviderRepository>,
        issued_token_repo: Arc<dyn IssuedTokenRepository>,
        tms_identity_repo: Arc<dyn TmsIdentityRepository>,
        key_repo: Arc<dyn KeyRepository>,
        authorization_code_exchange_gateway: Arc<dyn AuthorizationCodeExchangeGateway>,
    ) -> Self {
        Self {
            state_key,
            http_config,
            identity_provider_repo,
            issued_token_repo,
            tms_identity_repo,
            key_repo,
            authorization_code_exchange_gateway,
        }
    }

    pub async fn list_identity_providers(&self) -> Result<HashSet<IdentityProvider>> {
        self.identity_provider_repo.list_login_providers().await
    }

    pub async fn whoami(&self, ctx: &SecurityContext) -> anyhow::Result<WhoAmIResponse, AppError> {
        let token_header = decode_header(&ctx.token)?;
        
        let key = match token_header.kid {
            Some(kid) => self.key_repo.get_by_id(&kid).await,
            None => return Err(BadRequest(String::from("Unable to find key for jwt")).into()),
        }?;
    
        let tms_token_claims: TmsTokenClaims = JwtDecoderBuilder::builder()
            .public_key(key.jwt_public_key.as_bytes())
            .decode(&ctx.token)
            .await
            .context("Error decoding JWT")?;
    
        let idp_provider = &tms_token_claims.tms_idp_provider;
        let token_provider = match idp_provider {
            IdentityProviderType::Globus => GlobusTokenProvider {},
            _ => {
                return Err(ServiceError::Internal(format!(
                    "Unsupported provider type {0}",
                    idp_provider
                ))
                .into());
            }
        };
    
        token_provider.whoami(&tms_token_claims)
    }

    // Logout and return the redirct url
    pub async fn logout(&self, ctx: &SecurityContext) -> Result<RedirectUrl> {
        self.issued_token_repo.revoke(&ctx.token).await?;
        Ok(RedirectUrl(self.http_config.base_url.clone()))
    }

    pub async fn handle_callback(
        &self,
        configuration: &Configuration,
        state: &str,
        code: &str,
        cookie_state: &str,
    ) -> Result<(String, i64)> {
        if !cookie_state.eq(state) {
            return Err(Unauthorized("State cookies do not match".to_string()).into());
        }

        let keys = &self.state_key.key;
        // let decoding_key = Some(DecodingKey::from_rsa_pem(&keys.jwt_public_key.as_bytes())?);

        let decoded_state = JwtDecoderBuilder::builder()
            .public_key(keys.jwt_public_key.as_bytes())
            .validate_aud(false)
            .decode::<OAuth2State>(state.into())
            .await?;
        
        let idp = self.identity_provider_repo
            .get_login_provider_by_id(&decoded_state.idp_id)
            .await?;
        
        let oauth_token_set = self.authorization_code_exchange_gateway
            .exchange(
                &idp,
                ExchangeInput {
                    code: code.into(),
                    callback_url: self.http_config
                        .get_identity_provider_callback_url().clone(),
                }
            )
            .await?;
    
        let mut claims: HashMap<String, Value> = decode_access_token(&idp, &oauth_token_set.id_token).await?;

        // TODO: get iss from config
        claims.insert("iss".to_string(), Value::from(TMS_ISSUER));
    
        let tms_token_claims = get_tms_token_claims(
            configuration,
            &decoded_state.client_id,
            &idp.id,
            &idp.identity_provider_type,
            &claims,
        )
        .await?;
    
        // make sure the tms_identity is stored in the db
        let sub = tms_token_claims.get_sub()?;
        let _tms_identity = self.tms_identity_repo.insert_if_absent(&sub).await?;

        // let http_config = db_get_http_config(&mut tx).await?;
        let jwt_config = &configuration.jwt_config;
        let kid = &jwt_config.signing_key_kid;
        let keys = &jwt_config.key;
        // let tms_token_claims = get_tms_token_claims(&http_config,
        //         &jwt_config, &client_id, &idp_id, &idp_type, &claims).await?;

        // TODO: add kid, alg, and jti in header ... maybe other stuff?
        let tms_token_string = JwtEncoderBuilder::builder(
            tms_token_claims.clone(),
            keys.jwt_private_key.as_bytes(),
            DEFAULT_ALGORITHM,
            kid,
        )
            .encode()
            .await?;

        let _issued_token = self.issued_token_repo
            .insert(&tms_token_string, &tms_token_claims.get_expires_at_dt()?)
            .await?;

        Ok((tms_token_string, tms_token_claims.get_expires_in()?))
    }
}

pub struct RedirectUrl(String);

impl RedirectUrl {
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
}
