use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;
use crate::presentation::http::v1::delegations::handlers::{
    add_delegation, delete_delegation, list_delegations, list_delegations_by_client_id,
};

pub async fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(
        add_delegation::add_delegation,
        list_delegations::list_delegations,
        list_delegations_by_client_id::list_delegations_by_client_id,
        delete_delegation::delete_delegation
    ))
}
