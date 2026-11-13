use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use chrono::{TimeDelta, Utc};
use time::OffsetDateTime;
use tms_lib::utils::service_error::ServiceError::Unauthorized;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::login_service::handle_callback;
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{
    AuthCodeQueryParams, OAuth2State, ROOT_COOKIE_PATH, STATE_COOKIE_NAME, TOKEN_COOKIE_NAME,
};
use crate::utils::state_utils::decode_state;

/*
OAuth2 Authorization code callback.  This code accepts the code from the login idp,
and exchanges it for the token from the login idp.  A TMS token is created and
returned.
 */
#[axum::debug_handler]
pub async fn login_callback(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(STATE_COOKIE_NAME) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    // exchange code for token (state validated in handle_callback)
    let (token, expires_in) = handle_callback(
        &app_state.config,
        &app_state.db_pool,
        &query_params.state,
        &query_params.code,
        &state_cookie.value().to_owned(),
    )
    .await?;

    let expiration_time = Utc::now() + TimeDelta::seconds(expires_in);

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let decoded_state: OAuth2State =
        decode_state(&app_state.config.state_key, &state_cookie.value().to_owned()).await?;
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)]);

    // Build a new cookie and save it with the TMS token.
    let token_cookie = Cookie::build((TOKEN_COOKIE_NAME, token))
        .path(ROOT_COOKIE_PATH)
        .http_only(false)
        .secure(true)
        .expires(OffsetDateTime::from_unix_timestamp(expiration_time.timestamp())?)
        .build();

    // Build a 'removal cookie' to remove the state
    let removal_cookie = Cookie::build((STATE_COOKIE_NAME, String::from("")))
        .path(ROOT_COOKIE_PATH)
        .http_only(true);

    let updated_jar = jar.add(token_cookie).remove(removal_cookie);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
