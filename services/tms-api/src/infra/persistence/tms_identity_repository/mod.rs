pub mod tms_identities_dao;

use crate::obj_model::tms_identity::TmsIdentity;
use crate::ports::tms_identity::TmsIdentityRepository;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub struct PostgresTmsIdentityRepository {
    db_pool: PgPool,
}

impl PostgresTmsIdentityRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl From<&PgRow> for TmsIdentity {
    fn from(row: &PgRow) -> Self {
        TmsIdentity {
            seq_id: row.get("seq_id"),
            tms_identity: row.get("tms_identity"),
            created: row.get("created"),
        }
    }
}

#[async_trait::async_trait]
impl TmsIdentityRepository for PostgresTmsIdentityRepository {
    async fn get(&self, tms_identity: &str) -> anyhow::Result<TmsIdentity> {
        let row = query("select * from tms_identity where tms_identity = $1")
            .bind(tms_identity)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => {
                    NotFound(format!("Tms Identity {} not found", tms_identity)).into()
                }
                _ => anyhow::anyhow!(error),
            })?;
        Ok(TmsIdentity::from(&row))
    }

    async fn insert_if_absent(&self, tms_identity: &str) -> anyhow::Result<TmsIdentity> {
        // on conflict do update ... really does nothing, but it ensures that a record is returned
        let row = query("insert into tms_identities (tms_identity) values ($1) on conflict(tms_identity) do update set tms_identity = EXCLUDED.tms_identity returning *")
            .bind(tms_identity)
            .fetch_one(&self.db_pool)
            .await?;
        Ok(TmsIdentity::from(&row))
    }
}
