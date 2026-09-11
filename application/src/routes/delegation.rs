use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::{delete, get, post};
use http::StatusCode;
use serde::Deserialize;
use crate::AppState;
use crate::routes::api_obj_model::delegations::Delegation;
use crate::routes::api_obj_model::tms_response::TmsResponse;
use crate::services::delegation_service::{add_delegation, delete_delegation, get_delegations};
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

// params for tokens request
#[derive(Debug, Deserialize)]
pub struct AddDelegationRequest {
    client_name : String,
    resource_provider_id: String,
    resource_provider_account: String,
}
#[derive(Debug, Deserialize)]
pub struct GetDelegationQueryParams {
    client_name : Option<String>,
}
pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/delegations", post(add_delegation_handler))
        .route("/delegations/", get(get_delegations_handler))
        .route("/delegations/{client_name}", get(get_delegations_handler_with_client))
        .route("/delegations/{client_name}/{delegation_id}", delete(delete_delegations_handler))
}
#[axum::debug_handler]
pub async fn add_delegation_handler(State(app_state): State<AppState>,
                                    JwtValidator(security_context): JwtValidator,
                                    Json(add_delegation_request): Json<AddDelegationRequest>,
                                    ) -> anyhow::Result<TmsResponse<Delegation>, AppError> {
    let delegation = add_delegation(&app_state.db_pool, &security_context,
        &add_delegation_request.client_name, &add_delegation_request.resource_provider_id,
        &add_delegation_request.resource_provider_account).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(delegation.into()).build())
}
#[axum::debug_handler]
pub async fn get_delegations_handler_with_client(State(app_state): State<AppState>,
                                     Path(client_name):Path<String>,
                                     JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Delegation>>, AppError> {
    let delegations = get_delegations(&app_state.db_pool, &security_context, &Some(client_name)).await?;
    let mut result_delegations = Vec::with_capacity(delegations.len());
    for delegation in delegations {
        result_delegations.push(delegation.into());
    }
    Ok(TmsResponse::builder(StatusCode::OK).entity(result_delegations).build())
}
#[axum::debug_handler]
pub async fn get_delegations_handler(State(app_state): State<AppState>,
                                                 JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Vec<Delegation>>, AppError> {
    let delegations = get_delegations(&app_state.db_pool, &security_context, &None).await?;
    let mut result_delegations = Vec::with_capacity(delegations.len());
    for delegation in delegations {
        result_delegations.push(delegation.into());
    }
    Ok(TmsResponse::builder(StatusCode::OK).entity(result_delegations).build())
}
#[axum::debug_handler]
pub async fn delete_delegations_handler(State(app_state): State<AppState>, Path((client_name, delegation_id)): Path<(String, i32)>,
                                     JwtValidator(security_context): JwtValidator,
) -> anyhow::Result<TmsResponse<Delegation>, AppError> {
    let delegation = delete_delegation(&app_state.db_pool, &security_context, &client_name, delegation_id).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(delegation.into()).build())
}
