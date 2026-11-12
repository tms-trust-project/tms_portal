use crate::AppState;
use crate::presentation::http::v1::delegations::responses::Delegation;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;

#[axum::debug_handler]
pub async fn delete_delegation(
    State(app_state): State<AppState>,
    Path((client_name, delegation_id)): Path<(String, i32)>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Delegation>, AppError> {
    let delegation = app_state
        .delegation_service
        .delete_delegation(&security_context, &client_name, delegation_id)
        .await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(delegation.into())
        .build())
}
