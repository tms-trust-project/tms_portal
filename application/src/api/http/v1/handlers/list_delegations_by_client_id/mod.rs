use crate::AppState;
use crate::api::http::v1::responses::delegations::Delegation;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;

#[axum::debug_handler]
pub async fn list_delegations_by_client_id(
    State(app_state): State<AppState>,
    Path(client_name): Path<String>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Delegation>>, AppError> {
    let delegations: Vec<Delegation> = app_state
        .delegation_service
        .list_delegations_for_client(&security_context, &client_name)
        .await?
        .into_iter()
        .map(Delegation::from)
        .collect();

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(delegations)
        .build())
}
