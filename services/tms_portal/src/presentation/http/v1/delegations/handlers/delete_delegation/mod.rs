use crate::AppState;
use crate::presentation::http::v1::delegations::requests::DeleteDelegationPathParams;
use crate::presentation::http::v1::delegations::responses::Delegation;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// Deletes the specified delegation for the 
#[utoipa::path(
    delete,
    path = "/delegations/{client_name}/{delegation_id}",
    tag = "Delegations",
    params(DeleteDelegationPathParams),
    responses(
        (
            status = 200,
            description = "Delegation created",
            body = Vec<Delegation>
        ),
        (
            status = 401,
            description = "Unauthorized",
            body = TmsErrorResponse
        ),
        (
            status = 500,
            description = "Internal error",
            body = TmsErrorResponse
        )
    )
)]
#[axum::debug_handler]
pub async fn delete_delegation(
    State(app_state): State<AppState>,
    Path(params): Path<DeleteDelegationPathParams>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Delegation>, AppError> {
    let delegation = app_state
        .delegation_service
        .delete_delegation(&security_context, &params.client_name, params.delegation_id)
        .await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(delegation.into())
        .build())
}
