use std::collections::HashSet;

use axum::extract::State;
use reqwest::StatusCode;

use crate::AppState;
use crate::presentation::http::v1::resources::responses::ResourceAccountLink;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::resource_service::get_linked_resource_providers;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

#[axum::debug_handler]
pub async fn list_linked_resource_providers(
    State(app_state): State<AppState>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<HashSet<ResourceAccountLink>>, AppError> {
    let result = get_linked_resource_providers(&security_context, &app_state.db_pool).await?;
    let response = result.iter().map(ResourceAccountLink::from).collect();
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(response)
        .build())
}
