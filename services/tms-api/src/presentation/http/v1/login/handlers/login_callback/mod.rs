use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use chrono::{TimeDelta, Utc};
use log::warn;
use time::OffsetDateTime;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::login_service::{CallbackResult, LoginRejection, get_signin_url};
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{
    LoginCallbackQueryParams, ROOT_COOKIE_PATH, STATE_COOKIE_NAME, TOKEN_COOKIE_NAME,
};

/*
OAuth2 Authorization code callback.  This code accepts the code from the login idp,
and exchanges it for the token from the login idp.  A TMS token is created and
returned.  A login that is refused, cancelled, fails or expires goes back to the sign in
page with ?error=<code> instead, which says why.  The state cookie is removed either way.
 */
#[axum::debug_handler]
pub async fn login_callback(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Query(query_params): Query<LoginCallbackQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    let cookie_state = jar.get(STATE_COOKIE_NAME).map(|cookie| cookie.value().to_owned());
    let removal_cookie = Cookie::build((STATE_COOKIE_NAME, String::from("")))
        .path(ROOT_COOKIE_PATH)
        .http_only(true);
    let jar = jar.remove(removal_cookie);

    // exchange code for token (state validated in handle_callback)
    let result = app_state
        .login_service
        .handle_callback(&app_state.config, &query_params, cookie_state.as_deref())
        .await
        .unwrap_or_else(|error| {
            warn!("Unable to complete a login: {error:#}");
            CallbackResult::Rejected(LoginRejection::LoginFailed)
        });

    let (token, expires_in, redirect_uri) = match result {
        CallbackResult::LoggedIn { token, expires_in, redirect_uri } => {
            (token, expires_in, redirect_uri)
        }
        CallbackResult::Rejected(rejection) => {
            let location = get_signin_url(&app_state.config.http_config, Some(rejection));
            let headers = HashMap::from([(LOCATION.to_string(), location)]);
            return Ok((
                jar,
                TmsResponse::builder(StatusCode::SEE_OTHER)
                    .headers(headers)
                    .build(),
            ));
        }
    };

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), redirect_uri)]);

    // Build a new cookie and save it with the TMS token.
    let token_cookie = Cookie::build((TOKEN_COOKIE_NAME, token))
        .path(ROOT_COOKIE_PATH)
        .http_only(false)
        .secure(true)
        .expires(OffsetDateTime::from_unix_timestamp(
            (Utc::now() + TimeDelta::seconds(expires_in)).timestamp(),
        )?)
        .build();

    Ok((
        jar.add(token_cookie),
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
