use std::sync::Arc;

use crate::{db::{client_repository::test_fixtures::MockClientRepo, delegation_repository::test_fixtures::MockDelegationRepo, resource_provider_login_repository::test_fixtures::MockResourceProviderLoginRepo}, obj_model::{client::Client, configuration::DelegationPolicyConfig, delegation::{self, Delegation}, resources::ResourceProviderLogin}, ports::{client::ClientRepository, delegation::DelegationRepository, resource_provider_login::ResourceProviderLoginRepository}};

use super::DelegationService;

pub(super) struct TestDelegationServiceBuilder {
    client_repo: Option<Arc<dyn ClientRepository>>,
    delegation_repo: Option<Arc<dyn DelegationRepository>>,
    resource_provider_login_repo: Option<Arc<dyn ResourceProviderLoginRepository>>,
    delegation_policy: Option<DelegationPolicyConfig>,
}

impl TestDelegationServiceBuilder {
    pub fn new() -> Self {
        Self {
            client_repo: None,
            delegation_repo: None,
            resource_provider_login_repo: None,
            delegation_policy: None
        }
    }

    pub fn with_init_mock_client_repo(mut self, clients: Vec<Client>) -> Self {
        self.client_repo = Some(Arc::new(MockClientRepo::new(clients)));
        self
    }

    pub fn with_mock_client_repo_error(mut self, error: &str) -> Self {
        self.client_repo = Some(Arc::new(MockClientRepo::new(vec![]).with_error(error)));
        self
    }

    pub fn with_init_mock_delegation_repo(mut self, delegations: Vec<Delegation>) -> Self{
        self.delegation_repo = Some(Arc::new(MockDelegationRepo::new(delegations)));
        self
    }

    pub fn with_mock_delegation_repo_error(mut self, error: &str) -> Self {
        self.delegation_repo = Some(Arc::new(MockDelegationRepo::new(vec![]).with_error(error)));
        self
    }

    pub fn with_init_mock_rp_login_repo(mut self, logins: Vec<ResourceProviderLogin>) -> Self{
        self.resource_provider_login_repo = Some(Arc::new(MockResourceProviderLoginRepo::new(logins)));
        self
    }

    pub fn with_mock_rp_login_repo_error(mut self, error: &str) -> Self {
        self.resource_provider_login_repo = Some(Arc::new(MockResourceProviderLoginRepo::new(vec![]).with_error(error)));
        self
    }

    pub fn with_delegation_policy(mut self, policy: DelegationPolicyConfig) -> Self {
        self.delegation_policy = Some(policy);
        self
    }

    pub fn build(&self) -> DelegationService {
        let client_repo = self.client_repo.clone().unwrap_or(Arc::new(MockClientRepo::new(vec![])));
        let delegation_repo = self.delegation_repo.clone().unwrap_or(Arc::new(MockDelegationRepo::new(vec![])));
        let resource_provider_login_repo = self.resource_provider_login_repo.clone().unwrap_or(Arc::new(MockResourceProviderLoginRepo::new(vec![])));
        let delegation_policy = self.delegation_policy.clone().unwrap_or(DelegationPolicyConfig::default());

        DelegationService::new(
            client_repo,
            delegation_repo,
            resource_provider_login_repo,
            delegation_policy,
        )
    }
}

