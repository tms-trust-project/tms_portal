use std::collections::HashMap;

use chrono::Utc;
use serde_json::{Value, json};

use super::{
    LoginRejection, check_login_callback, check_login_identity, get_login_authorize_url,
    get_signin_url,
};
use crate::obj_model::configuration::{HttpConfig, StateKey};
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::obj_model::key::Key;
use crate::utils::oauth2_authorization_code_utils::{LoginCallbackQueryParams, OAuth2State};
use crate::utils::state_utils::encode_state;

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
    let result =
        check_login_identity(&fresh_login_as("jdoe@utexas.edu"), "utexas.edu", LOGIN_STARTED);

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
    assert!(
        url.as_str()
            .starts_with("https://auth.globus.org/v2/oauth2/authorize?")
    );
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

fn state_key() -> StateKey {
    StateKey {
        kid: "state-kid".into(),
        key: Key {
            kid: "state-kid".into(),
            jwt_public_key: include_str!("../../utils/test_keys/public.pem").into(),
            jwt_private_key: include_str!("../../utils/test_keys/private.pem").into(),
            ..Default::default()
        },
    }
}

// a signed login state that expires in the given number of seconds (negative: expired)
async fn signed_state(expires_in: i64) -> String {
    let now = Utc::now().timestamp();
    let state = OAuth2State {
        tms_identity: String::default(),
        idp_id: "globus".into(),
        client_id: "tms".into(),
        exp: (now + expires_in) as u64,
        redirect_uri: "https://tms.example.edu/".into(),
        client_state: None,
        nonce: 7,
        institution_id: Some("utexas".into()),
        login_started: now as u64,
    };
    encode_state(&state_key(), state)
        .await
        .expect("the test state should be signed")
}

fn callback(
    code: Option<&str>,
    state: Option<&str>,
    error: Option<&str>,
) -> LoginCallbackQueryParams {
    LoginCallbackQueryParams {
        code: code.map(String::from),
        state: state.map(String::from),
        error: error.map(String::from),
        error_description: None,
    }
}

async fn rejection(query: LoginCallbackQueryParams, cookie: Option<&str>) -> LoginRejection {
    check_login_callback(&state_key(), &query, cookie)
        .await
        .expect_err("the callback should be rejected")
}

#[tokio::test]
async fn a_callback_with_the_cookies_state_and_a_code_is_valid() {
    let state = signed_state(300).await;

    let (decoded, code) = check_login_callback(
        &state_key(),
        &callback(Some("the-code"), Some(&state), None),
        Some(&state),
    )
    .await
    .expect("the callback should be valid");

    assert_eq!(code, "the-code");
    assert_eq!(decoded.institution_id.as_deref(), Some("utexas"));
}

#[tokio::test]
async fn a_callback_without_a_state_cookie_has_expired() {
    let state = signed_state(300).await;

    let rejection = rejection(callback(Some("the-code"), Some(&state), None), None).await;

    assert_eq!(rejection, LoginRejection::LoginExpired);
}

#[tokio::test]
async fn a_callback_whose_state_isnt_the_cookies_has_expired() {
    let state = signed_state(300).await;
    // another login's state (it expires at a different time, so it's a different token)
    let other_state = signed_state(299).await;

    let rejection =
        rejection(callback(Some("the-code"), Some(&state), None), Some(&other_state)).await;

    assert_eq!(rejection, LoginRejection::LoginExpired);
}

#[tokio::test]
async fn a_callback_with_an_expired_state_has_expired() {
    let state = signed_state(-600).await;

    let rejection = rejection(callback(Some("the-code"), Some(&state), None), Some(&state)).await;

    assert_eq!(rejection, LoginRejection::LoginExpired);
}

#[tokio::test]
async fn a_callback_with_a_forged_state_has_expired() {
    let rejection =
        rejection(callback(Some("the-code"), Some("not-a-jwt"), None), Some("not-a-jwt")).await;

    assert_eq!(rejection, LoginRejection::LoginExpired);
}

