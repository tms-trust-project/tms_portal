use std::collections::HashMap;

use chrono::Utc;
use serde_json::{Value, json};

use super::{LoginRejection, check_login_identity, get_login_authorize_url};
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};

const LOGIN_STARTED: u64 = 1_800_000_000;

fn claims(value: Value) -> HashMap<String, Value> {
    serde_json::from_value(value).expect("the test claims should be an object")
}

// a fresh login as the username, with the sub every Globus id token has
fn fresh_login_as(username: &str) -> HashMap<String, Value> {
    claims(json!({
        "sub": "a-globus-uuid",
        "preferred_username": username,
        "last_authentication": LOGIN_STARTED + 30,
    }))
}

fn idp(identity_provider_type: IdentityProviderType) -> IdentityProvider {
    IdentityProvider {
        id: "globus".into(),
        name: "Globus".into(),
        client_id: "our-client-id".into(),
        client_secret: "our-client-secret".into(),
        identity_redirect_url: "https://auth.globus.org/v2/oauth2/authorize".into(),
        oauth2_token_url: "https://auth.globus.org/v2/oauth2/token".into(),
        oauth2_jwks_url: None,
        oidc_user_info_url: None,
        oauth2_public_key: None,
        scope: Some("openid profile email".into()),
        identity_provider_type,
        supports_login: true,
        resources_services_url: None,
        supports_resources: false,
        created: Utc::now(),
        updated: Utc::now(),
    }
}

fn query(url: &url::Url) -> HashMap<String, String> {
    url.query_pairs().into_owned().collect()
}

#[test]
fn accepts_a_fresh_identity_from_the_institutions_domain() {
    let result = check_login_identity(&fresh_login_as("jdoe@utexas.edu"), "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Ok(()));
}

#[test]
fn refuses_an_identity_from_another_domain() {
    let result =
        check_login_identity(&fresh_login_as("jdoe@uchicago.edu"), "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::UnapprovedInstitution));
}

#[test]
fn refuses_a_login_without_a_username() {
    let claims = claims(json!({"sub": "a-globus-uuid", "last_authentication": LOGIN_STARTED}));

    let result = check_login_identity(&claims, "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::UnapprovedInstitution));
}

#[test]
fn refuses_a_username_that_isnt_a_string() {
    let claims = claims(json!({"preferred_username": 42, "last_authentication": LOGIN_STARTED}));

    let result = check_login_identity(&claims, "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::UnapprovedInstitution));
}

#[test]
fn refuses_an_authentication_from_before_the_login_started() {
    let claims = claims(json!({
        "preferred_username": "jdoe@utexas.edu",
        "last_authentication": LOGIN_STARTED - 600,
    }));

    let result = check_login_identity(&claims, "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::StaleAuthentication));
}

#[test]
fn allows_a_few_seconds_of_clock_skew() {
    let claims = claims(json!({
        "preferred_username": "jdoe@utexas.edu",
        "last_authentication": LOGIN_STARTED - 3,
    }));

    assert_eq!(check_login_identity(&claims, "utexas.edu", LOGIN_STARTED), Ok(()));
}

#[test]
fn refuses_a_login_without_a_last_authentication() {
    let claims = claims(json!({"preferred_username": "jdoe@utexas.edu"}));

    let result = check_login_identity(&claims, "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::StaleAuthentication));
}

#[test]
fn a_globus_institution_login_requires_the_institutions_domain_and_a_new_login() {
    let url = get_login_authorize_url(
        &idp(IdentityProviderType::Globus),
        "https://tms.example.edu/v1/login/callback",
        "the-state",
        "the-nonce",
        Some("utexas.edu"),
    )
    .unwrap();

    let query = query(&url);
    assert!(url.as_str().starts_with("https://auth.globus.org/v2/oauth2/authorize?"));
    assert_eq!(query["response_type"], "code");
    assert_eq!(query["client_id"], "our-client-id");
    assert_eq!(query["redirect_uri"], "https://tms.example.edu/v1/login/callback");
    assert_eq!(query["state"], "the-state");
    assert_eq!(query["nonce"], "the-nonce");
    assert_eq!(query["scope"], "openid profile email");
    assert_eq!(query["prompt"], "login");
    assert_eq!(query["redirect_idp_domain"], "utexas.edu");
    assert_eq!(query["session_required_single_domain"], "utexas.edu");
}

#[test]
fn the_authorize_url_never_carries_the_client_secret() {
    let url = get_login_authorize_url(
        &idp(IdentityProviderType::Globus),
        "https://tms.example.edu/v1/login/callback",
        "the-state",
        "the-nonce",
        Some("utexas.edu"),
    )
    .unwrap();

    assert!(!url.as_str().contains("our-client-secret"));
}

#[test]
fn other_providers_get_no_globus_parameters() {
    let url = get_login_authorize_url(
        &idp(IdentityProviderType::TaccTapis),
        "https://tms.example.edu/v1/login/callback",
        "the-state",
        "the-nonce",
        Some("utexas.edu"),
    )
    .unwrap();

    let query = query(&url);
    assert!(!query.contains_key("prompt"));
    assert!(!query.contains_key("redirect_idp_domain"));
    assert!(!query.contains_key("session_required_single_domain"));
}
