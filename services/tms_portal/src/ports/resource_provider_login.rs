use chrono::{DateTime, Utc};

use crate::obj_model::resource::ResourceProviderLogin;

#[async_trait::async_trait]
pub trait ResourceProviderLoginRepository: Send + Sync {
    async fn find(
        &self,
        tms_identity: &str,
        rp_account: &str,
        rp_id: &str,
        minimum_last_login: DateTime<Utc>,
        enabled: bool,
    ) -> anyhow::Result<Option<ResourceProviderLogin>>;
}
