use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::utils::app_error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenidConfiguration {
    issuer: String,                 // "issuer": "https://auth.globus.org",
    authorization_endpoint: String, // "authorization_endpoint": "https://auth.globus.org/v2/oauth2/authorize",
    token_endpoint: String,         //"token_endpoint": "https://auth.globus.org/v2/oauth2/token",
    revocation_endpoint: String, // "revocation_endpoint": "https://auth.globus.org/v2/oauth2/token/revoke",
    jwks_uri: String,            // "jwks_uri": "https://auth.globus.org/jwk.json",
}
// more stuff we might add one day
// "response_types_supported": [
// "code",
// "token",
//     "token id_token",
//     "id_token"
//     ],
//     "id_token_signing_alg_values_supported": [
//     "RS512"
//     ],
// "scopes_supported": [
// "openid",
// "email",
// "profile"
// ],
// "token_endpoint_auth_methods_supported": [
// "client_secret_basic"
// ],
// "claims_supported": [
//     "at_hash",
//     "aud",
//     "email",
//     "exp",
//     "name",
//     "nonce",
//     "preferred_username",
//     "iat",
//     "iss",
//     "sub"
//     ],
//     "subject_types_supported" : ["public"]
// }

#[axum::debug_handler]
pub async fn get_openid_configuration(
    Path(client_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<TmsResponse<OpenidConfiguration>, AppError> {
    let http_config = &app_state.config.http_config;
    let openid_configuration = OpenidConfiguration {
        issuer: http_config.base_url.clone(),
        authorization_endpoint: http_config.get_authorization_url().clone(),
        token_endpoint: http_config.get_token_url().clone(),
        revocation_endpoint: http_config.get_revocation_url().clone(),
        jwks_uri: http_config.get_jwks_url().to_string(),
    };

    debug!("Client Id:{:?}", client_id);
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(openid_configuration)
        .build())
}
