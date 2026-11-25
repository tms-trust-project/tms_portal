mod login_rejection;
pub use login_rejection::LoginRejection;

use crate::infra::auth::globus_token_provider::GlobusTokenProvider;
use crate::obj_model::configuration::{HttpConfig, JwtConfig, StateKey};
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::obj_model::institutions::identity_is_from_domain;
use crate::services::institution_service::InstitutionService;
use crate::obj_model::login::WhoAmIResponse;
use crate::ports::authorization_code_gateway::{AuthorizationCodeExchangeGateway, ExchangeInput};
use crate::ports::identity_porvider::IdentityProviderRepository;
use crate::ports::issued_token::IssuedTokenRepository;
use crate::ports::tms_identity::TmsIdentityRepository;
use crate::ports::token_provider::TokenProvider;
use crate::utils::app_error::AppError;
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{SecurityContext, decode_tms_token, get_tms_token_claims};
use crate::utils::oauth2_authorization_code_utils::{
    LoginCallbackQueryParams, OAuth2State, decode_access_token,
};
use crate::utils::state_utils::decode_state;
use log::warn;
use crate::constants::{DEFAULT_ALGORITHM, TMS_ISSUER};
use anyhow::Result;
use url::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tms_lib::utils::jwt_encoder::JwtEncoderBuilder;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tms_lib::utils::service_error::ServiceError;

#[derive(Clone)]
pub struct LoginService {
    state_key: StateKey,
    http_config: HttpConfig,
    identity_provider_repo: Arc<dyn IdentityProviderRepository>,
    issued_token_repo: Arc<dyn IssuedTokenRepository>,
    tms_identity_repo: Arc<dyn TmsIdentityRepository>,
    jwt_config: JwtConfig,
    authorization_code_exchange_gateway: Arc<dyn AuthorizationCodeExchangeGateway>,
    institution_service: InstitutionService,
}

impl LoginService {
    pub fn new(
        state_key: StateKey,
        http_config: HttpConfig,
        identity_provider_repo: Arc<dyn IdentityProviderRepository>,
        issued_token_repo: Arc<dyn IssuedTokenRepository>,
        tms_identity_repo: Arc<dyn TmsIdentityRepository>,
        jwt_config: JwtConfig,
        authorization_code_exchange_gateway: Arc<dyn AuthorizationCodeExchangeGateway>,
        institution_service: InstitutionService,
    ) -> Self {
        Self {
            state_key,
            http_config,
            identity_provider_repo,
            issued_token_repo,
            tms_identity_repo,
            jwt_config,
            authorization_code_exchange_gateway,
            institution_service,
        }
    }

    pub async fn list_identity_providers(&self) -> Result<HashSet<IdentityProvider>> {
        self.identity_provider_repo.list_login_providers().await
    }

