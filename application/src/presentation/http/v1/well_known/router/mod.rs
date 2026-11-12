use axum::Router;
use axum::routing::get;

use crate::AppState;
use crate::presentation::http::v1::well_known::handlers::{
    get_jwks::get_jwks,
    get_openid_configuration::get_openid_configuration,
};

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/{client_id}/.well-known/openid-configuration", get(get_openid_configuration))
        .route("/{client_id}/jwks.json", get(get_jwks))
}
