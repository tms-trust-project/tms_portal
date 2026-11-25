use axum::Router;
use axum::routing::{get, post};

use crate::AppState;
use crate::presentation::http::v1::login::handlers::{
    list_identity_providers::list_identity_providers, login::login, login_callback::login_callback,
    logout::logout, whoami::whoami,
};

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/login", get(login))
        .route("/logout", get(logout))
        .route("/login/whoami", get(whoami))
        .route("/login/callback", get(login_callback))
        .route("/login/idps", get(list_identity_providers))
}
