use axum::Router;
use axum::routing::{delete, get};

use crate::AppState;
use crate::presentation::http::v1::resources::handlers::{
    authorize_resource_provider::authorize_resource_provider, get_resource::get_resource,
    list_linked_resource_providers::list_linked_resource_providers,
    list_resource_providers::list_resource_providers,
    resource_provider_callback::resource_provider_callback,
    unlink_resource_provider::unlink_resource_provider,
};

pub async fn router() -> Router<AppState> {
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
