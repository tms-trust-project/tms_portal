use crate::AppState;
use crate::presentation::http::v1::clients::requests::ListAllowedRedirectsForClientPathParams;
use crate::presentation::http::v1::clients::responses::{AllowedRedirect, Client};
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// List all redirect URIs for a client
#[utoipa::path(
    get,
    path = "/clients/{client_id}/allowed-redirects",
    tag = "Clients",
    params(ListAllowedRedirectsForClientPathParams),
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
    Path(params): Path<ListAllowedRedirectsForClientPathParams>,
    JwtValidator(_security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<AllowedRedirect>>, AppError> {
    let allowed_redirects: Vec<AllowedRedirect> = client_service
        .list_allowed_redirects_for_client(&params.client_id)
        .await?
        .into_iter()
        .map(AllowedRedirect::from)
        .collect();

    Ok(TmsResponse::builder(StatusCode::OK).entity(allowed_redirects).build())
}
