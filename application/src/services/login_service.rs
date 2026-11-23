use crate::db::identity_provider_dao::{db_get_login_provider_by_id, db_get_login_providers};
use crate::db::issued_tokens_dao::db_revoke_token;
use crate::db::tms_identities_dao::db_insert_tms_identity_if_absent;
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::obj_model::institution::Institution;
use crate::obj_model::login::WhoAmIResponse;
use crate::routes::login::SIGNIN_PATH;
use crate::services::globus_token_provider::GlobusTokenProvider;
use crate::services::institution_service::{
    PROVIDER_TACC_TAPIS, get_institution_by_id, get_institutions, is_approved_globus_identity,
};
use crate::services::resource_service::ResourceProviderAuthorizationCodeResponse;
use crate::services::token_provider::TokenProvider;
use crate::utils::app_error::AppError;
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{
    TmsTokenClaims, get_signing_key, get_tms_token_claims, make_auth_token,
};
use crate::utils::oauth2_authorization_code_utils::{
    LoginCallbackQueryParams, OAuth2State, decode_access_token, get_token_for_provider,
};
use crate::utils::state_utils::decode_state;
use anyhow::{Context, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use jsonwebtoken::decode_header;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::ServiceError;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
}

const GLOBUS_LOGOUT_URL: &str = "https://auth.globus.org/v2/web/logout";

pub enum CallbackResult {
    LoggedIn {
        token: String,
        expires_in: i64,
        // the post-login page, from the state (validated in the login step)
        redirect_uri: String,
    },
    // no tms token: the sign in page explains why
    Rejected(LoginRejection),
}
pub async fn get_identity_providers(pool: &PgPool) -> Result<HashSet<IdentityProvider>> {
    let mut tx = pool.begin().await?;
    let idps = db_get_login_providers(&mut tx).await?;
    tx.commit().await?;

    let mut idp_result = HashSet::new();
    idps.iter().for_each(|idp| {
        idp_result.insert(idp.clone());
    });
    Ok(idp_result)
}

/*
The pieces below are shared by the two flows that log a user in through the login idp:
the portal's own login (/login, /login/callback) and the TMS OAuth2 authorization code
flow for other clients (/oauth2/authorize, /oauth2/callback).
 */

// the login idp to use, and the approved institution it was chosen for
pub struct LoginTarget {
    pub idp: IdentityProvider,
    pub institution: Institution,
}

pub enum LoginIdentity {
    Approved(HashMap<String, Value>),
    Rejected(LoginRejection),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoginRejection {
    // the chosen institution, or the Globus identity, is not from an approved institution
    UnapprovedInstitution,
    // Globus did not authenticate the user again for this login (see prompt=login)
    StaleAuthentication,
    // the user cancelled the login at the idp (it sent back error=access_denied)
    LoginCancelled,
    // the idp sent back another error, or the code could not be exchanged for a token
    LoginFailed,
    // no state cookie, or it doesn't match the state that came back (another login was
    // started since), or the state expired (the login took more than 5 minutes)
    LoginExpired,
    // the login was started without an institution, or with one that isn't approved
    InvalidInstitution,
    // the login can't start: tms_server's institution list can't be read, or the login idp
    // isn't configured
    LoginUnavailable,
}

impl LoginRejection {
    // error query param value the sign in page shows a message for
    pub fn error_code(&self) -> &'static str {
        match self {
            LoginRejection::UnapprovedInstitution => "unapproved_institution",
            LoginRejection::StaleAuthentication => "stale_authentication",
            LoginRejection::LoginCancelled => "login_cancelled",
            LoginRejection::LoginFailed => "login_failed",
            LoginRejection::LoginExpired => "login_expired",
            LoginRejection::InvalidInstitution => "unapproved_institution",
            LoginRejection::LoginUnavailable => "login_unavailable",
        }
    }

