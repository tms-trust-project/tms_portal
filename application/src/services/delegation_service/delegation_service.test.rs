use chrono::{Duration, Utc};

use crate::{
    obj_model::client::test_fixtures::ClientBuilder,
    obj_model::configuration::DelegationPolicyConfig,
    obj_model::resources::ResourceProviderLogin,
    obj_model::resources::test_fixtures::ResourceProviderLoginBuilder,
    services::delegation_service::test_fixtures::{
        SecurityContextBuilder, TestDelegationServiceBuilder, TestValuesBuilder,
    },
};

#[tokio::test]
async fn add_delegation_fails_when_client_ids_dont_match_and_not_tms_client() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_init_mock_rp_login_repo(vec![])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    // Initialize a security context that uses a different client id than the mocked client
    // and where the security context is NOT a tms client
    let ctx = SecurityContextBuilder::new().build_with_non_matching_client_ids_as_non_tms_client(&values);

    // Add the delegation
    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

    // Fails as expected
    assert!(result.is_err())
}

#[tokio::test]
async fn add_delegation_succeeds_when_client_ids_dont_match_and_is_tms_client() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    // Initialize a security context that uses a different client id than the mocked client
    // and where the security context is a tms client
    let ctx = SecurityContextBuilder::new().build_with_non_match_client_ids_as_tms_client(&values);

    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

    assert!(result.is_ok())
}

#[tokio::test]
async fn add_delegation_succeeds_when_requested_client_id_matches_security_context() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx = SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

    assert!(result.is_ok())
}

#[tokio::test]
async fn add_delegation_fails_when_no_recent_login_found() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_init_mock_rp_login_repo(vec![])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx = SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

    assert!(result.is_err())
}

#[tokio::test]
async fn add_delegation_fails_due_to_internal_client_repo_error() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_mock_client_repo_error(&values.client_repo_error_message)
        .with_init_mock_rp_login_repo(vec![])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx = SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string() == values.client_repo_error_message);
}

#[tokio::test]
async fn add_delegation_fails_due_to_internal_rp_login_repo_error() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_mock_rp_login_repo_error(&values.rp_login_repo_error_message)
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx = SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string() == values.rp_login_repo_error_message);
}

#[tokio::test]
async fn add_delegation_fails_due_to_internal_delegation_repo_error() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new().with_client_id(values.client_id.clone()).with_name(values.client_name.clone()).build(),
        ])
        .with_mock_delegation_repo_error(&values.delegation_repo_error_message)
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx = SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service.add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account).await;

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
        let earliest_login = Utc::now() - Duration::minutes(policy.delegation_max_mins_since_login.clone());

        ResourceProviderLoginBuilder::new()
            .with_unexpired_last_login(earliest_login)
            .with_tms_identity(tms_identity.into())
            .with_rp_id(rp_id.into())
            .with_rp_account(rp_account.into())
            .with_enabled(true)
            .build()
    }
}
