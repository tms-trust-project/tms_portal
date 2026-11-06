use std::sync::Arc;

use crate::{
    db::{
        client_repository::test_fixtures::{MockClientRepo, MockClientReturnType},
        delegation_repository::test_fixtures::{MockDelegationRepo, MockDelegationReturnType},
        resource_provider_login_repository::test_fixtures::{
            MockResourceProviderLoginRepo, MockResourceProviderLoginReturnType,
        },
    },
    obj_model::configuration::DelegationPolicyConfig,
    utils::jwt_utils::SecurityContext,
};

use super::DelegationService;

#[tokio::test]
async fn delegation_fails_when_client_ids_dont_match_for_non_tms_client() {
    let mock_client_repo = Arc::new(MockClientRepo::new(MockClientReturnType::Client));
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(MockDelegationReturnType::Delegation));
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(MockResourceProviderLoginReturnType::ResourceProviderLogin));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default()
    );

    let ctx = SecurityContext {
        client_id: "client-id-A".into(),
        is_tms_client: false,
        tms_identity: "foo".into(),
        token: "token".into(),
    };

    let result = service.add_delegation(
        &ctx,
        "test_client_name".into(),
        "test_rp_id".into(),
        "test_rp_account".into(),
    ).await;

    assert!(result.is_err())
}
