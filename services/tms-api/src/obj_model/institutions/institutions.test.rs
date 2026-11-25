use tms_lib::utils::service_error::ServiceError;

use crate::obj_model::institutions::test_fixtures::InstitutionBuilder;
use crate::obj_model::institutions::{GlobusConfiguration, IdpConfig};

fn globus(domain: &str) -> IdpConfig {
    IdpConfig {
        globus: Some(GlobusConfiguration {
            redirect_idp_domain: domain.into(),
            idp_logout_url: format!("https://{}/logout", domain),
        }),
    }
}

fn assert_bad_request(result: Result<&str, ServiceError>, expected: &str) {
    match result {
        Err(ServiceError::BadRequest(message)) => assert_eq!(message, expected),
        other => panic!("expected a bad request, got {:?}", other),
    }
}

#[test]
fn an_enabled_institution_logs_in_through_its_globus_domain() {
    let institution = InstitutionBuilder::new()
        .with_id("utexas".into())
        .with_idp_config(globus("utexas.edu"))
        .build();

    assert_eq!(institution.globus_login_domain().unwrap(), "utexas.edu");
}

#[test]
fn a_disabled_institution_cant_be_used_to_log_in() {
    let institution = InstitutionBuilder::new()
        .with_id("offline".into())
        .with_enabled(false)
        .with_idp_config(globus("offline.edu"))
        .build();

    assert_bad_request(
        institution.globus_login_domain(),
        "Institution 'offline' is not enabled for login",
    );
}

#[test]
fn an_institution_without_globus_cant_be_used_to_log_in() {
    let institution = InstitutionBuilder::new()
        .with_id("noglobus".into())
        .with_idp_config(IdpConfig { globus: None })
        .build();

    assert_bad_request(
        institution.globus_login_domain(),
        "Institution 'noglobus' has no Globus login domain",
    );
}

#[test]
fn a_blank_globus_domain_cant_be_used_to_log_in() {
    let institution = InstitutionBuilder::new()
        .with_id("blank".into())
        .with_idp_config(globus("  "))
        .build();

    assert_bad_request(
        institution.globus_login_domain(),
        "Institution 'blank' has no Globus login domain",
    );
}

#[test]
fn an_identity_is_from_the_domain_after_its_last_at() {
    use crate::obj_model::institutions::identity_is_from_domain;

    assert!(identity_is_from_domain("jdoe@utexas.edu", "utexas.edu"));
    assert!(identity_is_from_domain("JDoe@UTexas.EDU", "utexas.edu"));
    assert!(identity_is_from_domain("odd@name@utexas.edu", "utexas.edu"));

    assert!(!identity_is_from_domain("jdoe@uchicago.edu", "utexas.edu"));
    assert!(!identity_is_from_domain("jdoe@cs.utexas.edu", "utexas.edu"));
    assert!(!identity_is_from_domain("jdoe@utexas.edu.evil.example", "utexas.edu"));
    assert!(!identity_is_from_domain("utexas.edu", "utexas.edu"));
    assert!(!identity_is_from_domain("", "utexas.edu"));
}
