use axum::Router;
use axum::routing::{delete, get, post};

use crate::AppState;
use crate::api::http::v1::handlers::{
    add_delegation::add_delegation, authorize_resource_provider::authorize_resource_provider,
    delete_delegation::delete_delegation, get_resource::get_resource,
    list_delegations::list_delegations,
    list_delegations_by_client_id::list_delegations_by_client_id,
    list_linked_resource_providers::list_linked_resource_providers,
    list_resource_providers::list_resource_providers,
    resource_provider_callback::resource_provider_callback,
    unlink_resource_provider::unlink_resource_provider,
};

pub async fn delegations_router() -> Router<AppState> {
    Router::new()
        .route("/delegations", post(add_delegation))
        .route("/delegations", get(list_delegations))
        .route("/delegations/{client_name}", get(list_delegations_by_client_id))
        .route("/delegations/{client_name}/{delegation_id}", delete(delete_delegation))
}

pub async fn resources_router() -> Router<AppState> {
    Router::new()
        .route("/resources/providers", get(list_resource_providers))
        .route("/resources/{provider_id}/{provider_account_id}", get(get_resource))
        .route(
            "/resources/providers/links/{resource_provider_link_id}",
            delete(unlink_resource_provider),
        )
        .route("/resources/providers/links", get(list_linked_resource_providers))
        .route("/resources/providers/authorize", get(authorize_resource_provider))
        .route("/resources/providers/callback", get(resource_provider_callback))
}
