use axum::extract::{Path, State};
use axum::http::StatusCode;
use jwtk::PublicKeyToJwk;
use jwtk::jwk::JwkSet;
use jwtk::rsa::RsaPublicKey;

use crate::AppState;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::infra::persistence::client_repository::get_client_by_id;
use crate::utils::app_error::AppError;

pub async fn get_jwks(
    Path(client_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<TmsResponse<JwkSet>, AppError> {
    let mut tx = app_state.db_pool.begin().await?;
    // lookup client ot make sure it's a real client id
    let _client = get_client_by_id(&mut tx, &client_id).await?;
    let jwt_config = &app_state.config.jwt_config;
    let key = &jwt_config.key;
    tx.commit().await?;

    let public_key_string = &key.jwt_public_key;
    let rpk = RsaPublicKey::from_pem(public_key_string.as_bytes(), None)?;
    // let encoding_key = EncodingKey::from_rsa_pem(&public_key_string.as_bytes())?;
    // let jwk = Jwk::from_encoding_key(&encoding_key, RS256)?;
    // let jwks = JwkSet { keys: vec![jwk] };

    let jwk = rpk.public_key_to_jwk()?;
    let keys = vec![jwk];
    let jwks = JwkSet { keys };
    // let key = serde_json::to_string(&jwks)?;
    // let key = "stuff".to_string();
    Ok(TmsResponse::builder(StatusCode::OK).entity(jwks).build())
}
