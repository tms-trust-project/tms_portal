use std::sync::Arc;

use crate::infra::persistence::institution_repository::test_fixtures::MockInstitutionRepo;
use crate::obj_model::institutions::test_fixtures::InstitutionBuilder;
use crate::obj_model::institutions::{GlobusConfiguration, IdpConfig};
use crate::services::institution_service::InstitutionService;

#[tokio::test]
async fn lists_institutions() {
    let first = InstitutionBuilder::new()
        .with_id("first-institution".into())
        .with_enabled(true)
        .with_idp_config(IdpConfig {
            globus: Some(GlobusConfiguration {
                redirect_idp_domain: "first.example.edu".into(),
                idp_logout_url: "https://first.example.edu/logout".into(),
            }),
        })
        .build();
    let second = InstitutionBuilder::new()
        .with_id("second-institution".into())
        .with_enabled(false)
        .build();
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![first, second])));

    let institutions = service
        .list_institutions()
        .await
        .expect("institutions should be listed");

    assert_eq!(institutions.len(), 2);
    assert_eq!(institutions[0].id, "first-institution");
    assert!(institutions[0].enabled);
    assert_eq!(
        institutions[0]
            .idp_config
            .globus
            .as_ref()
            .expect("the first institution should have Globus configuration")
            .redirect_idp_domain,
        "first.example.edu"
    );
    assert_eq!(institutions[1].id, "second-institution");
    assert!(!institutions[1].enabled);
    assert!(institutions[1].idp_config.globus.is_none());
}

#[tokio::test]
async fn finds_institution_by_id() {
    let institution = InstitutionBuilder::new()
        .with_id("requested-institution".into())
        .with_name("Requested Institution".into())
        .with_enabled(false)
        .with_idp_config(IdpConfig {
            globus: Some(GlobusConfiguration {
                redirect_idp_domain: "requested.example.edu".into(),
                idp_logout_url: "https://requested.example.edu/logout".into(),
            }),
        })
        .build();
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![institution])));

    let institution = service
        .find_institution_by_id("requested-institution")
        .await
        .expect("the institution should be found");

    assert_eq!(institution.id, "requested-institution");
    assert_eq!(institution.name, "Requested Institution");
    assert!(!institution.enabled);
    let globus = institution
        .idp_config
        .globus
        .expect("the institution should have Globus configuration");
    assert_eq!(globus.redirect_idp_domain, "requested.example.edu");
    assert_eq!(globus.idp_logout_url, "https://requested.example.edu/logout");
}

fn utexas() -> crate::obj_model::institutions::Institution {
    InstitutionBuilder::new()
        .with_id("utexas".into())
        .with_idp_config(IdpConfig {
            globus: Some(GlobusConfiguration {
                redirect_idp_domain: "utexas.edu".into(),
                idp_logout_url: "https://enterprise.login.utexas.edu/idp/profile/Logout".into(),
            }),
        })
        .build()
}

// the message of a bad request, or a panic naming what came back instead
fn bad_request_message(error: anyhow::Error) -> String {
    match error.downcast_ref::<tms_lib::utils::service_error::ServiceError>() {
        Some(tms_lib::utils::service_error::ServiceError::BadRequest(message)) => message.clone(),
        _ => panic!("expected a bad request, got {:#}", error),
    }
}

#[tokio::test]
async fn gives_the_globus_login_domain_of_an_enabled_institution() {
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![utexas()])));

    let domain = service.globus_login_domain("utexas").await.expect("a login domain");

    assert_eq!(domain, "utexas.edu");
}

#[tokio::test]
async fn an_unknown_institution_is_a_bad_request() {
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![utexas()])));

    let error = service.globus_login_domain("nope").await.expect_err("an error");

    assert_eq!(bad_request_message(error), "Unknown institution 'nope'");
}

#[tokio::test]
async fn a_disabled_institution_is_a_bad_request() {
    let mut disabled = utexas();
    disabled.enabled = false;
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![disabled])));

    let error = service.globus_login_domain("utexas").await.expect_err("an error");

    assert_eq!(bad_request_message(error), "Institution 'utexas' is not enabled for login");
}

#[tokio::test]
async fn other_repository_errors_are_not_turned_into_bad_requests() {
    let service = InstitutionService::new(Arc::new(
        MockInstitutionRepo::new(vec![utexas()]).with_error("database unavailable"),
    ));

    let error = service.globus_login_domain("utexas").await.expect_err("an error");

    assert!(error.downcast_ref::<tms_lib::utils::service_error::ServiceError>().is_none());
    assert_eq!(error.to_string(), "database unavailable");
}
