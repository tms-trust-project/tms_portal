use crate::AppState;
use crate::presentation::http::v1::delegations::responses::Delegation;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
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
