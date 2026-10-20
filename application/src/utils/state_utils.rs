use serde::de::DeserializeOwned;
use serde::Serialize;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use tms_lib::utils::jwt_encoder::JwtEncoderBuilder;
use crate::utils::configuration::Configuration;

const DEFAULT_STATE_ALGORITHM: &str = "RS256";

pub async fn encode_state<T>(
    configuration: &Configuration,
    oauth_state: T) ->
            anyhow::Result<String> where T:Serialize {
    let state_key = &configuration.state_key;
    let keys = &state_key.key;

    JwtEncoderBuilder::builder(
        oauth_state,
        &keys.jwt_private_key.as_bytes(),
        DEFAULT_STATE_ALGORITHM,
        keys.kid.as_str(),
    )
        .encode()
        .await
}

pub async fn decode_state<T>(
    configuration: &Configuration,
    state_string: &String) ->
            anyhow::Result<T> where T:DeserializeOwned {
    let state_key = &configuration.state_key;
    let keys = &state_key.key;
    // let decoding_key = Some(DecodingKey::from_rsa_pem(&keys.jwt_public_key.as_bytes())?);

    JwtDecoderBuilder::builder()
        .public_key(&keys.jwt_public_key.as_bytes())
        .validate_aud(false)
        .decode::<T>(&state_string)
        .await
}

