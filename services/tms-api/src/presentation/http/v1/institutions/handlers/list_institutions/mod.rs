use crate::AppState;
use crate::presentation::http::v1::institutions::responses::Institution;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::State;
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// List all institutions.
#[utoipa::path(
    get,
    path = "/institutions",
    tag = "Institutions",
    responses(
        (
            status = 200,
            description = "Institutions listed",
            body = Vec<Institution>
        ),
        (
            status = 500,
            description = "Internal error",
            body = TmsErrorResponse
        )
    )
)]
#[axum::debug_handler]
pub async fn list_institutions(
    State(AppState { institution_service, .. }): State<AppState>,
    // JwtValidator(_security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Institution>>, AppError> {
    let institutions = institution_service
        .list_institutions()
        .await?
        .into_iter()
        .map(Institution::from)
        .collect();

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(institutions)
        .build())
}

#[cfg(test)]
#[path = "list_institutions.test.rs"]
mod list_institutions_test;