    // error for an OAuth2 client's callback (RFC 6749, section 4.1.2.1)
    pub fn oauth2_error(&self) -> &'static str {
        match self {
            LoginRejection::LoginFailed => "server_error",
            LoginRejection::InvalidInstitution => "invalid_request",
            LoginRejection::LoginUnavailable => "temporarily_unavailable",
            _ => "access_denied",
        }
    }

    // error_description for an OAuth2 client's callback
    pub fn description(&self) -> &'static str {
        match self {
            LoginRejection::UnapprovedInstitution => {
                "The identity is not from an approved institution"
            }
            LoginRejection::StaleAuthentication => {
                "The user did not authenticate again for this login"
            }
            LoginRejection::LoginCancelled => "The user cancelled the login",
            LoginRejection::LoginFailed => "The login could not be completed",
            LoginRejection::LoginExpired => "The login expired or was started again",
            LoginRejection::InvalidInstitution => "An approved institution is required",
            LoginRejection::LoginUnavailable => "Logging in is unavailable right now",
        }
    }
}

// what came back to a login callback, before the code is exchanged
pub enum CallbackCheck {
    Valid {
        state: OAuth2State,
        code: String,
    },
    // state is the decoded state when it could be trusted, so an OAuth2 client can be told
    Rejected {
        state: Option<OAuth2State>,
        rejection: LoginRejection,
    },
}

/*
Check what the login idp sent back to a login callback, before the code is exchanged.  The
state must be the one in this browser's state cookie, and still valid (it is signed and
expires 5 minutes after the login started).  The idp may send an error instead of a code.
 */
pub async fn check_login_callback(
    configuration: &Configuration,
    query: &LoginCallbackQueryParams,
    cookie_state: Option<&str>,
) -> CallbackCheck {
    let expired = CallbackCheck::Rejected {
        state: None,
        rejection: LoginRejection::LoginExpired,
    };
    let (Some(state), Some(cookie_state)) = (&query.state, cookie_state) else {
        warn!("Login callback without a state, or without a state cookie");
        return expired;
    };
    if state != cookie_state {
        warn!("Login callback state does not match the state cookie");
        return expired;
    }
    let decoded_state: OAuth2State = match decode_state(configuration, state).await {
        Ok(decoded_state) => decoded_state,
        Err(error) => {
            warn!("Login callback state is not valid: {error:#}");
            return expired;
        }
    };

    if let Some(error) = &query.error {
        warn!(
            "Login idp sent back an error: {error} {0}",
            query.error_description.as_deref().unwrap_or_default()
        );
        let rejection = if error == "access_denied" {
            LoginRejection::LoginCancelled
        } else {
            LoginRejection::LoginFailed
        };
        return CallbackCheck::Rejected {
            state: Some(decoded_state),
            rejection,
        };
    }
    match &query.code {
        Some(code) => CallbackCheck::Valid {
            state: decoded_state,
            code: code.clone(),
        },
        None => {
            warn!("Login callback without a code or an error");
            CallbackCheck::Rejected {
                state: Some(decoded_state),
                rejection: LoginRejection::LoginFailed,
            }
        }
    }
}

// the sign in page, with the error query param its banner shows a message for
pub fn get_signin_url(configuration: &Configuration, rejection: Option<LoginRejection>) -> String {
    let http_config = &configuration.http_config;
    let signin_url = http_config.make_tms_url(&http_config.base_url, SIGNIN_PATH);
    match rejection {
        Some(rejection) => format!("{signin_url}?error={0}", rejection.error_code()),
        None => signin_url,
    }
}

// allows for the difference between Globus's clock and ours
const AUTHENTICATION_CLOCK_SKEW_SECS: f64 = 5.0;

// seconds since the epoch, for OAuth2State.login_started
pub fn epoch_seconds_now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

/*
Logging in requires an approved institution, which picks the login idp (Globus or Tapis)
from our configuration.  The institution is looked up by id, so its redirect domain always
comes from tms_server, never from the browser.
 */
