use std::collections::HashMap;

use serde_json::Value;
use tms_lib::utils::service_error::ServiceError;

use super::check_institution_identity;

fn claims(preferred_username: Option<&str>) -> HashMap<String, Value> {
    let mut claims = HashMap::from([("sub".to_string(), Value::from("a-globus-uuid"))]);
    if let Some(username) = preferred_username {
        claims.insert("preferred_username".to_string(), Value::from(username));
    }
    claims
}

fn unauthorized_message(result: anyhow::Result<()>) -> String {
    let error = result.expect_err("the login should be refused");
    match error.downcast_ref::<ServiceError>() {
        Some(ServiceError::Unauthorized(message)) => message.clone(),
        _ => panic!("expected unauthorized, got {:#}", error),
    }
}

#[test]
fn accepts_an_identity_from_the_institutions_domain() {
    assert!(check_institution_identity(&claims(Some("jdoe@utexas.edu")), "utexas.edu").is_ok());
}

#[test]
fn refuses_an_identity_from_another_domain() {
    let message = unauthorized_message(check_institution_identity(
        &claims(Some("jdoe@uchicago.edu")),
        "utexas.edu",
    ));

    assert!(message.contains("'jdoe@uchicago.edu'"), "{}", message);
    assert!(message.contains("isn't an identity from utexas.edu"), "{}", message);
}

#[test]
fn refuses_a_login_without_a_username() {
    let message = unauthorized_message(check_institution_identity(&claims(None), "utexas.edu"));

    assert_eq!(message, "The login returned no username, so it can't be checked against utexas.edu");
}

#[test]
fn refuses_a_username_that_isnt_a_string() {
    let mut claims = claims(None);
    claims.insert("preferred_username".to_string(), Value::from(42));

    unauthorized_message(check_institution_identity(&claims, "utexas.edu"));
}
