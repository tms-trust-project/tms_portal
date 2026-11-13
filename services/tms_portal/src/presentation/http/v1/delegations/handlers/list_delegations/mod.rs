use crate::AppState;
use crate::presentation::http::v1::delegations::responses::Delegation;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::State;
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// List all the delegations issued by the the requesting client and tms identity
#[utoipa::path(
    get,
    path = "/delegations",
    tag = "Delegations",
    responses(
        (
            status = 200,
            description = "Delegations listed",
            body = Vec<Delegation>
        ),
        (
            status = 500,
            description = "Internal error",
            body = TmsErrorResponse
        )
    )
)]
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