pub async fn get_login_target(
    configuration: &Configuration,
    db_pool: &PgPool,
    institution_id: Option<&str>,
) -> std::result::Result<LoginTarget, LoginRejection> {
    let Some(institution_id) = institution_id else {
        warn!("Login started without an institution");
        return Err(LoginRejection::InvalidInstitution);
    };
    let institutions = get_institutions(configuration).await.map_err(|error| {
        warn!("Unable to get the approved institutions: {error:#}");
        LoginRejection::LoginUnavailable
    })?;
    let Some(institution) = institutions
        .into_iter()
        .find(|institution| institution.id == institution_id)
    else {
        warn!("Login started with {institution_id}, which is not an approved institution");
        return Err(LoginRejection::InvalidInstitution);
    };
    let (login_idp_id, login_idp_type) = match institution.provider.as_str() {
        PROVIDER_TACC_TAPIS => (
            configuration
                .oauth_config
                .tapis_login_oauth_provider
                .as_ref()
                .ok_or_else(|| {
                    warn!("Tapis login is not configured (TMS_PORTAL_TAPIS_LOGIN_PROVIDER)");
                    LoginRejection::LoginUnavailable
                })?,
            IdentityProviderType::TaccTapis,
        ),
        _ => (
            &configuration.oauth_config.login_oauth_provider,
            IdentityProviderType::Globus,
        ),
    };

    let idp = get_login_idp(db_pool, login_idp_id)
        .await
        .map_err(|error| {
            warn!("Unable to get the login idp {login_idp_id}: {error:#}");
            LoginRejection::LoginUnavailable
        })?;
    if idp.identity_provider_type != login_idp_type {
        warn!(
            "Login idp {0} is not a {1} provider",
            idp.id, login_idp_type
        );
        return Err(LoginRejection::LoginUnavailable);
    }
    Ok(LoginTarget { idp, institution })
}

async fn get_login_idp(db_pool: &PgPool, login_idp_id: &String) -> Result<IdentityProvider> {
    let mut tx = db_pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, login_idp_id).await?;
    tx.commit().await?;
    Ok(idp)
}

/*
The login idp's authorize url, which the browser is redirected to.  For Globus:
- prompt=login makes Globus authenticate the user again even if they have a Globus session
  (a user can drop it from the url, so get_login_identity checks last_authentication too)
- an institution with a redirect_idp_domain steers the login to that institution's IdP
  (users not yet signed in skip the Globus login page), and session_required_single_domain
  makes Globus require an identity from that domain
 */
pub fn get_login_authorize_url(
    login_target: &LoginTarget,
    callback_url: &str,
    encoded_state: &str,
) -> Result<Url> {
    let idp = &login_target.idp;
    let encoded_nonce = BASE64_STANDARD.encode(generate_nonce().to_ne_bytes());
    let mut query_params = vec![
        ("response_type", "code"),
        ("client_id", idp.client_id.as_str()),
        ("redirect_uri", callback_url),
        ("state", encoded_state),
        ("nonce", &encoded_nonce),
        ("access_type", "offline"),
    ];

    if let Some(scope) = &idp.scope {
        query_params.push(("scope", scope.as_str()))
    }
    if idp.identity_provider_type == IdentityProviderType::Globus {
        query_params.push(("prompt", "login"));
        if let Some(domain) = &login_target.institution.redirect_idp_domain {
            query_params.push(("redirect_idp_domain", domain));
            query_params.push(("session_required_single_domain", domain));
        }
    }

    Ok(Url::parse_with_params(
        &idp.identity_redirect_url,
        query_params,
    )?)
}

/*
Whether Globus authenticated the user after the login started, as prompt=login asks.
last_authentication is the epoch time the identity last authenticated.  Without either
time the login is treated as not fresh.
 */
fn is_fresh_authentication(claims: &HashMap<String, Value>, login_started: Option<u64>) -> bool {
    let (Some(login_started), Some(last_authentication)) = (
        login_started,
        claims.get("last_authentication").and_then(Value::as_f64),
    ) else {
        return false;
    };
    last_authentication + AUTHENTICATION_CLOCK_SKEW_SECS >= login_started as f64
}

