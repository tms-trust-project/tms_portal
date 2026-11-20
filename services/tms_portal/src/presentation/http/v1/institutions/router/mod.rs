use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;
use crate::presentation::http::v1::institutions::handlers::{
    get_institution_by_id, list_institutions,
};

pub async fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(
        get_institution_by_id::get_institution_by_id,
        list_institutions::list_institutions,
    ))
}
