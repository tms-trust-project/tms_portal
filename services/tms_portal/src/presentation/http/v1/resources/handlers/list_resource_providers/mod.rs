use std::collections::HashSet;

use axum::extract::{Query, State};
use reqwest::StatusCode;

use crate::AppState;
use crate::presentation::http::v1::resources::responses::ResourceProvider;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::resource_service::get_resource_providers;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use crate::utils::oauth2_authorization_code_utils::ListResourceProviderRequestParams;

#[axum::debug_handler]
pub async fn list_resource_providers(
    State(app_state): State<AppState>,
    query_params: Query<ListResourceProviderRequestParams>,
    JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<HashSet<ResourceProvider>>, AppError> {
    let linked_only = query_params.linked_only.unwrap_or(false);
    let result =
        get_resource_providers(&security_context, &app_state.db_pool, &linked_only).await?;
    let response = result.iter().map(ResourceProvider::from).collect();
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(response)
        .build())
}
