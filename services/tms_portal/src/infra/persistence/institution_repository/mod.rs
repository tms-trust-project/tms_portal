#[cfg(test)]
pub mod test_fixtures;

use crate::obj_model::institutions::Institution;
use crate::ports::institution::InstitutionRepository;
use sqlx::postgres::PgRow;
use sqlx::types::Json;
use sqlx::{PgPool, Row, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub struct PostgresInstitutionRepository {
    db_pool: PgPool,
}

impl PostgresInstitutionRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl InstitutionRepository for PostgresInstitutionRepository {
    async fn list(&self) -> anyhow::Result<Vec<Institution>> {
        let rows = query(
            "SELECT id, name, enabled, federated_identity_config FROM institutions",
        )
            .fetch_all(&self.db_pool)
            .await?;

        Ok(rows.iter().map(Institution::from).collect())
    }

    async fn get_by_id(&self, institution_id: &str) -> anyhow::Result<Institution> {
        let row = query(
            "SELECT id, name, enabled, federated_identity_config FROM institutions WHERE id = $1",
        )
            .bind(institution_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => {
                    NotFound(format!("Institution id '{}' not found", institution_id)).into()
                }
                _ => anyhow::anyhow!(error),
            })?;

        Ok(Institution::from(&row))
    }
}

impl From<&PgRow> for Institution {
    fn from(row: &PgRow) -> Self {
        Institution {
            id: row.get("id"),
            name: row.get("name"),
            enabled: row.get("enabled"),
            federated_identity_config: row
                .get::<Json<_>, _>("federated_identity_config")
                .0,
        }
    }
}
