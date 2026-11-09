use crate::AppState;
use crate::obj_model::institution::Institution;
use crate::routes::api_obj_model::login::{
    AuthorizeRequest, IdentityProvider, LoginInstitution, LogoutIdpRequest, WhoAmIResponse,
};
use crate::routes::api_obj_model::tms_response::TmsResponse;
use crate::services::institution_service::{get_institution_by_id, get_institutions};
use crate::services::login_service::{
    CallbackResult, LoginRejection, epoch_seconds_now, get_identity_providers,
    get_login_authorize_url, get_login_target, get_signin_url, handle_callback, logout, whoami,
};
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use crate::utils::oauth2_authorization_code_utils::{
    CLIENT_ID_TMS, LoginCallbackQueryParams, OAuth2State, ROOT_COOKIE_PATH, STATE_COOKIE_NAME,
    TOKEN_COOKIE_NAME,
};
use crate::utils::redirect_utils::{
    check_allowed_redirects, check_allowed_redirects_by_client_name,
};
use crate::utils::state_utils::encode_state;
use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::header::{AUTHORIZATION, LOCATION};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Form, Router, debug_handler};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use chrono::{TimeDelta, Utc};
use log::warn;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use time::OffsetDateTime;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::ServiceError::Internal;
use url::Url;
/*
This file handles the web part of logging into the TMS portal.  This includes tasks such as:
- getting the list of login identity providers
- getting the list of approved institutions (from tms_server) for the login landing page
- requesting a login (which involves a redirect to the browser to login)
- handling the callback (i.e. redirect from the identity provider login)
- whoami - information about the logged in user extracted from the token
 */

// the client app's login landing page
pub const SIGNIN_PATH: &str = "signin";

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/login", get(login_handler))
        .route("/logout", post(logout_handler))
        .route("/logout/idp", get(logout_idp_handler))
        .route("/login/whoami", get(whoami_handler))
        .route("/login/callback", get(callback_handler))
        .route("/login/idps", get(get_idp_handler))
        .route("/login/institutions", get(get_institutions_handler))
}

/*
Accepts a form with information about the login (idp and redirect uri).  There's an
associated table of redirect uris that are allowed.  The redirect url must be in that
table.  This tells us where to redirect back to after the login.  Having multiple allowed
redirects allows for easier debugging - redirect back to localhost if debugging, etc.
A login that can't start (no or an unapproved institution, tms_server unavailable, a bad
redirect) goes back to the sign in page, which explains why.
 */
#[debug_handler]
pub async fn login_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    form_data: Form<AuthorizeRequest>,
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    let rejection = match start_login(&app_state, jar.clone(), &form_data).await {
        Ok(Ok(response)) => return Ok(response),
        Ok(Err(rejection)) => rejection,
        Err(error) => {
            warn!("Unable to start a login: {error:#}");
            LoginRejection::LoginFailed
        }
    };
    // 303, so the browser loads the sign in page with a GET (the login may have been a POST)
    let headers = HashMap::from([(
        LOCATION.to_string(),
        get_signin_url(&app_state.config, Some(rejection)),
    )]);
    Ok((
        jar,
        TmsResponse::builder(StatusCode::SEE_OTHER)
            .headers(headers)
            .build(),
    ))
}

