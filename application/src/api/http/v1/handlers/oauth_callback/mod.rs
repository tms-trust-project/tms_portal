use std::collections::HashMap;

use axum::extract::{Query, State};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use http::StatusCode;
use http::header::LOCATION;
use tms_lib::utils::service_error::ServiceError::Unauthorized;

use crate::AppState;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::services::oauth2_service::process_authorization_callback;
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{
    AuthCodeQueryParams, ROOT_COOKIE_PATH, STATE_COOKIE_NAME,
};

// this should really only be called by the oauth redirect from an oauth provider (like globus, etc)
#[axum::debug_handler]
pub async fn oauth_callback(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(STATE_COOKIE_NAME) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    // handle the callback logic
    let authorization_callback_result = process_authorization_callback(
        &app_state.config,
        &app_state.db_pool,
        &state_cookie.value().to_owned(),
        &query_params.state,
        &query_params.code,
    )
    .await?;

    let removal_cookie = Cookie::build((STATE_COOKIE_NAME, String::from("")))
        .path(ROOT_COOKIE_PATH)
        .http_only(true);

    let updated_jar = jar.remove(removal_cookie);

    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), authorization_callback_result.location)]);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
