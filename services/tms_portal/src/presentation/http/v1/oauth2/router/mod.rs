use axum::Router;
use axum::routing::{get, post};

use crate::AppState;
use crate::presentation::http::v1::oauth2::handlers::{
    authorize_oauth::authorize_oauth, create_oauth_token::create_oauth_token,
    oauth_callback::oauth_callback,
};

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/authorize", get(authorize_oauth))
        .route("/oauth2/tokens", post(create_oauth_token))
        .route("/oauth2/callback", get(oauth_callback))
}
