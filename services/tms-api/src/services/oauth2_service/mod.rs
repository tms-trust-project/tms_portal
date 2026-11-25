use crate::constants::TMS_ISSUER;
use crate::infra::persistence::auth_code_data_repository::auth_code_data_dao::{
    delete_auth_code_data, insert_auth_code_data,
};
use crate::infra::persistence::client_repository::get_client_by_credentials;
use crate::infra::persistence::identity_provider_repository::identity_provider_dao::get_login_provider_by_id;
use crate::obj_model::configuration::HttpConfig;
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::services::institution_service::InstitutionService;
use crate::services::login_service::{
    AuthorizationCodeResponse, LoginRejection, LoginService, check_login_callback,
    get_login_authorize_url, get_signin_url,
};
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{JwtClaims, get_tms_token_claims, make_auth_token};
use crate::utils::oauth2_authorization_code_utils::{
    LoginCallbackQueryParams, OAuth2State, decode_access_token, get_token_for_provider,
};
use crate::utils::redirect_utils::check_allowed_redirects;
use crate::utils::state_utils::encode_state;
use anyhow::Context;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::TimeDelta;
use rand::RngExt;
use rand::distr::Alphanumeric;
use rand::rngs::ThreadRng;
use serde_json::{Value, from_value};
use sqlx::PgPool;
use std::time::SystemTime;
use tms_lib::utils::oauth_utils::generate_nonce;
use log::warn;
use tms_lib::utils::service_error::ServiceError::{self, Internal};
use url::Url;

pub struct TokenResponse {
    pub access_token: String,
    pub expires_at: String,
    pub expires_in: i64,
    pub id_token: String,
    pub jti: String,
}

fn generate_code() -> String {
    ThreadRng::default()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/*
Where /oauth2/authorize sends the browser.
 */
pub enum AuthorizeOutcome {
    // to the login identity provider, with the state for the state cookie
    Login { location: String, encoded_state: String },
    // to the portal's institution picker, or back to the client with an error
    Redirect { location: String },
}

pub struct AuthorizeCodeRequest<'a> {
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub state: &'a Option<String>,
    pub scope: &'a Option<String>,
    pub institution: &'a Option<String>,
}

/*
Starts another client's OAuth2 login.  The client and its redirect uri are checked first;
an error there is returned (never redirected), since the redirect uri can't be trusted.
- Without an institution the browser goes to the portal's sign in page, which shows the
  institution picker and sends the same request back here with the chosen institution.
- An unusable institution, or no Globus login idp, goes back to the client's redirect uri
  with an RFC 6749 error.
- Otherwise the browser goes to Globus, steered to (and required to use) the
  institution's domain, like a portal login.
 */
pub async fn authorize_code(
    configuration: &Configuration,
    db_pool: &PgPool,
    institution_service: &InstitutionService,
    request: &AuthorizeCodeRequest<'_>,
) -> anyhow::Result<AuthorizeOutcome> {
    // Check redirect uri - this fails if the redirect doesnt exist
    let mut tx = db_pool.begin().await?;
    check_allowed_redirects(&mut tx, request.client_id, request.redirect_uri)
        .await
        .with_context(|| "OAuth redirect is invalid")?;
    tx.commit().await?;

    let reject = |rejection: LoginRejection| -> anyhow::Result<AuthorizeOutcome> {
        let location =
            get_oauth2_error_location(request.redirect_uri, request.state.as_deref(), rejection)?;
        Ok(AuthorizeOutcome::Redirect { location })
    };

    let Some(institution_id) = request.institution else {
        return Ok(AuthorizeOutcome::Redirect {
            location: get_hosted_signin_url(&configuration.http_config, request)?,
        });
    };

    let mut tx = db_pool.begin().await?;
    let login_idp_id = &configuration.oauth_config.login_oauth_provider;
    let idp = match get_login_provider_by_id(&mut tx, login_idp_id).await {
        Ok(idp) => idp,
        Err(error) => {
            warn!("Unable to get the login idp {login_idp_id}: {error:#}");
            return reject(LoginRejection::LoginUnavailable);
        }
    };
    tx.commit().await?;
    if idp.identity_provider_type != IdentityProviderType::Globus {
        warn!("Institution logins need a Globus login idp, and {0} isn't one", idp.id);
        return reject(LoginRejection::LoginUnavailable);
    }
    let domain = match institution_service.globus_login_domain(institution_id).await {
        Ok(domain) => domain,
        Err(error)
            if matches!(error.downcast_ref::<ServiceError>(), Some(ServiceError::BadRequest(_))) =>
        {
            warn!("OAuth2 login started with an unusable institution: {error:#}");
            return reject(LoginRejection::UnapprovedInstitution);
        }
        Err(error) => return Err(error),
    };

    // encode our state
    let encoded_state = get_state(configuration, request, &idp.id, institution_id).await?;

    // compute the location url that will be used for the redirect
    let callback_url = configuration.http_config.get_oauth_provider_callback_url();
    let encoded_nonce = BASE64_STANDARD.encode(generate_nonce().to_ne_bytes());
    let location =
        get_login_authorize_url(&idp, &callback_url, &encoded_state, &encoded_nonce, &domain)?;

    Ok(AuthorizeOutcome::Login { location: location.to_string(), encoded_state })
}