/*
Exchange the code from the login idp for the user's identity claims.  The institution chosen
at the start of the login must still be approved (the list may have changed since).
redirect_idp_domain only steers the Globus login, it does not restrict it, so a Globus
identity must also be from an approved institution, and Globus must have authenticated the
user for this login rather than reusing an earlier session.
 */
pub async fn get_login_identity(
    configuration: &Configuration,
    idp: &IdentityProvider,
    state: &OAuth2State,
    callback_url: &String,
    code: &String,
) -> Result<LoginIdentity> {
    let Some(institution_id) = &state.institution_id else {
        return Ok(LoginIdentity::Rejected(
            LoginRejection::UnapprovedInstitution,
        ));
    };
    if !get_institutions(configuration)
        .await?
        .iter()
        .any(|institution| &institution.id == institution_id)
    {
        return Ok(LoginIdentity::Rejected(
            LoginRejection::UnapprovedInstitution,
        ));
    }

    let claims_result: Result<HashMap<String, Value>> = match idp.identity_provider_type {
        IdentityProviderType::TaccTapis => get_tapis_claims(idp, callback_url, code).await,
        _ => {
            async {
                let token: AuthorizationCodeResponse =
                    get_token_for_provider(idp, callback_url, code).await?;
                decode_access_token(idp, &token.id_token).await
            }
            .await
        }
    };
    // e.g. an expired or already used code (a reloaded callback), or the idp being down
    let mut claims = match claims_result {
        Ok(claims) => claims,
        Err(error) => {
            warn!(
                "Unable to get the login identity from {0}: {error:#}",
                idp.id
            );
            return Ok(LoginIdentity::Rejected(LoginRejection::LoginFailed));
        }
    };

    if idp.identity_provider_type == IdentityProviderType::Globus {
        let username = claims
            .get("preferred_username")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !is_approved_globus_identity(configuration, username).await? {
            return Ok(LoginIdentity::Rejected(
                LoginRejection::UnapprovedInstitution,
            ));
        }
        if !is_fresh_authentication(&claims, state.login_started) {
            return Ok(LoginIdentity::Rejected(LoginRejection::StaleAuthentication));
        }
    }

    // TODO: get iss from config
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    Ok(LoginIdentity::Approved(claims))
}

pub async fn handle_callback(
    configuration: &Configuration,
    db_pool: &PgPool,
    query: &LoginCallbackQueryParams,
    cookie_state: Option<&str>,
) -> Result<CallbackResult> {
    let (decoded_state, code) = match check_login_callback(configuration, query, cookie_state).await
    {
        CallbackCheck::Valid { state, code } => (state, code),
        CallbackCheck::Rejected { rejection, .. } => {
            return Ok(CallbackResult::Rejected(rejection));
        }
    };

    let mut tx = db_pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    tx.commit().await?;

    let callback_url = configuration
        .http_config
        .get_identity_provider_callback_url();
    let claims = match get_login_identity(configuration, &idp, &decoded_state, &callback_url, &code)
        .await?
    {
        LoginIdentity::Approved(claims) => claims,
        LoginIdentity::Rejected(rejection) => return Ok(CallbackResult::Rejected(rejection)),
    };

    let mut tms_token_claims = get_tms_token_claims(
        configuration,
        &decoded_state.client_id,
        &idp.id,
        &idp.identity_provider_type,
        &claims,
    )
    .await?;
    // remembered so logout can end the institution's IdP session too
    tms_token_claims.tms_institution = decoded_state.institution_id.map(Value::from);

    // make sure the tms_identity is stored in the db
    let mut tx = db_pool.begin().await?;
    let tms_identity = tms_token_claims.get_sub()?;
    db_insert_tms_identity_if_absent(&mut tx, &tms_identity).await?;
    tx.commit().await?;
    let tms_token = make_auth_token(configuration, db_pool, &tms_token_claims).await?;
    Ok(CallbackResult::LoggedIn {
        token: tms_token,
        expires_in: tms_token_claims.get_expires_in()?,
        redirect_uri: decoded_state.redirect_uri,
    })
}

