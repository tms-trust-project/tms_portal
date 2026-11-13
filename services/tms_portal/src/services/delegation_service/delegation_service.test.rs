use chrono::{Duration, Utc};

use crate::{
    obj_model::{
        client::test_fixtures::ClientBuilder,
        configuration::DelegationPolicyConfig,
        delegation::test_fixtures::DelegationBuilder,
        resource::{ResourceProviderLogin, test_fixtures::ResourceProviderLoginBuilder},
    },
    services::delegation_service::test_fixtures::{
        SecurityContextBuilder, TestDelegationServiceBuilder, TestValuesBuilder,
    },
};

use tms_lib::utils::service_error::ServiceError;

#[tokio::test]
async fn add_delegation_succeeds_when_client_ids_dont_match_and_is_tms_client() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
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
    let ctx =
        SecurityContextBuilder::new().build_with_non_matching_client_ids_as_tms_client(&values);

    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

    assert!(result.is_ok())
}

#[tokio::test]
async fn add_delegation_succeeds_when_requested_client_id_matches_security_context() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
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

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

    assert!(result.is_ok())
}

#[tokio::test]
async fn add_delegation_fails_when_client_ids_dont_match_and_not_tms_client() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_init_mock_rp_login_repo(vec![])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    // Initialize a security context that uses a different client id than the mocked client
    // and where the security context is NOT a tms client
    let ctx =
        SecurityContextBuilder::new().build_with_non_matching_client_ids_as_non_tms_client(&values);

    // Add the delegation
    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

    // Fails as expected
    assert!(result.is_err())
}

#[tokio::test]
async fn add_delegation_fails_when_no_recent_login_found() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_init_mock_rp_login_repo(vec![])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

    assert!(result.is_err())
}

#[tokio::test]
async fn add_delegation_fails_due_to_internal_client_repo_error() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ])
        .with_mock_client_repo_error(&values.client_repo_error_message)
        .with_init_mock_rp_login_repo(vec![])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

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
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![])
        .with_mock_rp_login_repo_error(&values.rp_login_repo_error_message)
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

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
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
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

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .add_delegation(&ctx, &values.client_name, &values.rp_id, &values.rp_account)
        .await;

    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string() == values.delegation_repo_error_message)
}

#[tokio::test]
async fn tms_client_can_list_all_delegations_for_a_tms_identity() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_client_id("different-client-id".into())
                .with_client_name("different-client-name".into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_non_matching_client_ids_as_tms_client(&values);

    let result = service.list_delegations(&ctx).await;

    let delegations = result.expect("Returns delegations with no error");

    assert_eq!(delegations.len(), 2);
}