/*
The portal's sign in page, carrying the client's authorize request.  The page shows the
institution picker and sends the request back to /oauth2/authorize with &institution=<id>.
authorize_code checks the client and redirect uri again then, so the request needs no
signature.
 */
pub fn get_hosted_signin_url(
    http_config: &HttpConfig,
    request: &AuthorizeCodeRequest<'_>,
) -> anyhow::Result<String> {
    let mut url = Url::parse(&get_signin_url(http_config, None))?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("response_type", "code");
        query.append_pair("client_id", request.client_id);
        query.append_pair("redirect_uri", request.redirect_uri);
        if let Some(state) = request.state {
            query.append_pair("state", state);
        }
        if let Some(scope) = request.scope {
            query.append_pair("scope", scope);
        }
    }
    Ok(url.to_string())
}

/*
The client's redirect uri with an RFC 6749 (4.1.2.1) error: error, error_description and
the client's state.  Only used with a redirect uri that has been checked.
 */
pub fn get_oauth2_error_location(
    redirect_uri: &str,
    client_state: Option<&str>,
    rejection: LoginRejection,
) -> anyhow::Result<String> {
    let mut location = Url::parse(redirect_uri)?;
    {
        let mut query = location.query_pairs_mut();
        query.append_pair("error", rejection.oauth2_error());
        query.append_pair("error_description", rejection.description());
        if let Some(state) = client_state {
            query.append_pair("state", state);
        }
    }
    Ok(location.to_string())
}

pub async fn generate_code_and_redirect(
    pool: &PgPool,
    configuration: &Configuration,
    identity_provider: &IdentityProvider,
    state: &OAuth2State,
    claims: &JwtClaims,
) -> anyhow::Result<Url> {
    let auth_code = generate_code();

    let mut claims = claims.clone();
    // TODO: get iss from config
    claims.insert("iss".to_string(), Value::from(TMS_ISSUER));
    // check_institution_login has already refused a state without an institution
    let institution_id = state.institution_id.as_deref().context("The login has no institution")?;
    let tms_token_claims = get_tms_token_claims(
        configuration,
        &state.client_id,
        &identity_provider.id,
        &identity_provider.identity_provider_type,
        institution_id,
        &claims,
    )
    .await?;

    let mut tx = pool.begin().await?;
    insert_auth_code_data(
        &mut tx,
        &auth_code,
        &state.client_id,
        &state.redirect_uri,
        &tms_token_claims,
        &identity_provider.id,
        &identity_provider.identity_provider_type,
    )
    .await?;
    tx.commit().await?;

    let mut location = Url::parse(&state.redirect_uri)?;
    if let Some(state) = &state.client_state {
        location.query_pairs_mut().append_pair("state", state);
    }
    location.query_pairs_mut().append_pair("code", &auth_code);
    Ok(location)
}

async fn get_state(
    configuration: &Configuration,
    request: &AuthorizeCodeRequest<'_>,
    idp_id: &str,
    institution_id: &str,
) -> anyhow::Result<String> {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    // populate state structure
    let oauth_state = OAuth2State {
        tms_identity: String::default(), // we don't have a tms identity at this point
        client_id: request.client_id.to_owned(),
        idp_id: idp_id.to_owned(),
        redirect_uri: request.redirect_uri.to_owned(),
        // TODO:  This should be a config setting
        exp: now + 300,
        nonce: generate_nonce(),
        client_state: request.state.clone(),
        institution_id: Some(institution_id.to_owned()),
        login_started: now,
    };

    // encode state structure
    match encode_state(&configuration.state_key, oauth_state)
        .await
        .context("Unable to encode state")
    {
        Ok(state_string) => Ok(state_string),
        Err(error) => Err(Internal(error.to_string()).into()),
    }
}

