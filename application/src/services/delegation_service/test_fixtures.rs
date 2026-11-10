use std::sync::Arc;

use crate::{
    db::{
        client_repository::test_fixtures::MockClientRepo,
        delegation_repository::test_fixtures::MockDelegationRepo,
        resource_provider_login_repository::test_fixtures::MockResourceProviderLoginRepo,
    },
    obj_model::{
        client::Client, configuration::DelegationPolicyConfig, delegation::Delegation,
        resources::ResourceProviderLogin,
    },
    ports::{
        client::ClientRepository, delegation::DelegationRepository,
        resource_provider_login::ResourceProviderLoginRepository,
    },
    utils::jwt_utils::SecurityContext,
};

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
            delegation_policy: None,
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

    pub fn with_init_mock_delegation_repo(mut self, delegations: Vec<Delegation>) -> Self {
        self.delegation_repo = Some(Arc::new(MockDelegationRepo::new(delegations)));
        self
    }

    pub fn with_mock_delegation_repo_error(mut self, error: &str) -> Self {
        self.delegation_repo = Some(Arc::new(MockDelegationRepo::new(vec![]).with_error(error)));
        self
    }

    pub fn with_init_mock_rp_login_repo(mut self, logins: Vec<ResourceProviderLogin>) -> Self {
        self.resource_provider_login_repo =
            Some(Arc::new(MockResourceProviderLoginRepo::new(logins)));
        self
    }

    pub fn with_mock_rp_login_repo_error(mut self, error: &str) -> Self {
        self.resource_provider_login_repo =
            Some(Arc::new(MockResourceProviderLoginRepo::new(vec![]).with_error(error)));
        self
    }

    pub fn with_delegation_policy(mut self, policy: DelegationPolicyConfig) -> Self {
        self.delegation_policy = Some(policy);
        self
    }

    pub fn build(&self) -> DelegationService {
        let client_repo = self
            .client_repo
            .clone()
            .unwrap_or(Arc::new(MockClientRepo::new(vec![])));
        let delegation_repo = self
            .delegation_repo
            .clone()
            .unwrap_or(Arc::new(MockDelegationRepo::new(vec![])));
        let resource_provider_login_repo = self
            .resource_provider_login_repo
            .clone()
            .unwrap_or(Arc::new(MockResourceProviderLoginRepo::new(vec![])));
        let delegation_policy = self
            .delegation_policy
            .clone()
            .unwrap_or(DelegationPolicyConfig::default());

        DelegationService::new(
            client_repo,
            delegation_repo,
            resource_provider_login_repo,
            delegation_policy,
        )
    }
}

#[derive(Clone, Debug)]
pub(super) struct TestValues {
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

pub struct TestValuesBuilder {
    pub delegation_policy: Option<DelegationPolicyConfig>,
    pub tms_identity: Option<String>,
    pub rp_id: Option<String>,
    pub rp_account: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub ctx_token: Option<String>,
    pub delegation_repo_error_message: Option<String>,
    pub rp_login_repo_error_message: Option<String>,
    pub client_repo_error_message: Option<String>,
}

impl TestValuesBuilder {
    const DEFAULT_TMS_IDENTITY: &str = "test-tms-identity";
    const DEFAULT_CLIENT_ID: &str = "test-client-id";
    const DEFAULT_CLIENT_NAME: &str = "test-client-name";
    const DEFAULT_SECURITY_CONTEXT_TOKEN: &str = "test-token";
    const DEFAULT_RP_ID: &str = "test-rp-id";
    const DEFAULT_RP_ACCOUNT: &str = "test-rp-account";
    const DEFAULT_DELEGATION_REPO_ERROR: &str = "Delegation Repo Network Error";
    const DEFAULT_CLIENT_REPO_ERROR: &str = "Client Repo Network Error";
    const DEFAULT_RP_LOGIN_REPO_ERROR: &str = "Resource Provider Login Repo Network Error";

    pub fn new() -> Self {
        Self {
            delegation_policy: None,
            tms_identity: None,
            rp_id: None,
            rp_account: None,
            client_id: None,
            client_name: None,
            ctx_token: None,
            delegation_repo_error_message: None,
            rp_login_repo_error_message: None,
            client_repo_error_message: None,
        }
    }

    pub fn with_delegation_policy(mut self, delegation_policy: DelegationPolicyConfig) -> Self {
        self.delegation_policy = Some(delegation_policy);
        self
    }

    pub fn with_tms_identity(mut self, tms_identity: String) -> Self {
        self.tms_identity = Some(tms_identity);
        self
    }

    pub fn with_rp_id(mut self, rp_id: String) -> Self {
        self.rp_id = Some(rp_id);
        self
    }

