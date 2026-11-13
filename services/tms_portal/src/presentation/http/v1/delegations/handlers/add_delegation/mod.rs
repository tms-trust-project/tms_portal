use crate::AppState;
use crate::presentation::http::v1::delegations::responses::{AddDelegationResponse, Delegation};
use crate::presentation::http::v1::delegations::requests::AddDelegationRequestBody;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;

use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

use tms_lib::http::requests::TmsErrorResponse;

use axum::Json;
use axum::extract::State;
use http::StatusCode;

/// Create a delegation after verifying that the authenticated user recently logged in to the
/// requested resource provider.
#[utoipa::path(
    post,
    path = "/delegations",
    tag = "Delegations",
    request_body(
        content = AddDelegationRequestBody,
        description = "The client and resource-provider account receiving delegated access",
        content_type = "application/json"
    ),
    responses(
        (
            status = 200,
            description = "Delegation created",
            body = AddDelegationResponse
        ),
        (
            status = 401,
            description = "The client is not authorized or the resource-provider login is stale",
            body = TmsErrorResponse
        ),
        (
            status = 500,
            description = "The delegation could not be created",
            body = TmsErrorResponse
        )
    )
)]
#[axum::debug_handler]
pub async fn add_delegation(
    State(app_state): State<AppState>,
    JwtValidator(security_context): JwtValidator,
    Json(body): Json<AddDelegationRequestBody>,
) -> anyhow::Result<TmsResponse<Delegation>, AppError> {
    let delegation = app_state
        .delegation_service
        .add_delegation(
            &security_context,
            &body.client_name,
            &body.resource_provider_id,
            &body.resource_provider_account,
        )
        .await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(delegation.into())
        .build())
}

#[cfg(test)]
#[path = "add_delegation.test.rs"]
mod add_delegation_test;
