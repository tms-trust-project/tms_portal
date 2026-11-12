use axum::extract::{Path, State};
use reqwest::StatusCode;

use crate::AppState;
use crate::api::http::v1::responses::resources::ResourceAccountLogin;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::services::resource_service;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

#[axum::debug_handler]
pub async fn unlink_resource_provider(
    State(app_state): State<AppState>,
    Path(resource_provider_link_id): Path<i64>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<ResourceAccountLogin>, AppError> {
    let result = resource_service::unlink_resource_provider(
        &security_context,
        &app_state.db_pool,
        &resource_provider_link_id,
    )
    .await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(result.into())
        .build())
}
