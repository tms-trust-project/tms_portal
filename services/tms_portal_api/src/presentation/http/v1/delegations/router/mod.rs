use axum::Router;
use axum::routing::{delete, get, post};

use crate::AppState;
use crate::presentation::http::v1::delegations::handlers::{
    add_delegation::add_delegation,
    delete_delegation::delete_delegation,
    list_delegations::list_delegations,
    list_delegations_by_client_id::list_delegations_by_client_id,
};

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/delegations", post(add_delegation))
        .route("/delegations", get(list_delegations))
        .route("/delegations/{client_name}", get(list_delegations_by_client_id))
        .route("/delegations/{client_name}/{delegation_id}", delete(delete_delegation))
}