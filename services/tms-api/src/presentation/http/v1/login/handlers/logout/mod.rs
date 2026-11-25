use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::header::{AUTHORIZATION, LOCATION};
use axum::http::{HeaderMap, StatusCode};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use serde::Deserialize;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{ROOT_COOKIE_PATH, TOKEN_COOKIE_NAME};

/*
Log out.  This works without a valid token (e.g. one revoked by logging out in another
tab), so a user can always clear the token cookie and get to the sign-out pages.  It's a
POST, so another site can't log a user out with a link.  The token is in the
Authorization: Bearer header, or the token cookie (from the browser).  The answer is a
303 to Globus's sign-out page (see LoginService::logout), so the browser follows it with
a GET.
 */
#[axum::debug_handler]
pub async fn logout(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(String::from)
        .or_else(|| jar.get(TOKEN_COOKIE_NAME).map(|cookie| cookie.value().to_string()));

    let location = app_state
        .login_service
        .logout(&app_state.config.oauth_config.login_oauth_provider, token.as_deref())
        .await?;

    // remove the token cookie
    let updated_jar =
        jar.remove(Cookie::build((TOKEN_COOKIE_NAME, String::from(""))).path(ROOT_COOKIE_PATH));

    let headers = HashMap::from([(LOCATION.to_string(), location)]);
    Ok((updated_jar, TmsResponse::builder(StatusCode::SEE_OTHER).headers(headers).build()))
}

#[derive(Debug, Deserialize)]
pub struct LogoutIdpRequest {
    pub institution: String,
}

/*
Second step of logging out of an institution: Globus's sign-out page links here, and this
redirects to the institution's own IdP sign-out page so its session ends too.  Only urls
from the institutions list are used, so this isn't an open redirect.
 */
#[axum::debug_handler]
pub async fn logout_idp(
    State(app_state): State<AppState>,
    Query(request): Query<LogoutIdpRequest>,
) -> anyhow::Result<TmsResponse<()>, AppError> {
    let location = app_state.login_service.get_idp_logout_url(&request.institution).await;
    let headers = HashMap::from([(LOCATION.to_string(), location)]);
    Ok(TmsResponse::builder(StatusCode::SEE_OTHER).headers(headers).build())
}
