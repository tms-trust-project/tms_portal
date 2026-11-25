use std::collections::HashMap;

use url::Url;

use super::{AuthorizeCodeRequest, get_hosted_signin_url, get_oauth2_error_location};
use crate::obj_model::configuration::HttpConfig;
use crate::services::login_service::LoginRejection;

fn query(location: &str) -> HashMap<String, String> {
    Url::parse(location)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

#[test]
fn the_hosted_sign_in_page_carries_the_clients_authorize_request() {
    let http_config =
        HttpConfig { base_url: "https://tms.example.edu".into(), ..Default::default() };
    let state = Some(String::from("client-state"));
    let scope = Some(String::from("openid"));
    let request = AuthorizeCodeRequest {
        client_id: "other-portal",
        redirect_uri: "https://other.example.edu/callback?x=1",
        state: &state,
        scope: &scope,
        institution: &None,
    };

    let location = get_hosted_signin_url(&http_config, &request).unwrap();

    assert!(location.starts_with("https://tms.example.edu/?"), "{location}");
    let query = query(&location);
    assert_eq!(query["response_type"], "code");
    assert_eq!(query["client_id"], "other-portal");
    assert_eq!(query["redirect_uri"], "https://other.example.edu/callback?x=1");
    assert_eq!(query["state"], "client-state");
    assert_eq!(query["scope"], "openid");
    assert!(!query.contains_key("institution"));
}

#[test]
fn the_hosted_sign_in_page_leaves_out_a_missing_state_and_scope() {
    let http_config =
        HttpConfig { base_url: "https://tms.example.edu".into(), ..Default::default() };
    let request = AuthorizeCodeRequest {
        client_id: "other-portal",
        redirect_uri: "https://other.example.edu/callback",
        state: &None,
        scope: &None,
        institution: &None,
    };

    let query = query(&get_hosted_signin_url(&http_config, &request).unwrap());

    assert!(!query.contains_key("state"));
    assert!(!query.contains_key("scope"));
}

#[test]
fn a_refused_login_goes_back_to_the_client_with_an_oauth2_error_and_its_state() {
    let location = get_oauth2_error_location(
        "https://other.example.edu/callback",
        Some("client-state"),
        LoginRejection::UnapprovedInstitution,
    )
    .unwrap();

    assert!(location.starts_with("https://other.example.edu/callback?"), "{location}");
    let query = query(&location);
    assert_eq!(query["error"], "invalid_request");
    assert_eq!(query["error_description"], LoginRejection::UnapprovedInstitution.description());
    assert_eq!(query["state"], "client-state");
}

#[test]
fn a_cancelled_login_is_access_denied_for_the_client() {
    let location = get_oauth2_error_location(
        "https://other.example.edu/callback",
        None,
        LoginRejection::LoginCancelled,
    )
    .unwrap();

    let query = query(&location);
    assert_eq!(query["error"], "access_denied");
    assert!(!query.contains_key("state"));
}

#[test]
fn each_rejection_maps_to_an_rfc_6749_error() {
    let errors: Vec<&str> = [
        LoginRejection::InstitutionRequired,
        LoginRejection::UnapprovedInstitution,
        LoginRejection::StaleAuthentication,
        LoginRejection::LoginCancelled,
        LoginRejection::LoginExpired,
        LoginRejection::LoginFailed,
        LoginRejection::LoginUnavailable,
    ]
    .iter()
    .map(LoginRejection::oauth2_error)
    .collect();

    assert_eq!(
        errors,
        [
            "invalid_request",
            "invalid_request",
            "access_denied",
            "access_denied",
            "access_denied",
            "server_error",
            "temporarily_unavailable",
        ]
    );
}
