use crate::AppState;
use crate::presentation::http::v1::clients::responses::Client;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::State;
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// List all clients
#[utoipa::path(
    get,
    path = "/clients",
    tag = "Clients",
    responses(
        (
            status = 200,
            description = "Clients listed",
            body = Vec<Client>
        ),
        (
            status = 500,
            description = "Internal error",
            body = TmsErrorResponse
        )
    )
)]
#[axum::debug_handler]
pub async fn list_clients(
    State(AppState { client_service, .. }): State<AppState>,
    JwtValidator(_security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Client>>, AppError> {
    let clients = client_service
        .list_clients()
        .await?
        .into_iter()
        .map(Client::from)
        .collect();

    Ok(TmsResponse::builder(StatusCode::OK).entity(clients).build())
}