pub async fn get_access_token_from_code(
    configuration: &Configuration,
    db_pool: &PgPool,
    client_id: &String,
    client_secret: &String,
    code: &String,
    redirect_uri: &String,
) -> anyhow::Result<TokenResponse> {
    // validate client id
    let mut tx = db_pool.begin().await?;
    let client = get_client_by_credentials(&mut tx, client_id, client_secret).await?;

    // validate redirect uri
    check_allowed_redirects(&mut tx, &client.client_id, redirect_uri)
        .await
        .with_context(|| "OAuth redirect is invalid")?;
    // TODO: get time delta fron config (how recently the auth code must have been issued)
    let time_delta = TimeDelta::seconds(30);

    // this will remove the auth code data, but return it.  It's only valid for a single use, so we
    // don't want it hanging around.
    // TODO: we could invalidate the record insted if we think it might help debuging / tracking errors down.  They would still need to be removed at some point, but we could do that once a day or whatever.  I prefer just deleting them I think.
    let auth_code_data =
        delete_auth_code_data(&mut tx, code, client_id, redirect_uri, time_delta).await?;
    // let idp = db_get_login_provider_by_id(&mut tx, &auth_code_data.idp_id).await?;
    tx.commit().await?;

    let tms_token_claims = from_value(auth_code_data.claims)?;
    let tms_token_string = make_auth_token(configuration, db_pool, &tms_token_claims).await?;

    Ok(TokenResponse {
        access_token: tms_token_string.clone(),
        expires_at: tms_token_claims.get_expires_at()?,
        expires_in: tms_token_claims.get_expires_in()?,
        // TODO: why are we creating an empty `id_token`
        id_token: String::from(""),
        jti: tms_token_claims.get_jti()?,
    })
}

/*
The login identity provider's redirect back to /oauth2/callback, for another client's
login.  Returns where to send the browser: the client's redirect uri with a code, or with
an RFC 6749 error when the login is refused, cancelled or fails.  When the state can't be
trusted (missing, not this browser's, expired), the client's redirect uri isn't known, so
the browser goes to the portal's sign in page with ?error=<code>.
 */
pub async fn process_authorization_callback(
    configuration: &Configuration,
    db_pool: &PgPool,
    login_service: &LoginService,
    query: &LoginCallbackQueryParams,
    cookie_state: Option<&str>,
) -> String {
    let http_config = &configuration.http_config;
    let (state, code) =
        match check_login_callback(&configuration.state_key, query, cookie_state).await {
            Ok(valid) => valid,
            Err((Some(state), rejection)) => {
                return client_error_location(http_config, &state, rejection);
            }
            Err((None, rejection)) => return get_signin_url(http_config, Some(rejection)),
        };

    match complete_oauth2_login(configuration, db_pool, login_service, &state, &code).await {
        Ok(Ok(location)) => location.to_string(),
        Ok(Err(rejection)) => client_error_location(http_config, &state, rejection),
        Err(error) => {
            warn!("Unable to complete an OAuth2 login for {0}: {error:#}", state.client_id);
            client_error_location(http_config, &state, LoginRejection::LoginFailed)
        }
    }
}

// exchange the code, check the identity against the institution, and issue the TMS code
async fn complete_oauth2_login(
    configuration: &Configuration,
    db_pool: &PgPool,
    login_service: &LoginService,
    state: &OAuth2State,
    code: &str,
) -> anyhow::Result<Result<Url, LoginRejection>> {
    let mut tx = db_pool.begin().await?;
    let idp = get_login_provider_by_id(&mut tx, &state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    tx.commit().await?;

    let callback_url = configuration.http_config.get_oauth_provider_callback_url();
    let claims: anyhow::Result<JwtClaims> = async {
        let token: AuthorizationCodeResponse =
            get_token_for_provider(&idp, &callback_url, code).await?;
        decode_access_token(&idp, &token.id_token).await
    }
    .await;
    let claims = match claims {
        Ok(claims) => claims,
        Err(error) => {
            warn!("Unable to get the login identity from {0}: {error:#}", idp.id);
            return Ok(Err(LoginRejection::LoginFailed));
        }
    };

    if let Err(rejection) = login_service.check_institution_login(state, &claims).await? {
        return Ok(Err(rejection));
    }

    let location = generate_code_and_redirect(db_pool, configuration, &idp, state, &claims).await?;
    Ok(Ok(location))
}

// the client's redirect uri (from the signed state) with the error, or the sign in page
fn client_error_location(
    http_config: &HttpConfig,
    state: &OAuth2State,
    rejection: LoginRejection,
) -> String {
    get_oauth2_error_location(&state.redirect_uri, state.client_state.as_deref(), rejection)
        .unwrap_or_else(|_| get_signin_url(http_config, Some(rejection)))
}

#[cfg(test)]
#[path = "oauth2_service.test.rs"]
mod oauth2_service_test;
