use std::sync::Arc;

use crate::infra::persistence::institution_repository::test_fixtures::MockInstitutionRepo;
use crate::obj_model::institutions::test_fixtures::InstitutionBuilder;
use crate::obj_model::institutions::{FederatedIdentityConfig, GlobusConfiguration};
use crate::services::institution_service::InstitutionService;

#[tokio::test]
async fn lists_institutions() {
    let first = InstitutionBuilder::new()
        .with_id("first-institution".into())
        .with_enabled(true)
        .with_ipd_config(FederatedIdentityConfig {
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
            .ipd_config
            .globus
            .as_ref()
            .expect("the first institution should have Globus configuration")
            .redirect_idp_domain,
        "first.example.edu"
    );
    assert_eq!(institutions[1].id, "second-institution");
    assert!(!institutions[1].enabled);
    assert!(institutions[1].ipd_config.globus.is_none());
}

#[tokio::test]
async fn finds_institution_by_id() {
    let institution = InstitutionBuilder::new()
        .with_id("requested-institution".into())
        .with_name("Requested Institution".into())
        .with_enabled(false)
        .with_ipd_config(FederatedIdentityConfig {
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
        .ipd_config
        .globus
        .expect("the institution should have Globus configuration");
    assert_eq!(globus.redirect_idp_domain, "requested.example.edu");
    assert_eq!(globus.idp_logout_url, "https://requested.example.edu/logout");
}