async fn start_login(
    app_state: &AppState,
    jar: CookieJar,
    form_data: &AuthorizeRequest,
) -> anyhow::Result<Result<(CookieJar, TmsResponse<()>), LoginRejection>> {
    // Portal login will always be the tms client id
    let client_id = String::from(CLIENT_ID_TMS);
    let configuration = &app_state.config;
    // the institution chosen on the landing page picks the login idp (and its IdP)
    let login_target = match get_login_target(
        configuration,
        &app_state.db_pool,
        form_data.institution.as_deref(),
    )
    .await
    {
        Ok(login_target) => login_target,
        Err(rejection) => return Ok(Err(rejection)),
    };

    // we will not use this value, but we need to make sure this redirect uri is in the database.
    let mut tx = app_state.db_pool.begin().await?;
    check_allowed_redirects(&mut tx, &client_id, &form_data.redirect_uri)
        .await
        .with_context(|| "Invalid login redirect")?;
    tx.commit().await?;

    let mut redirect_uri = Url::parse(&form_data.redirect_uri)?;
    if let Some(client_return_uri) = &form_data.client_return_uri {
        if let Some(return_client_name) = &form_data.client_name {
            // check to make sure the client name / return uri are in the allowed list
            let mut tx = app_state.db_pool.begin().await?;
            check_allowed_redirects_by_client_name(&mut tx, return_client_name, client_return_uri)
                .await
                .with_context(|| "Invalid redirect url for login")?;
            tx.commit().await?;
        } else {
            // if the client name was not specified, it's an error
            warn!("Unable to lookup client return uri.  No client name specified");
            return Ok(Err(LoginRejection::LoginFailed));
        }
        redirect_uri
            .query_pairs_mut()
            .append_pair("client_return_uri", client_return_uri.as_str());
    }
    // It's ok if they give the client_name but no return uri
    if let Some(client_name) = &form_data.client_name {
        redirect_uri
            .query_pairs_mut()
            .append_pair("client_name", client_name.as_str());
    }

    let oauth_state = OAuth2State {
        tms_identity: String::default(), // we don't have a tms identity at this point
        client_id,
        idp_id: login_target.idp.id.clone(),
        redirect_uri: redirect_uri.to_string(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            + 300,
        nonce: generate_nonce(),
        client_state: None,
        institution_id: Some(login_target.institution.id.clone()),
        login_started: Some(epoch_seconds_now()?),
    };

    let encoded_state = match encode_state(&app_state.config, oauth_state).await {
        Ok(state_string) => state_string,
        Err(error) => return Err(Internal(error.to_string()).into()),
    };

    let callback_url = &configuration
        .http_config
        .get_identity_provider_callback_url();
    let location = get_login_authorize_url(&login_target, callback_url, &encoded_state)?;

    let updated_jar = jar.add(
        Cookie::build((STATE_COOKIE_NAME, encoded_state))
            .path(ROOT_COOKIE_PATH)
            .http_only(true),
    );

    // This response goes to the browser, so it must not carry the idp client secret.  The
    // secret is only sent server to server, when the code is exchanged for a token.
    let mut headers = HashMap::new();
    headers.insert("location".to_string(), location.to_string());

    Ok(Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    )))
}

/*
Second step of logging out of an institution: Globus's sign-out page links here, and
this redirects to the institution's own IdP sign-out page so its session ends too.
Only urls from the approved institutions list are used, so this isn't an open redirect.
 */
#[debug_handler]
pub async fn logout_idp_handler(
    State(app_state): State<AppState>,
    Query(params): Query<LogoutIdpRequest>,
) -> anyhow::Result<TmsResponse<()>, AppError> {
    let http_config = &app_state.config.http_config;
    let location = match get_institution_by_id(&app_state.config, &params.institution).await {
        Ok(Institution {
            idp_logout_url: Some(idp_logout_url),
            ..
        }) => idp_logout_url,
        _ => http_config.make_tms_url(&http_config.base_url, SIGNIN_PATH),
    };
    let headers = HashMap::from([(LOCATION.to_string(), location)]);
    Ok(TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
        .headers(headers)
        .build())
}

/*
Log out.  This works without a valid token (e.g. one revoked by logging out in another tab),
so a user can always clear the token cookie and get to the sign-out pages.  It's a POST, so
another site can't log a user out with a link.
 */
