use serde_json::Value;
use tms_lib::utils::jwt_encoder::JwtEncoderBuilder;
use tms_lib::utils::service_error::ServiceError;

use super::{TmsTokenClaims, decode_tms_token, get_signing_key};
use crate::obj_model::configuration::JwtConfig;
use crate::obj_model::identity_provider::IdentityProviderType;
use crate::obj_model::key::Key;

const PRIVATE_KEY: &str = include_str!("test_keys/private.pem");
const PUBLIC_KEY: &str = include_str!("test_keys/public.pem");
const OTHER_PUBLIC_KEY: &str = include_str!("test_keys/other_public.pem");
const KID: &str = "tms-signing-key";

fn jwt_config(public_key: &str) -> JwtConfig {
    JwtConfig {
        default_expiration_minutes: 60,
        signing_key_kid: KID.into(),
        key: Key {
            kid: KID.into(),
            jwt_public_key: public_key.into(),
            jwt_private_key: PRIVATE_KEY.into(),
            ..Default::default()
        },
    }
}

fn claims() -> TmsTokenClaims {
    TmsTokenClaims {
        jti: Value::from("a-jti"),
        iss: Value::from("https://tms.example.edu"),
        sub: Value::from("a-globus-uuid@globus"),
        aud: Value::from("tms"),
        tms_token_type: Value::from("access"),
        tms_username: Value::from("a-globus-uuid@globus"),
        tms_grant_type: Value::from("password"),
        tms_account_type: Value::from("user"),
        tms_name: None,
        tms_idp_display_name: None,
        tms_idp_provider: IdentityProviderType::Globus,
        tms_organization: None,
        // milliseconds, as get_tms_token_claims sets it
        exp: Value::from(4_102_444_800_000_u64),
    }
}

async fn sign(kid: &str) -> String {
    JwtEncoderBuilder::builder(claims(), PRIVATE_KEY.as_bytes(), "RS256", kid)
        .encode()
        .await
        .expect("the test token should be signed")
}

fn assert_unauthorized(error: anyhow::Error) {
    assert!(
        matches!(error.downcast_ref::<ServiceError>(), Some(ServiceError::Unauthorized(_))),
        "expected unauthorized, got {:#}",
        error
    );
}

#[test]
fn the_configured_kid_gets_the_signing_key() {
    let config = jwt_config(PUBLIC_KEY);

    let key = get_signing_key(&config, Some(KID)).expect("the signing key should be found");

    assert_eq!(key.kid, KID);
}

#[test]
fn another_kid_is_refused() {
    assert_unauthorized(get_signing_key(&jwt_config(PUBLIC_KEY), Some("other-kid")).unwrap_err());
}

#[test]
fn a_token_without_a_kid_is_refused() {
    assert_unauthorized(get_signing_key(&jwt_config(PUBLIC_KEY), None).unwrap_err());
}

#[tokio::test]
async fn decodes_a_token_signed_with_the_signing_key() {
    let token = sign(KID).await;

    let claims = decode_tms_token(&jwt_config(PUBLIC_KEY), &token)
        .await
        .expect("the token should decode");

    assert_eq!(claims.get_sub().unwrap(), "a-globus-uuid@globus");
    assert_eq!(claims.get_aud().unwrap(), "tms");
}

#[tokio::test]
async fn refuses_a_token_with_an_unknown_kid() {
    let token = sign("other-kid").await;

    assert_unauthorized(decode_tms_token(&jwt_config(PUBLIC_KEY), &token).await.unwrap_err());
}

#[tokio::test]
async fn refuses_a_token_whose_signature_does_not_match() {
    let token = sign(KID).await;

    assert_unauthorized(decode_tms_token(&jwt_config(OTHER_PUBLIC_KEY), &token).await.unwrap_err());
}

#[tokio::test]
async fn refuses_a_token_that_is_not_a_jwt() {
    assert_unauthorized(decode_tms_token(&jwt_config(PUBLIC_KEY), "not-a-jwt").await.unwrap_err());
}
