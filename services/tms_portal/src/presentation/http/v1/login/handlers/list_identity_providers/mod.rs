use std::collections::HashSet;

use axum::extract::State;
use axum::http::StatusCode;

use crate::AppState;
use crate::presentation::http::v1::login::responses::IdentityProvider;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;

/*
Return a list of logon identity providers
 */
#[axum::debug_handler]
pub async fn list_identity_providers(
    State(AppState { login_service, .. }): State<AppState>,
) -> anyhow::Result<TmsResponse<HashSet<IdentityProvider>>, AppError> {

    let result = login_service.list_identity_providers().await?;
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(result.iter().map(IdentityProvider::from).collect())
        .build())
}
