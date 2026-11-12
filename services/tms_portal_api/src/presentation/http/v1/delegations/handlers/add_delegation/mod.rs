use crate::AppState;
use crate::presentation::http::v1::delegations::responses::Delegation;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::Json;
use axum::extract::State;
use http::StatusCode;
use serde::Deserialize;

// params for tokens request
#[derive(Debug, Deserialize)]
pub struct AddDelegationRequestBody {
    client_name: String,
    resource_provider_id: String,
    resource_provider_account: String,
}

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
