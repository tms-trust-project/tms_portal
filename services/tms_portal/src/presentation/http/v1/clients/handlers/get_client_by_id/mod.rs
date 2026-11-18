use crate::AppState;
use crate::presentation::http::v1::clients::responses::ClientDetails;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// Find a client by its client identifier.
#[utoipa::path(
    get,
    path = "/clients/{client_id}",
    tag = "Clients",
    params(
        ("client_id" = String, Path, description = "Client identifier")
    ),
    responses(
        (
            status = 200,
            description = "Client found",
            body = ClientDetails
        ),
        (
            status = 404,
            description = "Client not found",
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
pub async fn get_client_by_id(
    State(AppState { client_service, .. }): State<AppState>,
    Path(client_id): Path<String>,
    JwtValidator(_security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<ClientDetails>, AppError> {
    let details = client_service.find_client_by_id(&client_id).await?.into();

    Ok(TmsResponse::builder(StatusCode::OK).entity(details).build())
}
