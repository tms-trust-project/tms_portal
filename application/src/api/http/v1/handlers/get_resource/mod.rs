use std::collections::HashSet;

use axum::extract::{Path, State};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::AppState;
use crate::api::http::v1::responses::resources::Resource;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

#[axum::debug_handler]
pub async fn get_resource(
    State(_app_state): State<AppState>,
    Path((provider_id, provider_account_id)): Path<(String, String)>,
    JwtValidator(_security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<HashSet<Resource>>, AppError> {
    // Check that token is valid by getting claims
    let r1 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Stampede"),
        description: String::from("Stampede at TACC"),
        provider_id: provider_id.clone(),
        provider_account_id: provider_account_id.clone(),
        provider_name: String::from("TACC Resource Provider"),
    };

    let r2 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Vista"),
        description: String::from("Vista at TACC"),
        provider_id: provider_id.clone(),
        provider_account_id: provider_account_id.clone(),
        provider_name: String::from("TACC Resource Provider"),
    };

    let r3 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Frontera"),
        description: String::from("Frontera at TACC"),
        provider_id: provider_id.clone(),
        provider_account_id: provider_account_id.clone(),
        provider_name: String::from("TACC Resource Provider"),
    };

    let result = HashSet::from([r1, r2, r3]);
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}
