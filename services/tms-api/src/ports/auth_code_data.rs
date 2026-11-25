use crate::obj_model::auth_code_data::AuthCodeData;
use crate::obj_model::identity_provider::IdentityProviderType;
use crate::utils::jwt_utils::TmsTokenClaims;
use chrono::TimeDelta;

#[async_trait::async_trait]
pub trait AuthCodeDataRepository: Send + Sync {
    async fn delete(
        &self,
        auth_code: &str,
        client_id: &str,
        redirect_uri: &str,
        expiration: TimeDelta,
    ) -> anyhow::Result<AuthCodeData>;

    async fn insert(
        &self,
        auth_code: &str,
        client_id: &str,
        redirect_uri: &str,
        claims: &TmsTokenClaims,
        idp_id: &str,
        idp_type: &IdentityProviderType,
    ) -> anyhow::Result<AuthCodeData>;
}
