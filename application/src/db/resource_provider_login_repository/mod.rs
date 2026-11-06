pub mod test_fixtures;

use crate::obj_model::resources::ResourceProviderLogin;
use crate::ports::resource_provider_login::ResourceProviderLoginRepository;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query};

pub struct PostgresResourceProviderLoginRepository {
    db_pool: PgPool,
}

impl PostgresResourceProviderLoginRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl ResourceProviderLoginRepository for PostgresResourceProviderLoginRepository {
    async fn find(
        &self,
        tms_identity: &str,
        rp_account: &str,
        rp_id: &str,
        minimum_last_login: DateTime<Utc>,
        enabled: bool,
    ) -> anyhow::Result<Option<ResourceProviderLogin>> {
        let row = query(
            "SELECT * FROM resource_provider_logins WHERE tms_identity = $1 and rp_account=$2 and rp_id=$3 and last_login >= $4 and enabled is $5",
        )
        .bind(tms_identity)
        .bind(rp_account)
        .bind(rp_id)
        .bind(minimum_last_login)
        .bind(enabled)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

        match row {
            Some(row) => Ok(Some(ResourceProviderLogin::from(&row))),
            None => Ok(None),
        }
    }
}
