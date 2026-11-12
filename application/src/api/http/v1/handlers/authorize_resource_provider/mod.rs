use std::collections::HashMap;

use axum::extract::{Query, State};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use reqwest::StatusCode;

use crate::AppState;
use crate::api::http::v1::handlers::resource_provider_state::{
    RP_COOKIE_PATH, get_rp_state_cookie_name,
};
use crate::api::http::v1::requests::resources::ResourceProviderAuthorizeRequest;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::services::resource_service::get_authenticate_redirect_info;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use crate::utils::oauth2_authorization_code_utils::CLIENT_ID_TMS;

#[axum::debug_handler]
pub async fn authorize_resource_provider(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<ResourceProviderAuthorizeRequest>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // resource provider login will always be TMS client id
    let client_id = String::from(CLIENT_ID_TMS);

    let authorize_info = get_authenticate_redirect_info(
        &app_state.config,
        &security_context.tms_identity,
        &app_state.db_pool,
        &client_id,
        &query_params.provider_id,
        &query_params.redirect_url,
        &query_params.state,
    )
    .await?;

    let updated_jar = jar.add(
        Cookie::build((
            get_rp_state_cookie_name(&query_params.provider_id),
            authorize_info.encoded_state.clone(),
        ))
        .path(RP_COOKIE_PATH)
        .http_only(true),
    );

    let creds = format!("{}:{}", authorize_info.client_id, authorize_info.client_secret);
    let authorization = format!("Basic {}", BASE64_STANDARD.encode(&creds));
    let mut headers = HashMap::new();
    headers.insert("location".to_string(), authorize_info.identity_redirect_url.to_string());
    headers.insert("Authorization".to_string(), authorization);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