/*
Exchange the code for a Tapis token, and get the identity claims from its id token.
Tapis wraps the tokens in its response envelope, like it does for resource providers.
 */
async fn get_tapis_claims(
    idp: &IdentityProvider,
    callback_url: &String,
    code: &String,
) -> Result<HashMap<String, Value>> {
    let response: ResourceProviderAuthorizationCodeResponse =
        get_token_for_provider(idp, callback_url, code).await?;
    let mut claims: HashMap<String, Value> = JwtDecoderBuilder::builder()
        .jwks_url(&idp.oauth2_jwks_url)
        .decode(&response.result.access_token.id_token)
        .await
        .context("Error decoding Tapis id token")?;

    // Tapis tokens have no name or organization claims, which whoami returns
    if !claims.contains_key("name")
        && let Some(username) = claims.get("tapis/username").or(claims.get("sub")).cloned()
    {
        claims.insert("name".to_string(), username);
    }
    if !claims.contains_key("organization")
        && let Some(tenant) = claims.get("tapis/tenant_id").cloned()
    {
        claims.insert("organization".to_string(), tenant);
    }
    Ok(claims)
}

/*
Revoke the token and return where to send the browser.  For Globus that's its
sign-out page, which ends the Globus session (otherwise the next login skips the
institution's IdP).  If the institution has its own IdP sign-out, Globus's page links on
to /logout/idp to end that session too.  Tapis's sign-out page has no way back to the
portal, so the client opens it in a new tab (whoami's logout url) and this returns the
sign in page.
 */
pub async fn logout(
    configuration: &Configuration,
    db_pool: &PgPool,
    token: Option<&String>,
) -> Result<String> {
    let signin_url = get_signin_url(configuration, None);
    // Without a token we can still log out (the cookie is removed), but we can't tell which
    // provider's sign-out page to send the browser to.  An expired token's signature check
    // fails too; a revoked one still decodes, so its provider is still known.
    let Some(token) = token else {
        return Ok(signin_url);
    };
    let tms_token_claims = match decode_tms_token(configuration, token).await {
        Ok(tms_token_claims) => tms_token_claims,
        Err(error) => {
            warn!("Logging out with a token that can't be decoded: {error:#}");
            return Ok(signin_url);
        }
    };
    let mut tx = db_pool.begin().await?;
    match db_revoke_token(&mut tx, token).await {
        Ok(_) => {}
        // e.g. after the database was reset: there is nothing to revoke, so carry on
        Err(error)
            if matches!(
                error.downcast_ref::<ServiceError>(),
                Some(ServiceError::NotFound(_))
            ) =>
        {
            warn!("Logging out with a token that isn't in the database")
        }
        Err(error) => return Err(error),
    }
    tx.commit().await?;

    let http_config = &configuration.http_config;
    if tms_token_claims.tms_idp_provider != IdentityProviderType::Globus {
        return Ok(signin_url);
    }

    let institution = match tms_token_claims
        .tms_institution
        .as_ref()
        .and_then(Value::as_str)
    {
        Some(institution_id) => get_institution_by_id(configuration, institution_id)
            .await
            .ok(),
        None => None,
    };
    let (return_url, return_name) = match institution {
        Some(institution) if institution.idp_logout_url.is_some() => {
            let mut idp_logout =
                Url::parse(&http_config.make_tms_url(&http_config.base_url, "logout/idp"))?;
            idp_logout
                .query_pairs_mut()
                .append_pair("institution", &institution.id);
            (
                idp_logout.to_string(),
                format!("{} sign-out", institution.name),
            )
        }
        _ => (signin_url, "TMS Portal".to_string()),
    };

    let mut tx = db_pool.begin().await?;
    let idp =
        db_get_login_provider_by_id(&mut tx, &configuration.oauth_config.login_oauth_provider)
            .await?;
    tx.commit().await?;
    Ok(Url::parse_with_params(
        GLOBUS_LOGOUT_URL,
        [
            ("client_id", idp.client_id.as_str()),
            ("redirect_uri", return_url.as_str()),
            ("redirect_name", return_name.as_str()),
        ],
    )?
    .to_string())
}