#[tokio::test]
async fn access_denied_from_the_identity_provider_is_a_cancelled_login() {
    let state = signed_state(300).await;

    let rejection =
        rejection(callback(None, Some(&state), Some("access_denied")), Some(&state)).await;

    assert_eq!(rejection, LoginRejection::LoginCancelled);
}

#[tokio::test]
async fn another_error_from_the_identity_provider_is_a_failed_login() {
    let state = signed_state(300).await;

    let rejection =
        rejection(callback(None, Some(&state), Some("server_error")), Some(&state)).await;

    assert_eq!(rejection, LoginRejection::LoginFailed);
}

#[tokio::test]
async fn a_callback_without_a_code_or_an_error_is_a_failed_login() {
    let state = signed_state(300).await;

    let rejection = rejection(callback(None, Some(&state), None), Some(&state)).await;

    assert_eq!(rejection, LoginRejection::LoginFailed);
}

fn http_config(base_url: &str) -> HttpConfig {
    HttpConfig { base_url: base_url.into(), ..Default::default() }
}

#[test]
fn the_sign_in_page_is_the_portals_home_page() {
    assert_eq!(
        get_signin_url(&http_config("https://tms.example.edu"), None),
        "https://tms.example.edu/"
    );
}

#[test]
fn a_refused_login_goes_to_the_sign_in_page_with_its_error_code() {
    let url = get_signin_url(
        &http_config("https://tms.example.edu/"),
        Some(LoginRejection::UnapprovedInstitution),
    );

    assert_eq!(url, "https://tms.example.edu/?error=unapproved_institution");
}

#[test]
fn each_rejection_has_its_own_error_code() {
    let rejections = [
        LoginRejection::InstitutionRequired,
        LoginRejection::UnapprovedInstitution,
        LoginRejection::StaleAuthentication,
        LoginRejection::LoginCancelled,
        LoginRejection::LoginExpired,
        LoginRejection::LoginFailed,
        LoginRejection::LoginUnavailable,
    ];
    let codes: Vec<&str> = rejections.iter().map(LoginRejection::error_code).collect();

    assert_eq!(
        codes,
        [
            "institution_required",
            "unapproved_institution",
            "stale_authentication",
            "login_cancelled",
            "login_expired",
            "login_failed",
            "login_unavailable",
        ]
    );
}

// an id token whose primary identity isn't the institution's, with the given identity_set
fn login_with_identity_set(identity_set: Value) -> HashMap<String, Value> {
    claims(json!({
        "sub": "a-globus-uuid",
        "preferred_username": "0000-0002-1825-0097@orcid.org",
        "last_authentication": LOGIN_STARTED - 86_400,
        "identity_set": identity_set,
    }))
}

#[test]
fn accepts_a_fresh_linked_identity_from_the_institutions_domain() {
    let claims = login_with_identity_set(json!([
        {"username": "0000-0002-1825-0097@orcid.org", "last_authentication": LOGIN_STARTED - 86_400},
        {"username": "jdoe@utexas.edu", "last_authentication": LOGIN_STARTED + 20},
    ]));

    assert_eq!(check_login_identity(&claims, "utexas.edu", LOGIN_STARTED), Ok(()));
}

#[test]
fn refuses_when_no_linked_identity_is_from_the_domain() {
    let claims = login_with_identity_set(json!([
        {"username": "0000-0002-1825-0097@orcid.org", "last_authentication": LOGIN_STARTED + 20},
        {"username": "jdoe@uchicago.edu", "last_authentication": LOGIN_STARTED + 20},
    ]));

    let result = check_login_identity(&claims, "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::UnapprovedInstitution));
}

#[test]
fn refuses_a_linked_identity_from_the_domain_that_didnt_authenticate_for_this_login() {
    // the user authenticated with ORCID just now, but with utexas a day ago
    let claims = login_with_identity_set(json!([
        {"username": "0000-0002-1825-0097@orcid.org", "last_authentication": LOGIN_STARTED + 20},
        {"username": "jdoe@utexas.edu", "last_authentication": LOGIN_STARTED - 86_400},
    ]));

    let result = check_login_identity(&claims, "utexas.edu", LOGIN_STARTED);

    assert_eq!(result, Err(LoginRejection::StaleAuthentication));
}
