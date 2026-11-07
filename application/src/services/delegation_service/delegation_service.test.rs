use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::{
    db::{
        client_repository::test_fixtures::MockClientRepo,
        delegation_repository::test_fixtures::MockDelegationRepo,
        resource_provider_login_repository::test_fixtures::MockResourceProviderLoginRepo,
    },
    obj_model::{client::test_fixtures::ClientBuilder, configuration::DelegationPolicyConfig, resources::{test_fixtures::ResourceProviderLoginBuilder, ResourceProviderLogin}},
    utils::jwt_utils::SecurityContext,
};

use super::DelegationService;

#[tokio::test]
async fn delegation_fails_when_client_ids_dont_match_and_not_tms_client() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id(values.client_id.clone())
            .with_name(values.client_name.clone())
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
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service.add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account
        )
        .await;

    assert!(result.is_err())
}

#[tokio::test]
async fn delegation_succeeds_when_client_ids_dont_match_and_is_tms_client() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    // Initialize mock Client repo
    let mock_client_repo = Arc::new(MockClientRepo::new(
        vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ]
    ));

    // Initialize mock Delegation repo
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));

    // Initialize mock ResourceProviderLogin repo
    let initial_rp_logins = vec![
        utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )
    ];
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(initial_rp_logins));

    // Initialize Delegation service
    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        values.delegation_policy.clone(),
    );

    let ctx = SecurityContext {
        client_id: "different-client-id".into(),
        is_tms_client: true,
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service
        .add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account,
        )
        .await;
    
    assert!(result.is_ok())
}

#[tokio::test]
async fn delegation_succeeds_when_requested_client_id_matches_security_context() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id(values.client_id.clone())
            .with_name(values.client_name.clone())
            .build(),
    ];
    let mock_client_repo = Arc::new(MockClientRepo::new(initial_clients));
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));
    
    // Initialize mock ResourceProviderLogin repo
    let initial_rp_logins = vec![
        utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )
    ];
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(initial_rp_logins));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        values.delegation_policy.clone(),
    );

    let ctx = SecurityContext {
        client_id: values.client_id.clone(),
        is_tms_client: false,
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service
        .add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account,
        )
        .await;
    
    assert!(result.is_ok())
}

#[tokio::test]
async fn delegation_fails_when_no_recent_login_found() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id(values.client_id.clone())
            .with_name(values.client_name.clone())
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
        client_id: values.client_id.clone(),
        is_tms_client: false,
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service
        .add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account,
        )
        .await;

    assert!(result.is_err())
}

#[tokio::test]
async fn fails_due_to_internal_client_repo_error() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id(values.client_id.clone())
            .with_name(values.client_name.clone())
            .build(),
    ];
    let mock_client_repo = Arc::new(
        MockClientRepo::new(initial_clients).with_error(values.client_repo_error_message.clone())
    );
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(vec![]));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default(),
    );

    let ctx = SecurityContext {
        client_id: values.client_id.clone(),
        is_tms_client: false,
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service
        .add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account,
        )
        .await;
    
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string() == values.client_repo_error_message);
}

#[tokio::test]
async fn fails_due_to_internal_rp_login_repo_error() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id(values.client_id.clone())
            .with_name(values.client_name.clone())
            .build(),
    ];
    let mock_client_repo = Arc::new(MockClientRepo::new(initial_clients));
    let mock_delegation_repo = Arc::new(MockDelegationRepo::new(vec![]));
    let mock_rp_login_repo = Arc::new(
        MockResourceProviderLoginRepo::new(vec![])
            .with_error(values.rp_login_repo_error_message.clone())
        );

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default(),
    );

    let ctx = SecurityContext {
        client_id: values.client_id.clone(),
        is_tms_client: false,
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service
        .add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account,
        )
        .await;
    
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string() == values.rp_login_repo_error_message);
}

#[tokio::test]
async fn fails_due_to_internal_delegation_repo_error() {
    // Initialize the values that will be used in this test
    let values = utils::DefaultTestValues::new();

    let initial_clients = vec![
        ClientBuilder::new()
            .with_client_id(values.client_id.clone())
            .with_name(values.client_name.clone())
            .build(),
    ];
    let mock_client_repo = Arc::new(MockClientRepo::new(initial_clients));
    let mock_delegation_repo = Arc::new(
        MockDelegationRepo::new(vec![])
            .with_error(values.delegation_repo_error_message.clone())
    );
    let mock_rp_login_repo = Arc::new(MockResourceProviderLoginRepo::new(vec![]));

    let service = DelegationService::new(
        mock_client_repo,
        mock_delegation_repo,
        mock_rp_login_repo,
        DelegationPolicyConfig::default(),
    );

    let ctx = SecurityContext {
        client_id: values.client_id.clone(),
        is_tms_client: false,
        tms_identity: values.tms_identity.clone(),
        token: values.ctx_token.clone(),
    };

    let result = service
        .add_delegation(
            &ctx,
            &values.client_name,
            &values.rp_id,
            &values.rp_account,
        )
        .await;
    
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string() == values.delegation_repo_error_message)
}

mod utils {
    use super::*;

    pub(super) fn build_valid_login_from_delegation_policy(
        policy: DelegationPolicyConfig,
        tms_identity: &str,
        rp_id: &str,
        rp_account: &str,
    ) -> ResourceProviderLogin {
        let earliest_login = Utc::now() - Duration::minutes(
            policy.delegation_max_mins_since_login.clone()
        );
    
        ResourceProviderLoginBuilder::new()
            .with_unexpired_last_login(earliest_login)
            .with_tms_identity(tms_identity.into())
            .with_rp_id(rp_id.into())
            .with_rp_account(rp_account.into())
            .with_enabled(true)
            .build()
    }

    #[derive(Clone, Debug)]
    pub(super) struct DefaultTestValues {
        pub delegation_policy: DelegationPolicyConfig,
        pub tms_identity: String,
        pub rp_id: String,
        pub rp_account: String,
        pub client_id: String,
        pub client_name: String,
        pub ctx_token: String,
        pub delegation_repo_error_message: String,
        pub rp_login_repo_error_message: String,
        pub client_repo_error_message: String,
    }

    impl DefaultTestValues {
        pub fn new() -> Self {
            Self {
                delegation_policy: DelegationPolicyConfig::default(),
                tms_identity: "test-tms-identity".into(),
                rp_id: "test-rp-id".into(),
                rp_account: "test-rp-account".into(),
                client_id: "test-client-id".into(),
                client_name: "test-client-name".into(),
                ctx_token: "test-token".into(),
                client_repo_error_message: "Client Repo Network Error".into(),
                delegation_repo_error_message: "Delegation Repo Network Error".into(),
                rp_login_repo_error_message: "Resource Provider Login Repo Network Error".into(),
            }
        }
    }
}