async fn decode_tms_token(configuration: &Configuration, token: &String) -> Result<TmsTokenClaims> {
    let token_header = decode_header(token)?;
    let key = get_signing_key(configuration, &token_header.kid)?;

    JwtDecoderBuilder::builder()
        .public_key(key.jwt_public_key.as_bytes())
        .decode(token)
        .await
        .context("Error decoding JWT")
}

pub async fn whoami(
    configuration: &Configuration,
    db_pool: &PgPool,
    token: &String,
) -> anyhow::Result<WhoAmIResponse, AppError> {
    let tms_token_claims = decode_tms_token(configuration, token).await?;

    let idp_provider = &tms_token_claims.tms_idp_provider;
    let token_provider = match idp_provider {
        // claims from Tapis are mapped to the same tms claims as Globus at login
        IdentityProviderType::Globus | IdentityProviderType::TaccTapis => GlobusTokenProvider {},
        _ => {
            return Err(ServiceError::Internal(format!(
                "Unsupported provider type {0}",
                idp_provider
            ))
            .into());
        }
    };

    let mut whoami_response = token_provider.whoami(&tms_token_claims)?;
    if *idp_provider == IdentityProviderType::TaccTapis
        && let Some(tapis_idp_id) = &configuration.oauth_config.tapis_login_oauth_provider
    {
        // Tapis's sign-out page is next to its authorize endpoint
        let mut tx = db_pool.begin().await?;
        let idp = db_get_login_provider_by_id(&mut tx, tapis_idp_id).await?;
        tx.commit().await?;
        whoami_response.logout_url = Some(
            Url::parse(&idp.identity_redirect_url)?
                .join("logout")?
                .to_string(),
        );
    }
    Ok(whoami_response)
}

#[cfg(test)]
mod tests {
    use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
    use crate::obj_model::institution::Institution;
    use crate::services::login_service::{
        CallbackCheck, LoginRejection, LoginTarget, check_login_callback, get_login_authorize_url,
        get_login_target, get_signin_url, is_fresh_authentication,
    };
    use crate::utils::oauth2_authorization_code_utils::LoginCallbackQueryParams;
    use chrono::Utc;
    use serde_json::Value;
    use std::collections::HashMap;

    fn globus_target(redirect_idp_domain: Option<&str>) -> LoginTarget {
        LoginTarget {
            idp: IdentityProvider {
                id: "globus_idp".to_string(),
                name: "Globus".to_string(),
                client_id: "client-id".to_string(),
                client_secret: "client-secret".to_string(),
                identity_redirect_url: "https://auth.globus.org/v2/oauth2/authorize".to_string(),
                oauth2_token_url: "https://auth.globus.org/v2/oauth2/token".to_string(),
                oauth2_jwks_url: None,
                oidc_user_info_url: None,
                oauth2_public_key: None,
                scope: Some("openid profile email".to_string()),
                identity_provider_type: IdentityProviderType::Globus,
                supports_login: true,
                resources_services_url: None,
                supports_resources: false,
                created: Utc::now(),
                updated: Utc::now(),
            },
            institution: Institution {
                id: "utexas".to_string(),
                name: "The University of Texas at Austin".to_string(),
                provider: "globus".to_string(),
                redirect_idp_domain: redirect_idp_domain.map(String::from),
                idp_logout_url: None,
            },
        }
    }

    #[test]
    fn test_authorize_url_carries_redirect_idp_domain() -> anyhow::Result<()> {
        let url = get_login_authorize_url(
            &globus_target(Some("utexas.edu")),
            "https://tms.example.org/oauth2/callback",
            "state123",
        )?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(url.host_str(), Some("auth.globus.org"));
        assert_eq!(query["redirect_idp_domain"], "utexas.edu");
        assert_eq!(query["session_required_single_domain"], "utexas.edu");
        assert_eq!(query["prompt"], "login");
        assert_eq!(query["client_id"], "client-id");
        assert_eq!(
            query["redirect_uri"],
            "https://tms.example.org/oauth2/callback"
        );
        assert_eq!(query["state"], "state123");
        assert_eq!(query["scope"], "openid profile email");
        // the url goes to the browser, so it must not carry the client secret
        assert!(!url.as_str().contains("client-secret"));
        Ok(())
    }

