use axum::extract::State;
use axum::http::StatusCode;

use crate::AppState;
use crate::presentation::http::v1::login::responses::WhoAmIResponse;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::login_service;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

/*
This method requires the user to be logged in (token in Authorization: Bearer token).
Information can be returned back from the verified/valid token such as the user's name.
 */
#[axum::debug_handler]
pub async fn whoami(
    State(app_state): State<AppState>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<WhoAmIResponse>, AppError> {
    let whoami_response =
        login_service::whoami(&app_state.db_pool, &security_context.token).await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(whoami_response.into())
        .build())
}
