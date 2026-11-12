use std::collections::HashSet;

use axum::extract::State;
use axum::http::StatusCode;

use crate::AppState;
use crate::api::http::v1::responses::login::IdentityProvider;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::services::login_service::get_identity_providers;
use crate::utils::app_error::AppError;

/*
Return a list of logon identity providers
 */
#[axum::debug_handler]
pub async fn list_identity_providers(
    State(app_state): State<AppState>,
) -> anyhow::Result<TmsResponse<HashSet<IdentityProvider>>, AppError> {
    let idp_result = get_identity_providers(&app_state.db_pool).await?;
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(idp_result.iter().map(IdentityProvider::from).collect())
        .build())
}
