use std::sync::Arc;

use crate::{
    db::{
        client_repository::test_fixtures::MockClientRepo,
        delegation_repository::test_fixtures::MockDelegationRepo,
        resource_provider_login_repository::test_fixtures::MockResourceProviderLoginRepo,
    },
    obj_model::{client::test_fixtures::ClientBuilder, configuration::DelegationPolicyConfig},
    utils::jwt_utils::SecurityContext,
};

use super::DelegationService;

#[tokio::test]
async fn delegation_fails_when_client_ids_dont_match_for_non_tms_client() {
    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id("test-client-id".into())
            .with_name("test_client_name".into())
            .build(),
    ];
    let mock_client_repo = Arc::new(MockClientRepo::new(initial_clients));
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(vec![]));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default(),
    );

    let ctx = SecurityContext {
        client_id: "different-client-id".into(),
        is_tms_client: false,
        tms_identity: "foo".into(),
        token: "token".into(),
    };

    let result = service
        .add_delegation(
            &ctx,
            "test-client-name".into(),
            "test-rp-id".into(),
            "test-rp-account".into(),
        )
        .await;

    assert!(result.is_err())
}

#[tokio::test]
async fn delegation_succeeds_if_client_ids_ne_and_is_tms_client() {
    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id("test-client-id".into())
            .with_name("test_client_name".into())
            .build(),
    ];
    let mock_client_repo = Arc::new(MockClientRepo::new(initial_clients));
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(vec![]));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default(),
    );

    let ctx = SecurityContext {
        client_id: "different-client-id".into(),
        is_tms_client: true,
        tms_identity: "foo".into(),
        token: "token".into(),
    };

    let result = service
        .add_delegation(
            &ctx,
            "test-client-name".into(),
            "test-rp-id".into(),
            "test-rp-account".into(),
        )
        .await;

    assert!(result.is_err() != false)
}

#[tokio::test]
async fn delegation_fails_when_no_recent_login_found() {
    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id("client-id-A".into())
            .with_name("test_client_name".into())
            .build(),
    ];
    let mock_client_repo = Arc::new(MockClientRepo::new(initial_clients));
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(vec![]));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default(),
    );

    let ctx = SecurityContext {
        client_id: "client-id-A".into(),
        is_tms_client: false,
        tms_identity: "foo".into(),
        token: "token".into(),
    };

    let result = service
        .add_delegation(
            &ctx,
            "test_client_name".into(),
            "test_rp_id".into(),
            "test_rp_account".into(),
        )
        .await;

    assert!(result.is_err())
}
