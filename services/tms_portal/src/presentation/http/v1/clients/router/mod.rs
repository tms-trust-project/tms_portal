use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;
use crate::presentation::http::v1::clients::handlers::{
    get_client_by_id,
    list_clients,
};

pub async fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_clients::list_clients))
        .routes(routes!(get_client_by_id::get_client_by_id))
}
