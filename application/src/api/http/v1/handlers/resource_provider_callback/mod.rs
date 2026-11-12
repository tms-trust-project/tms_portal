use std::collections::HashMap;
use std::string::ToString;

use axum::extract::{Query, State};
use axum::http::header::LOCATION;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use reqwest::StatusCode;
use tms_lib::utils::service_error::ServiceError::Unauthorized;

use crate::AppState;
use crate::api::http::v1::handlers::resource_provider_state::{
    RP_COOKIE_PATH, get_rp_state_cookie_name,
};
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::services::resource_service::get_resource_provider_token;
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{AuthCodeQueryParams, OAuth2State};
use crate::utils::state_utils::decode_state;

#[axum::debug_handler]
pub async fn resource_provider_callback(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    let decoded_state: OAuth2State =
        decode_state(&app_state.config.state_key, &query_params.state).await?;
    let resource_provider_id = decoded_state.idp_id;

    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(get_rp_state_cookie_name(&resource_provider_id).as_str())
    else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    if !&query_params.state.eq(state_cookie.value()) {
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    let decoded_state: OAuth2State =
        decode_state(&app_state.config.state_key, &state_cookie.value().to_string()).await?;

    get_resource_provider_token(
        &app_state.config,
        &app_state.db_pool,
        &resource_provider_id,
        &decoded_state.tms_identity,
        &query_params.code,
    )
    .await?;

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)]);

    let removal_cookie =
        Cookie::build((get_rp_state_cookie_name(&resource_provider_id), String::from("")))
            .path(RP_COOKIE_PATH);
    let updated_jar = jar.remove(removal_cookie);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
