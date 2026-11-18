use crate::AppState;
use crate::presentation::http::v1::clients::responses::{AllowedRedirect, Client};
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::State;
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// List all redirect URIs for a client
#[utoipa::path(
    get,
    path = "/clients/{client_id}/allowed-redirects",
    tag = "Clients",
    responses(
        (
            status = 200,
            description = "Allowed Redirects listed",
            body = Vec<AllowedRedirect>
        ),
        (
            status = 500,
            description = "Internal error",
            body = TmsErrorResponse
        )
    )
)]
#[axum::debug_handler]
pub async fn list_allowed_redirects_for_client(
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
