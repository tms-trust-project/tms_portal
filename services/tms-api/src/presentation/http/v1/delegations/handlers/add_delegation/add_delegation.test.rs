use std::sync::Arc;

use axum::{Json, extract::State};
use http::StatusCode;

use crate::{
    infra::persistence::{
        client_repository::test_fixtures::MockClientRepo,
        delegation_repository::test_fixtures::MockDelegationRepo,
        resource_provider_login_repository::test_fixtures::MockResourceProviderLoginRepo,
    },
    obj_model::{
        client::test_fixtures::ClientBuilder, configuration::DelegationPolicyConfig,
        resource::test_fixtures::ResourceProviderLoginBuilder,
    },
    services::delegation_service::DelegationService,
    utils::jwt_utils::{JwtValidator, SecurityContext},
};

use super::{AddDelegationRequestBody, add_delegation};

#[tokio::test]
async fn adds_a_delegation_and_maps_it_to_the_api_response() {
    let client_id = "test-client-id";
    let client_name = "test-client";
    let tms_identity = "test-tms-identity";
    let resource_provider_id = "test-resource-provider";
    let resource_provider_account = "test-account";
    let delegation_policy = DelegationPolicyConfig::default();

    let client = ClientBuilder::new()
        .with_client_id(client_id.into())
        .with_name(client_name.into())
        .build();

    let recent_login = ResourceProviderLoginBuilder::new()
        .with_tms_identity(tms_identity.into())
        .with_rp_id(resource_provider_id.into())
        .with_rp_account(resource_provider_account.into())
        .build();

    let delegation_service = DelegationService::new(
        delegation_policy.clone(),
        Arc::new(MockClientRepo::new(vec![client])),
        Arc::new(MockDelegationRepo::new(vec![])),
        Arc::new(MockResourceProviderLoginRepo::new(vec![recent_login])),
    );

    let security_context = SecurityContext {
        tms_identity: tms_identity.into(),
        token: "test-token".into(),
        client_id: client_id.into(),
        is_tms_client: false,
    };

    let request = AddDelegationRequestBody {
        client_name: client_name.into(),
        resource_provider_id: resource_provider_id.into(),
        resource_provider_account: resource_provider_account.into(),
    };

    let response_result =
        add_delegation(State(delegation_service), JwtValidator(security_context), Json(request))
            .await;

    let response = match response_result {
        Ok(response) => response,
        Err(_) => panic!("the delegation should be added"),
    };

    assert_eq!(response.status_code, StatusCode::OK);

    let delegation = response
        .entity
        .expect("the response should contain a delegation");

    assert_eq!(delegation.client_id, client_id);
    assert_eq!(delegation.tms_identity, tms_identity);
    assert_eq!(delegation.rp_id, resource_provider_id);
    assert_eq!(delegation.rp_account, resource_provider_account);
}