    #[test]
    fn test_authorize_url_without_domain() -> anyhow::Result<()> {
        // e.g. "Other institution (choose in Globus)": Globus shows its organization picker
        let url = get_login_authorize_url(&globus_target(None), "https://tms/cb", "s")?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert!(!query.contains_key("redirect_idp_domain"));
        assert!(!query.contains_key("session_required_single_domain"));
        // still a new Globus login, even without an institution domain
        assert_eq!(query["prompt"], "login");
        Ok(())
    }

    #[test]
    fn test_tapis_authorize_url_has_no_globus_params() -> anyhow::Result<()> {
        let mut tapis_target = globus_target(None);
        tapis_target.idp.identity_provider_type = IdentityProviderType::TaccTapis;
        tapis_target.idp.identity_redirect_url =
            "https://tacc.tapis.io/v3/oauth2/authorize".to_string();
        tapis_target.institution.provider = "tacc_tapis".to_string();
        let url = get_login_authorize_url(&tapis_target, "https://tms/cb", "s")?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert!(!query.contains_key("prompt"));
        assert!(!query.contains_key("redirect_idp_domain"));
        assert!(!query.contains_key("session_required_single_domain"));
        Ok(())
    }

    fn claims_authenticated_at(last_authentication: Option<Value>) -> HashMap<String, Value> {
        let mut claims = HashMap::from([(
            "preferred_username".to_string(),
            Value::from("jdoe@utexas.edu"),
        )]);
        if let Some(last_authentication) = last_authentication {
            claims.insert("last_authentication".to_string(), last_authentication);
        }
        claims
    }

    #[test]
    fn test_fresh_authentication() {
        let login_started = 1_800_000_000;
        // authenticated after the login started, or within the clock skew allowance
        for authenticated in [login_started + 5, login_started, login_started - 5] {
            assert!(is_fresh_authentication(
                &claims_authenticated_at(Some(Value::from(authenticated))),
                Some(login_started)
            ));
        }
        // a float timestamp is accepted too
        assert!(is_fresh_authentication(
            &claims_authenticated_at(Some(Value::from(login_started as f64 + 0.5))),
            Some(login_started)
        ));
    }

    #[test]
    fn test_stale_authentication() {
        let login_started = 1_800_000_000;
        // an earlier Globus session was reused
        assert!(!is_fresh_authentication(
            &claims_authenticated_at(Some(Value::from(login_started - 6))),
            Some(login_started)
        ));
        assert!(!is_fresh_authentication(
            &claims_authenticated_at(Some(Value::from(login_started - 3600))),
            Some(login_started)
        ));
        // no last_authentication claim, or one that isn't a number
        assert!(!is_fresh_authentication(
            &claims_authenticated_at(None),
            Some(login_started)
        ));
        assert!(!is_fresh_authentication(
            &claims_authenticated_at(Some(Value::from("yesterday"))),
            Some(login_started)
        ));
        // a state from before login_started was recorded
        assert!(!is_fresh_authentication(
            &claims_authenticated_at(Some(Value::from(login_started))),
            None
        ));
    }

    fn configuration() -> crate::utils::configuration::Configuration {
        use crate::obj_model::configuration::{
            DelegationPolicyConfig, HttpConfig, JwtConfig, OAuthConfig, RuntimeConfig, StateKey,
        };
        use crate::obj_model::keys::Key;
        crate::utils::configuration::Configuration {
            http_config: HttpConfig {
                base_url: "https://tms.example.org".to_string(),
                ..Default::default()
            },
            oauth_config: OAuthConfig {
                login_oauth_provider: "globus".to_string(),
                tapis_login_oauth_provider: None,
            },
            jwt_config: JwtConfig::default(),
            runtime_config: RuntimeConfig::default(),
            delegation_policy_config: DelegationPolicyConfig::default(),
            // no real key: any state fails its signature check
            state_key: StateKey {
                kid: "tms-state-key-1".to_string(),
                key: Key::default(),
            },
        }
    }

