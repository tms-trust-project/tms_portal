use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::LOCATION;

use crate::AppState;
use crate::presentation::http::v1::login::requests::LogoutIdpRequest;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;

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
    let location = app_state
        .login_service
        .get_idp_logout_url(&request.institution)
        .await;
    let headers = HashMap::from([(LOCATION.to_string(), location)]);
    Ok(TmsResponse::builder(StatusCode::SEE_OTHER)
        .headers(headers)
        .build())
}
