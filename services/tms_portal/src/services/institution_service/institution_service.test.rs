use std::sync::Arc;

use crate::infra::persistence::institution_repository::test_fixtures::MockInstitutionRepo;
use crate::obj_model::institutions::test_fixtures::InstitutionBuilder;
use crate::services::institution_service::InstitutionService;

#[tokio::test]
async fn lists_institutions() {
    let first = InstitutionBuilder::new()
        .with_id("first-institution".into())
        .build();
    let second = InstitutionBuilder::new()
        .with_id("second-institution".into())
        .build();
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![first, second])));

    let institutions = service
        .list_institutions()
        .await
        .expect("institutions should be listed");

    assert_eq!(institutions.len(), 2);
    assert_eq!(institutions[0].id, "first-institution");
    assert_eq!(institutions[1].id, "second-institution");
}

#[tokio::test]
async fn finds_institution_by_id() {
    let institution = InstitutionBuilder::new()
        .with_id("requested-institution".into())
        .with_name("Requested Institution".into())
        .build();
    let service = InstitutionService::new(Arc::new(MockInstitutionRepo::new(vec![institution])));

    let institution = service
        .find_institution_by_id("requested-institution")
        .await
        .expect("the institution should be found");

    assert_eq!(institution.id, "requested-institution");
    assert_eq!(institution.name, "Requested Institution");
}
