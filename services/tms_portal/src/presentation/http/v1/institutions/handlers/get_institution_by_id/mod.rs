use crate::AppState;
use crate::presentation::http::v1::institutions::responses::Institution;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use axum::extract::{Path, State};
use http::StatusCode;
use tms_lib::http::requests::TmsErrorResponse;

/// Find an institution by its institution identifier.
#[utoipa::path(
    get,
    path = "/institutions/{institution_id}",
    tag = "Institutions",
    params(
        ("institution_id" = String, Path, description = "Institution identifier")
    ),
    responses(
        (
            status = 200,
            description = "Institution found",
            body = Institution
        ),
        (
            status = 404,
            description = "Institution not found",
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
pub async fn get_institution_by_id(
    State(AppState { institution_service, .. }): State<AppState>,
    Path(institution_id): Path<String>,
    JwtValidator(_security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Institution>, AppError> {
    let institution = institution_service
        .find_institution_by_id(&institution_id)
        .await?
        .into();

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(institution)
        .build())
}

#[cfg(test)]
#[path = "get_institution_by_id.test.rs"]
mod get_institution_by_id_test;
