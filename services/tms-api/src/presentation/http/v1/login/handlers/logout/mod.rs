use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use crate::utils::oauth2_authorization_code_utils::{ROOT_COOKIE_PATH, TOKEN_COOKIE_NAME};

#[axum::debug_handler]
pub async fn logout(
    State(AppState { login_service, .. }): State<AppState>,
    JwtValidator(security_context): JwtValidator,
    jar: CookieJar,
) -> anyhow::Result<(CookieJar, TmsResponse<String>), AppError> {
    let login_service = login_service;
    
    let redirect_url = login_service.logout(&security_context).await?;

    // remove the token cookie
    let updated_jar =
        jar.remove(Cookie::build((TOKEN_COOKIE_NAME, String::from(""))).path(ROOT_COOKIE_PATH));

    let mut headers = HashMap::new();
    headers.insert("location".to_string(), redirect_url.into_inner());
    
    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
