pub mod keys_dao;

use crate::obj_model::key::Key;
use crate::ports::key::KeyRepository;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub struct PostgresKeyRepository {
    db_pool: PgPool,
}

impl PostgresKeyRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl From<&PgRow> for Key {
    fn from(row: &PgRow) -> Self {
        Key {
            kid: row.get("kid"),
            jwt_public_key: row.get("jwt_public_key"),
            jwt_private_key: row.get("jwt_private_key"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

#[async_trait::async_trait]
impl KeyRepository for PostgresKeyRepository {
    async fn get_by_id(&self, kid: &String) -> anyhow::Result<Key> {
        let row = query("select * from keys where kid = $1")
            .bind(kid)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => NotFound(format!("Key kid {} not found", kid)).into(),
                _ => anyhow::anyhow!(error),
            })?;
        Ok(Key::from(&row))
    }
}
