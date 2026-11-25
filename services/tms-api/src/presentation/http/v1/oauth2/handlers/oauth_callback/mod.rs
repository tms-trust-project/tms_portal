use std::collections::HashMap;

use axum::extract::{Query, State};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use http::StatusCode;
use http::header::LOCATION;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::oauth2_service::process_authorization_callback;
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{
    LoginCallbackQueryParams, ROOT_COOKIE_PATH, STATE_COOKIE_NAME,
};

// this should really only be called by the oauth redirect from an oauth provider (like globus, etc)
// It always redirects: to the client with a code, to the client with an RFC 6749 error, or
// (when the state can't be trusted) to the portal's sign in page.
#[axum::debug_handler]
pub async fn oauth_callback(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Query(query_params): Query<LoginCallbackQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Get the state cookie set during the login process.
    let cookie_state = jar
        .get(STATE_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned());

    // handle the callback logic
    let location = process_authorization_callback(
        &app_state.config,
        &app_state.db_pool,
        &app_state.login_service,
        &query_params,
        cookie_state.as_deref(),
    )
    .await;

    let removal_cookie = Cookie::build((STATE_COOKIE_NAME, String::from("")))
        .path(ROOT_COOKIE_PATH)
        .http_only(true);

    let updated_jar = jar.remove(removal_cookie);

    let headers: HashMap<String, String> = HashMap::from([(LOCATION.to_string(), location)]);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::SEE_OTHER)
            .headers(headers)
            .build(),
    ))
}