#[tokio::test]
async fn non_tms_clients_can_only_list_delegations_it_issued_for_a_tms_identity() {
    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_client_id("different-client-id".into())
                .with_client_name("different-client-name".into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service.list_delegations(&ctx).await;

    let delegations = result.expect("Returns delegations with no error");

    assert_eq!(delegations.len(), 1);
}

#[tokio::test]
async fn tms_client_can_list_delegations_issued_by_any_client_for_a_tms_identity() {
    const OTHER_CLIENT_ID: &'static str = "other-client-id";
    const OTHER_CLIENT_NAME: &'static str = "other-client-name";

    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
            ClientBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_name(OTHER_CLIENT_NAME.into())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_client_name(OTHER_CLIENT_NAME.into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_non_matching_client_ids_as_tms_client(&values);

    let owned_delegations = service
        .list_delegations_for_client(&ctx, &values.client_name)
        .await
        .expect("Returns owned delegations with no error");

    assert_eq!(owned_delegations.len(), 1);
    assert_eq!(owned_delegations.get(0).map(|c| &c.client_id), Some(&values.client_id));

    let non_owned_delegations = service
        .list_delegations_for_client(&ctx, OTHER_CLIENT_NAME)
        .await
        .expect("Returns non-owned delegations with no error");

    print!("{:#?}", &non_owned_delegations);

    assert_eq!(non_owned_delegations.len(), 1);
    assert_eq!(
        non_owned_delegations.get(0).map(|c| &c.client_id),
        Some(&OTHER_CLIENT_ID.to_string())
    );
}

#[tokio::test]
async fn non_tms_client_fails_to_list_delegations_it_did_not_issue() {
    const OTHER_CLIENT_ID: &'static str = "other-client-id";
    const OTHER_CLIENT_NAME: &'static str = "other-client-name";

    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
            ClientBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_name(OTHER_CLIENT_NAME.into())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_client_name(OTHER_CLIENT_NAME.into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let owned_delegations = service
        .list_delegations_for_client(&ctx, &values.client_name)
        .await
        .expect("Returns owned delegations with no error");

    assert_eq!(owned_delegations.len(), 1);
    assert_eq!(owned_delegations.get(0).map(|c| &c.client_id), Some(&values.client_id));

    let result = service
        .list_delegations_for_client(&ctx, OTHER_CLIENT_NAME)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().downcast_ref::<ServiceError>(),
        Some(ServiceError::Unauthorized(_))
    ))
}

#[tokio::test]
async fn non_tms_client_successfully_deletes_its_own_delegations() {
    const OTHER_CLIENT_ID: &'static str = "other-client-id";
    const OTHER_CLIENT_NAME: &'static str = "other-client-name";
    const NON_TMS_OWNED_DELEGATION_ID: i32 = 1;
    const TMS_OWNED_DELEGATION_ID: i32 = 2;

    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
            ClientBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_name(OTHER_CLIENT_NAME.into())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_id(NON_TMS_OWNED_DELEGATION_ID)
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_id(TMS_OWNED_DELEGATION_ID)
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_client_name(OTHER_CLIENT_NAME.into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .delete_delegation(&ctx, &values.client_name, NON_TMS_OWNED_DELEGATION_ID)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tms_client_successfully_deletes_any_delegations() {
    const OTHER_CLIENT_ID: &'static str = "other-client-id";
    const OTHER_CLIENT_NAME: &'static str = "other-client-name";
    const NON_TMS_OWNED_DELEGATION_ID: i32 = 1;
    const TMS_OWNED_DELEGATION_ID: i32 = 2;

    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
            ClientBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_name(OTHER_CLIENT_NAME.into())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_id(TMS_OWNED_DELEGATION_ID)
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_id(NON_TMS_OWNED_DELEGATION_ID)
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_client_name(OTHER_CLIENT_NAME.into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx = SecurityContextBuilder::new().build_with_matching_client_ids_as_tms_client(&values);

    let result = service
        .delete_delegation(&ctx, &values.client_name, TMS_OWNED_DELEGATION_ID)
        .await;

    assert!(result.is_ok());

    let result = service
        .delete_delegation(&ctx, OTHER_CLIENT_NAME, NON_TMS_OWNED_DELEGATION_ID)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn non_tms_client_failes_to_delete_a_delegation_it_does_not_own() {
    const OTHER_CLIENT_ID: &'static str = "other-client-id";
    const OTHER_CLIENT_NAME: &'static str = "other-client-name";
    const NON_TMS_OWNED_DELEGATION_ID: i32 = 1;
    const TMS_OWNED_DELEGATION_ID: i32 = 2;

    // Initialize the values that will be used in this test
    let values = TestValuesBuilder::new().build();

    // Build a DelegationService with mocked dependencies
    let service = TestDelegationServiceBuilder::new()
        .with_init_mock_client_repo(vec![
            ClientBuilder::new()
                .with_client_id(values.client_id.clone())
                .with_name(values.client_name.clone())
                .build(),
            ClientBuilder::new()
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_name(OTHER_CLIENT_NAME.into())
                .build(),
        ])
        .with_init_mock_delegation_repo(vec![
            DelegationBuilder::new()
                .with_id(NON_TMS_OWNED_DELEGATION_ID)
                .with_client_id(values.client_id.clone())
                .with_client_name(values.client_name.clone())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
            DelegationBuilder::new()
                .with_id(TMS_OWNED_DELEGATION_ID)
                .with_client_id(OTHER_CLIENT_ID.into())
                .with_client_name(OTHER_CLIENT_NAME.into())
                .with_rp_account(values.rp_account.clone())
                .with_rp_id(values.rp_id.clone())
                .with_tms_identity(values.tms_identity.clone())
                .build(),
        ])
        .with_init_mock_rp_login_repo(vec![utils::build_valid_login_from_delegation_policy(
            values.delegation_policy.clone(),
            &values.tms_identity,
            &values.rp_id,
            &values.rp_account,
        )])
        .with_delegation_policy(values.delegation_policy.clone())
        .build();

    let ctx =
        SecurityContextBuilder::new().build_with_matching_client_ids_as_non_tms_client(&values);

    let result = service
        .delete_delegation(&ctx, OTHER_CLIENT_NAME, NON_TMS_OWNED_DELEGATION_ID)
        .await;

    assert!(result.is_err());
}

mod utils {
    use super::*;

    pub(super) fn build_valid_login_from_delegation_policy(
        policy: DelegationPolicyConfig,
        tms_identity: &str,
        rp_id: &str,
        rp_account: &str,
    ) -> ResourceProviderLogin {
        let earliest_login =
            Utc::now() - Duration::minutes(policy.delegation_max_mins_since_login.clone());

        ResourceProviderLoginBuilder::new()
            .with_unexpired_last_login(earliest_login)
            .with_tms_identity(tms_identity.into())
            .with_rp_id(rp_id.into())
            .with_rp_account(rp_account.into())
            .with_enabled(true)
            .build()
    }
}