    pub fn with_rp_account(mut self, rp_account: String) -> Self {
        self.rp_account = Some(rp_account);
        self
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_client_name(mut self, client_name: String) -> Self {
        self.client_name = Some(client_name);
        self
    }

    pub fn with_ctx_token(mut self, ctx_token: String) -> Self {
        self.ctx_token = Some(ctx_token);
        self
    }

    pub fn with_delegation_repo_error_message(mut self, error_message: String) -> Self {
        self.delegation_repo_error_message = Some(error_message);
        self
    }

    pub fn with_rp_login_repo_error_message(mut self, error_message: String) -> Self {
        self.rp_login_repo_error_message = Some(error_message);
        self
    }

    pub fn with_client_repo_error_message(mut self, error_message: String) -> Self {
        self.client_repo_error_message = Some(error_message);
        self
    }

    pub fn build(&self) -> TestValues {
        let tms_identity = self
            .tms_identity
            .clone()
            .unwrap_or(Self::DEFAULT_TMS_IDENTITY.into());
        let client_id = self
            .client_id
            .clone()
            .unwrap_or(Self::DEFAULT_CLIENT_ID.into());

        TestValues {
            delegation_policy: self.delegation_policy.clone().unwrap_or_default(),
            tms_identity,
            rp_id: self.rp_id.clone().unwrap_or(Self::DEFAULT_RP_ID.into()),
            rp_account: self
                .rp_account
                .clone()
                .unwrap_or(Self::DEFAULT_RP_ACCOUNT.into()),
            client_id,
            client_name: self
                .client_name
                .clone()
                .unwrap_or(Self::DEFAULT_CLIENT_NAME.into()),
            ctx_token: self
                .ctx_token
                .clone()
                .unwrap_or(Self::DEFAULT_SECURITY_CONTEXT_TOKEN.into()),
            delegation_repo_error_message: self
                .delegation_repo_error_message
                .clone()
                .unwrap_or(Self::DEFAULT_DELEGATION_REPO_ERROR.into()),
            rp_login_repo_error_message: self
                .rp_login_repo_error_message
                .clone()
                .unwrap_or(Self::DEFAULT_RP_LOGIN_REPO_ERROR.into()),
            client_repo_error_message: self
                .client_repo_error_message
                .clone()
                .unwrap_or(Self::DEFAULT_CLIENT_REPO_ERROR.into()),
        }
    }
}

// TODO Move to SecurityContext module once it's refactored
pub struct SecurityContextBuilder {
    pub tms_identity: Option<String>,
    pub token: Option<String>,
    pub client_id: Option<String>,
    pub is_tms_client: Option<bool>,
}

impl SecurityContextBuilder {
    pub fn new() -> Self {
        Self { tms_identity: None, token: None, client_id: None, is_tms_client: None }
    }

    pub fn with_tms_identity(mut self, tms_identity: String) -> Self {
        self.tms_identity = Some(tms_identity);
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn as_tms_client(mut self, is_tms_client: bool) -> Self {
        self.is_tms_client = Some(is_tms_client);
        self
    }

    pub fn build_with_matching_client_ids_as_non_tms_client(
        &self,
        test_values: &TestValues,
    ) -> SecurityContext {
        SecurityContext {
            tms_identity: self
                .tms_identity
                .clone()
                .unwrap_or(test_values.tms_identity.clone()),
            token: self.token.clone().unwrap_or(test_values.ctx_token.clone()),
            client_id: test_values.client_id.clone(),
            is_tms_client: false,
        }
    }

    pub fn build_with_matching_client_ids_as_tms_client(
        &self,
        test_values: &TestValues,
    ) -> SecurityContext {
        SecurityContext {
            tms_identity: self
                .tms_identity
                .clone()
                .unwrap_or(test_values.tms_identity.clone()),
            token: self.token.clone().unwrap_or(test_values.ctx_token.clone()),
            client_id: test_values.client_id.clone(),
            is_tms_client: true,
        }
    }

    pub fn build_with_non_matching_client_ids_as_tms_client(
        &self,
        test_values: &TestValues,
    ) -> SecurityContext {
        SecurityContext {
            tms_identity: self
                .tms_identity
                .clone()
                .unwrap_or(test_values.tms_identity.clone()),
            token: self.token.clone().unwrap_or(test_values.ctx_token.clone()),
            client_id: format!(
                "different-{}",
                self.client_id
                    .clone()
                    .unwrap_or(test_values.client_id.clone())
            ),
            is_tms_client: true,
        }
    }

    pub fn build_with_non_matching_client_ids_as_non_tms_client(
        &self,
        test_values: &TestValues,
    ) -> SecurityContext {
        SecurityContext {
            tms_identity: self
                .tms_identity
                .clone()
                .unwrap_or(test_values.tms_identity.clone()),
            token: self.token.clone().unwrap_or(test_values.ctx_token.clone()),
            client_id: format!(
                "different-{}",
                self.client_id
                    .clone()
                    .unwrap_or(test_values.client_id.clone())
            ),
            is_tms_client: false,
        }
    }
}