#[debug_handler]
pub async fn logout_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> anyhow::Result<(CookieJar, TmsResponse<String>), AppError> {
    // the token is in the Authorization: Bearer header, or the token cookie (from the browser)
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(String::from)
        .or_else(|| {
            jar.get(TOKEN_COOKIE_NAME)
                .map(|cookie| cookie.value().to_string())
        });

    // revoke the token and find the identity provider's sign-out page (or the sign in page)
    let location = logout(&app_state.config, &app_state.db_pool, token.as_ref()).await?;

    // remove the token cookie
    let updated_jar =
        jar.remove(Cookie::build((TOKEN_COOKIE_NAME, String::from(""))).path(ROOT_COOKIE_PATH));

    // 303, so the browser follows it with a GET (a 307 would repeat the POST)
    let headers = HashMap::from([(LOCATION.to_string(), location)]);
    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::SEE_OTHER)
            .headers(headers)
            .build(),
    ))
}

/*
This method requires the user to be logged in (token in Authorization: Bearer token).
Information can be returned back from the verified/valid token such as the user's name.
 */
#[debug_handler]
pub async fn whoami_handler(
    State(app_state): State<AppState>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<WhoAmIResponse>, AppError> {
    let whoami_response = whoami(
        &app_state.config,
        &app_state.db_pool,
        &security_context.token,
    )
    .await?;
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(whoami_response.into())
        .build())
}

/*
OAuth2 Authorization code callback.  This code accepts the code from the login idp,
and exchanges it for the token from the login idp.  A TMS token is created and
returned.
 */
#[debug_handler]
pub async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<LoginCallbackQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // the state cookie set during the login process (validated in handle_callback)
    let cookie_state = jar
        .get(STATE_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    // Build a 'removal cookie' to remove the state
    let removal_cookie = Cookie::build((STATE_COOKIE_NAME, String::from("")))
        .path(ROOT_COOKIE_PATH)
        .http_only(true);

    // exchange code for token (state validated in handle_callback)
    let (token, expires_in, redirect_uri) = match handle_callback(
        &app_state.config,
        &app_state.db_pool,
        &query_params,
        cookie_state.as_deref(),
    )
    .await?
    {
        CallbackResult::LoggedIn {
            token,
            expires_in,
            redirect_uri,
        } => (token, expires_in, redirect_uri),
        // the login was rejected, cancelled, failed or expired: no tms token, back to the
        // sign in page, which explains why.
        CallbackResult::Rejected(rejection) => {
            let location = get_signin_url(&app_state.config, Some(rejection));
            let headers = HashMap::from([(LOCATION.to_string(), location)]);
            return Ok((
                jar.remove(removal_cookie),
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            ));
        }
    };

    let expiration_time = Utc::now() + TimeDelta::seconds(expires_in);

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), redirect_uri)]);

    // Build a new cookie and save it with the TMS token.
    let token_cookie = Cookie::build((TOKEN_COOKIE_NAME, token))
        .path(ROOT_COOKIE_PATH)
        .http_only(false)
        .secure(true)
        .expires(OffsetDateTime::from_unix_timestamp(
            expiration_time.timestamp(),
        )?)
        .build();

    let updated_jar = jar.add(token_cookie).remove(removal_cookie);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}

/*
Return a list of logon identity providers
 */
#[debug_handler]
pub async fn get_idp_handler(
    State(app_state): State<AppState>,
) -> anyhow::Result<TmsResponse<HashSet<IdentityProvider>>, AppError> {
    let idp_result = get_identity_providers(&app_state.db_pool).await?;
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(idp_result.iter().map(IdentityProvider::from).collect())
        .build())
}

/*
Return the approved institutions a user can log in through, fetched from tms_server.
 */
#[debug_handler]
pub async fn get_institutions_handler(
    State(app_state): State<AppState>,
) -> anyhow::Result<TmsResponse<Vec<LoginInstitution>>, AppError> {
    let institutions = get_institutions(&app_state.config).await?;
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(institutions.iter().map(LoginInstitution::from).collect())
        .build())
}
