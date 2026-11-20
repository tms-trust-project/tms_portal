use crate::AppState;
use crate::presentation::http::v1::delegations::requests::ListDelegationByClientPathParams;
use crate::presentation::http::v1::delegations::responses::Delegation;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// List all the delegations issued for a tms identity by the client identifier provided in the request.
/// Normal clients can only list their own delegations. The TMS client can list delegations for any client.
#[utoipa::path(
    get,
    path = "/delegations/{client_name}",
    tag = "Delegations",
    params(ListDelegationByClientPathParams),
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
pub async fn list_delegations_by_client_id(
    State(app_state): State<AppState>,
    Path(params): Path<ListDelegationByClientPathParams>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Delegation>>, AppError> {
    let delegations: Vec<Delegation> = app_state
        .delegation_service
        .list_delegations_for_client(&security_context, &params.client_name)
        .await?
        .into_iter()
        .map(Delegation::from)
        .collect();

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(delegations)
        .build())
}
