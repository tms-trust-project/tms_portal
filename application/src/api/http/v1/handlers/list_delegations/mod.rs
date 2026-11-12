use crate::AppState;
use crate::api::http::v1::responses::delegations::Delegation;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::State;
use http::StatusCode;

#[axum::debug_handler]
pub async fn list_delegations(
    State(app_state): State<AppState>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Delegation>>, AppError> {
    let delegations = app_state
        .delegation_service
        .list_delegations(&security_context)
        .await?
        .into_iter()
        .map(Delegation::from)
        .collect();

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(delegations)
        .build())
}
