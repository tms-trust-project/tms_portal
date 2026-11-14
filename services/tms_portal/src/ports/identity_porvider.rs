use std::collections::HashSet;

use anyhow::Result;

use crate::obj_model::identity_provider::IdentityProvider;


#[async_trait::async_trait]
pub trait IdentityProviderRepository: Send + Sync {
    /// Returns the list of identity providers that support login
    async fn get_login_providers(&self) -> Result<HashSet<IdentityProvider>>;

    /// Returns an identity provider by id if it supports login
    async fn get_login_provider_by_id(&self, id: &str) -> Result<IdentityProvider>;

    /// Returns the list of identity providers that support resources
    async fn list_resource_providers(
        &self,
        tms_identity: &str,
        linked_only: &bool,
    ) -> Result<HashSet<IdentityProvider>>;

    /// Returns an identity provider by id if it supports resources
    async fn get_resource_provider_by_id(
        &self,
        provider_id: &str,
    ) -> Result<IdentityProvider>;
}