    pub async fn whoami(&self, ctx: &SecurityContext) -> anyhow::Result<WhoAmIResponse, AppError> {
        let tms_token_claims = decode_tms_token(&self.jwt_config, &ctx.token).await?;

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

    /*
    The login identity provider's redirect back to /login/callback.  Every way the login
    can be refused (a bad or expired state, an error or cancel at the identity provider, a
    code that can't be exchanged, an identity that isn't the institution's) is a
    LoginRejection; an Err is something unexpected (e.g. the database).
     */
    pub async fn handle_callback(
        &self,
        configuration: &Configuration,
        query: &LoginCallbackQueryParams,
        cookie_state: Option<&str>,
    ) -> Result<CallbackResult> {
        let (decoded_state, code) =
            match check_login_callback(&self.state_key, query, cookie_state).await {
                Ok(valid) => valid,
                Err((_, rejection)) => return Ok(CallbackResult::Rejected(rejection)),
            };

        let idp = self
            .identity_provider_repo
            .get_login_provider_by_id(&decoded_state.idp_id)
            .await?;

        let exchanged = async {
            let oauth_token_set = self
                .authorization_code_exchange_gateway
                .exchange(
                    &idp,
                    ExchangeInput {
                        code,
                        callback_url: self.http_config.get_identity_provider_callback_url(),
                    },
                )
                .await?;
            decode_access_token::<HashMap<String, Value>>(&idp, &oauth_token_set.id_token).await
        }
        .await;
        let mut claims = match exchanged {
            Ok(claims) => claims,
            Err(error) => {
                warn!("Unable to get the login identity from {0}: {error:#}", idp.id);
                return Ok(CallbackResult::Rejected(LoginRejection::LoginFailed));
            }
        };

        // the login must come back as one of the institution's identities, freshly
        // authenticated, before any TMS token is issued.  The institution is looked up
        // again, so one disabled since the login started can't be used.
        if let Err(rejection) = self.check_institution_login(&decoded_state, &claims).await? {
            return Ok(CallbackResult::Rejected(rejection));
        }

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

        let jwt_config = &configuration.jwt_config;
        let kid = &jwt_config.signing_key_kid;
        let keys = &jwt_config.key;

        // TODO: add kid, alg, and jti in header ... maybe other stuff?
        let tms_token_string = JwtEncoderBuilder::builder(
            tms_token_claims.clone(),
            keys.jwt_private_key.as_bytes(),
            DEFAULT_ALGORITHM,
            kid,
        )
        .encode()
        .await?;

        let _issued_token = self
            .issued_token_repo
            .insert(&tms_token_string, &tms_token_claims.get_expires_at_dt()?)
            .await?;

        Ok(CallbackResult::LoggedIn {
            token: tms_token_string,
            expires_in: tms_token_claims.get_expires_in()?,
            redirect_uri: decoded_state.redirect_uri,
        })
    }
}

impl LoginService {
    /*
    Checks a login against the institution it went through (from the signed state): the
    institution must still be usable, and the identity must be one of its identities,
    authenticated for this login.  Every login has an institution; a state without one is
    refused.
     */
    pub async fn check_institution_login(
        &self,
        state: &OAuth2State,
        claims: &HashMap<String, Value>,
    ) -> Result<std::result::Result<(), LoginRejection>> {
        let Some(institution_id) = &state.institution_id else {
            warn!("Login callback for a login without an institution");
            return Ok(Err(LoginRejection::InstitutionRequired));
        };
        let domain = match self.institution_service.globus_login_domain(institution_id).await {
            Ok(domain) => domain,
            Err(error) if is_bad_request(&error) => {
                warn!("Login callback for an unusable institution: {error:#}");
                return Ok(Err(LoginRejection::UnapprovedInstitution));
            }
            Err(error) => return Err(error),
        };
        let checked = check_login_identity(claims, &domain, state.login_started);
        if let Err(rejection) = &checked {
            warn!(
                "Login through {institution_id} refused ({0}) for {1:?}",
                rejection.error_code(),
                claims.get("preferred_username")
            );
        }
        Ok(checked)
    }
}

pub enum CallbackResult {
    // the TMS token, its lifetime in seconds, and where to send the browser
    LoggedIn {
        token: String,
        expires_in: i64,
        redirect_uri: String,
    },
    Rejected(LoginRejection),
}

/*
Check what the login identity provider sent back, before the code is exchanged.  The
state must be the one in this browser's state cookie, and still valid (signed, and it
expires 5 minutes after the login started).  The identity provider may send an error
(RFC 6749 4.1.2.1) instead of a code.  A rejection comes with the decoded state when it
was valid, so an OAuth2 login can report it to its client.
 */
pub async fn check_login_callback(
    state_key: &StateKey,
    query: &LoginCallbackQueryParams,
    cookie_state: Option<&str>,
) -> std::result::Result<(OAuth2State, String), (Option<OAuth2State>, LoginRejection)> {
    let (Some(state), Some(cookie_state)) = (query.state.as_deref(), cookie_state) else {
        warn!("Login callback without a state, or without a state cookie");
        return Err((None, LoginRejection::LoginExpired));
    };
    if state != cookie_state {
        warn!("Login callback state does not match the state cookie");
        return Err((None, LoginRejection::LoginExpired));
    }
    let decoded_state: OAuth2State = match decode_state(state_key, state).await {
        Ok(decoded_state) => decoded_state,
        Err(error) => {
            warn!("Login callback state is not valid: {error:#}");
            return Err((None, LoginRejection::LoginExpired));
        }
    };
    if let Some(error) = &query.error {
        warn!(
            "The login identity provider sent back an error: {error} {0}",
            query.error_description.as_deref().unwrap_or_default()
        );
        let rejection = if error == "access_denied" {
            LoginRejection::LoginCancelled
        } else {
            LoginRejection::LoginFailed
        };
        return Err((Some(decoded_state), rejection));
    }
    match &query.code {
        Some(code) => Ok((decoded_state, code.clone())),
        None => {
            warn!("Login callback without a code or an error");
            Err((Some(decoded_state), LoginRejection::LoginFailed))
        }
    }
}

/*
The portal's sign in page (its home page), with ?error=<code> when a login was refused,
so the page can say why.
 */
pub fn get_signin_url(http_config: &HttpConfig, rejection: Option<LoginRejection>) -> String {
    let signin_url = http_config.make_tms_url(&http_config.base_url, "/");
    match rejection {
        Some(rejection) => match Url::parse(&signin_url) {
            Ok(mut url) => {
                url.query_pairs_mut().append_pair("error", rejection.error_code());
                url.to_string()
            }
            Err(_) => signin_url,
        },
        None => signin_url,
    }
}

// whether the error is a ServiceError::BadRequest (e.g. an unusable institution)
fn is_bad_request(error: &anyhow::Error) -> bool {
    matches!(error.downcast_ref::<ServiceError>(), Some(ServiceError::BadRequest(_)))
}

// how far the identity provider's clock may be behind ours
const AUTHENTICATION_CLOCK_SKEW_SECS: f64 = 5.0;

/*
The login identity provider's authorize url, which the browser is redirected to.  For
Globus:
- prompt=login makes Globus authenticate the user again, even with a Globus session
  (a user can drop it from the url, so check_login_identity checks last_authentication)
- redirect_idp_domain sends a user not yet signed in to Globus straight to the
  institution's IdP, and session_required_single_domain makes Globus require an identity
  from that domain
 */
pub fn get_login_authorize_url(
    idp: &IdentityProvider,
    callback_url: &str,
    encoded_state: &str,
    nonce: &str,
    institution_domain: &str,
) -> Result<Url> {
    let mut query_params = vec![
        ("response_type", "code"),
        ("client_id", idp.client_id.as_str()),
        ("redirect_uri", callback_url),
        ("state", encoded_state),
        ("nonce", nonce),
        ("access_type", "offline"),
    ];
    if let Some(scope) = &idp.scope {
        query_params.push(("scope", scope.as_str()))
    }
    if idp.identity_provider_type == IdentityProviderType::Globus {
        query_params.push(("prompt", "login"));
        query_params.push(("redirect_idp_domain", institution_domain));
        query_params.push(("session_required_single_domain", institution_domain));
    }
    Ok(Url::parse_with_params(&idp.identity_redirect_url, query_params)?)
}

/*
Checks the identity a login through an institution came back with, before a TMS token is
issued:
- Its username (e.g. jdoe@utexas.edu) must be from the institution's domain.
  redirect_idp_domain only steers the login, and a user can edit the authorize url.
- It must have authenticated after the login started (last_authentication, epoch
  seconds), as prompt=login asks, rather than come from an earlier Globus session.
Globus's top-level preferred_username and last_authentication are for the account's
primary identity.  With the view_identity_set scope the id token also has identity_set,
every identity linked to the account, each with its own username and
last_authentication; then any of them can be the institution's.  That way a user whose
primary identity is, say, ORCID can still log in with their institution identity.
 */
pub fn check_login_identity(
    claims: &HashMap<String, Value>,
    domain: &str,
    login_started: u64,
) -> std::result::Result<(), LoginRejection> {
    // the primary identity's claims are at the top level of the id token
    let mut candidates = vec![LoginIdentity::from_claims(
        claims.get("preferred_username"),
        claims.get("last_authentication"),
    )];
    if let Some(identity_set) = claims.get("identity_set").and_then(Value::as_array) {
        candidates.extend(identity_set.iter().map(|identity| {
            let last_authentication = identity.get("last_authentication");
            LoginIdentity::from_claims(identity.get("username"), last_authentication)
        }));
    }

    let from_domain: Vec<&LoginIdentity> = candidates
        .iter()
        .filter(|identity| {
            identity.username.is_some_and(|name| identity_is_from_domain(name, domain))
        })
        .collect();
    if from_domain.is_empty() {
        return Err(LoginRejection::UnapprovedInstitution);
    }
    let fresh = |identity: &&LoginIdentity| {
        identity
            .last_authentication
            .is_some_and(|last| last + AUTHENTICATION_CLOCK_SKEW_SECS >= login_started as f64)
    };
    if !from_domain.iter().any(fresh) {
        return Err(LoginRejection::StaleAuthentication);
    }
    Ok(())
}

// one of the account's identities, as the id token describes it
struct LoginIdentity<'a> {
    username: Option<&'a str>,
    last_authentication: Option<f64>,
}

impl<'a> LoginIdentity<'a> {
    fn from_claims(username: Option<&'a Value>, last_authentication: Option<&Value>) -> Self {
        Self {
            username: username.and_then(Value::as_str),
            last_authentication: last_authentication.and_then(Value::as_f64),
        }
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

#[cfg(test)]
#[path = "login_service.test.rs"]
mod login_service_test;
