use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use super::JwtDecoderBuilder;
use crate::utils::jwt_encoder::JwtEncoderBuilder;

const PRIVATE_KEY: &str = include_str!("test_keys/private.pem");
const PUBLIC_KEY: &str = include_str!("test_keys/public.pem");

// a token that expires in 2100, with the given claims
async fn token(claims: Value) -> String {
    let mut claims = claims;
    claims["exp"] = json!(4_102_444_800_u64);
    JwtEncoderBuilder::builder(claims, PRIVATE_KEY.as_bytes(), "RS256", "a-kid")
        .encode()
        .await
        .expect("the test token should be signed")
}

fn decoder() -> JwtDecoderBuilder {
    JwtDecoderBuilder::builder().public_key(PUBLIC_KEY.as_bytes())
}

fn audience(aud: &str) -> HashSet<String> {
    HashSet::from([String::from(aud)])
}

async fn decode(builder: JwtDecoderBuilder, token: &str) -> anyhow::Result<HashMap<String, Value>> {
    builder.decode(token).await
}

#[tokio::test]
async fn accepts_a_token_for_the_audience() {
    let token = token(json!({"aud": "our-client"})).await;

    let claims = decode(decoder().audience(audience("our-client")), &token)
        .await
        .unwrap();

    assert_eq!(claims["aud"], "our-client");
}

#[tokio::test]
async fn refuses_a_token_for_another_audience() {
    let token = token(json!({"aud": "another-client"})).await;

    assert!(
        decode(decoder().audience(audience("our-client")), &token)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn refuses_a_token_without_an_aud_when_an_audience_is_given() {
    let token = token(json!({"sub": "someone"})).await;

    assert!(
        decode(decoder().audience(audience("our-client")), &token)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn refuses_a_token_with_an_aud_when_no_audience_is_given() {
    let token = token(json!({"aud": "some-client"})).await;

    assert!(decode(decoder(), &token).await.is_err());
}

#[tokio::test]
async fn ignores_the_aud_without_aud_validation() {
    let token = token(json!({"aud": "some-client"})).await;

    assert!(decode(decoder().validate_aud(false), &token).await.is_ok());
}

#[tokio::test]
async fn accepts_a_token_from_the_issuer() {
    let token = token(json!({"iss": "https://auth.example.org"})).await;

    let result = decode(
        decoder()
            .validate_aud(false)
            .issuer("https://auth.example.org"),
        &token,
    );

    assert!(result.await.is_ok());
}

#[tokio::test]
async fn refuses_a_token_from_another_issuer() {
    let token = token(json!({"iss": "https://other.example.org"})).await;

    let result = decode(
        decoder()
            .validate_aud(false)
            .issuer("https://auth.example.org"),
        &token,
    );

    assert!(result.await.is_err());
}

#[tokio::test]
async fn refuses_a_token_without_an_iss_when_an_issuer_is_given() {
    let token = token(json!({"sub": "someone"})).await;

    let result = decode(
        decoder()
            .validate_aud(false)
            .issuer("https://auth.example.org"),
        &token,
    );

    assert!(result.await.is_err());
}

#[tokio::test]
async fn refuses_an_expired_token() {
    let token = JwtEncoderBuilder::builder(
        json!({"exp": 1_000_000_000_u64}),
        PRIVATE_KEY.as_bytes(),
        "RS256",
        "a-kid",
    )
    .encode()
    .await
    .unwrap();

    assert!(decode(decoder().validate_aud(false), &token).await.is_err());
}