    fn callback(
        code: Option<&str>,
        state: Option<&str>,
        error: Option<&str>,
    ) -> LoginCallbackQueryParams {
        LoginCallbackQueryParams {
            code: code.map(String::from),
            state: state.map(String::from),
            error: error.map(String::from),
            error_description: None,
        }
    }

    fn expired(check: CallbackCheck) -> bool {
        matches!(
            check,
            CallbackCheck::Rejected {
                state: None,
                rejection: LoginRejection::LoginExpired,
            }
        )
    }

    #[tokio::test]
    async fn test_callback_without_matching_state_is_expired() {
        let configuration = configuration();
        // no state cookie (e.g. a reloaded callback after a successful login)
        assert!(expired(
            check_login_callback(&configuration, &callback(Some("c"), Some("s"), None), None).await
        ));
        // no state in the callback
        assert!(expired(
            check_login_callback(&configuration, &callback(Some("c"), None, None), Some("s")).await
        ));
        // another login was started since, so the cookie holds its state
        assert!(expired(
            check_login_callback(
                &configuration,
                &callback(Some("c"), Some("first"), None),
                Some("second")
            )
            .await
        ));
        // an idp error doesn't get past a state that doesn't match
        assert!(expired(
            check_login_callback(
                &configuration,
                &callback(None, Some("first"), Some("access_denied")),
                Some("second")
            )
            .await
        ));
    }

    #[tokio::test]
    async fn test_callback_with_unverifiable_state_is_expired() {
        // matches the cookie, but isn't a valid signed state (or has expired)
        let configuration = configuration();
        assert!(expired(
            check_login_callback(
                &configuration,
                &callback(Some("c"), Some("not-a-jwt"), None),
                Some("not-a-jwt")
            )
            .await
        ));
    }

    #[test]
    fn test_rejection_codes() {
        let rejections = [
            (
                LoginRejection::UnapprovedInstitution,
                "unapproved_institution",
                "access_denied",
            ),
            (
                LoginRejection::StaleAuthentication,
                "stale_authentication",
                "access_denied",
            ),
            (
                LoginRejection::LoginCancelled,
                "login_cancelled",
                "access_denied",
            ),
            (LoginRejection::LoginFailed, "login_failed", "server_error"),
            (
                LoginRejection::InvalidInstitution,
                "unapproved_institution",
                "invalid_request",
            ),
            (
                LoginRejection::LoginUnavailable,
                "login_unavailable",
                "temporarily_unavailable",
            ),
            (
                LoginRejection::LoginExpired,
                "login_expired",
                "access_denied",
            ),
        ];
        for (rejection, error_code, oauth2_error) in rejections {
            assert_eq!(rejection.error_code(), error_code);
            assert_eq!(rejection.oauth2_error(), oauth2_error);
        }
    }

    #[tokio::test]
    async fn test_login_cannot_start_without_an_approved_institution() {
        let configuration = configuration();
        // never connects: these logins are rejected before the database is needed
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://tms@localhost/unused")
            .unwrap();

        let rejection = get_login_target(&configuration, &db_pool, None).await.err();
        assert_eq!(rejection, Some(LoginRejection::InvalidInstitution));

        // no tms_server url, so there's no approved list to log in against
        let rejection = get_login_target(&configuration, &db_pool, Some("utexas"))
            .await
            .err();
        assert_eq!(rejection, Some(LoginRejection::LoginUnavailable));
    }

    #[test]
    fn test_signin_url() {
        let configuration = configuration();
        assert_eq!(
            get_signin_url(&configuration, None),
            "https://tms.example.org/signin"
        );
        assert_eq!(
            get_signin_url(&configuration, Some(LoginRejection::LoginCancelled)),
            "https://tms.example.org/signin?error=login_cancelled"
        );
    }
}
