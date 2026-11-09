#[cfg(test)]
pub mod test_fixtures;

use crate::obj_model::delegation::Delegation;
use crate::ports::delegation::DelegationRepository;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, PgTransaction, Row, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub struct PostgresDelegationRepository {
    db_pool: PgPool,
}

impl PostgresDelegationRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl DelegationRepository for PostgresDelegationRepository {
    async fn save(
        &self,
        client_id: &str,
        rp_account: &str,
        expires_at: &DateTime<Utc>,
        tms_identity: &str,
        rp_id: &str,
    ) -> anyhow::Result<Delegation> {
        // on conflict do update ... really does nothing, but it ensures that a record is returned
        let delegation_id: i32 = query(
            "INSERT INTO delegations (client_id, rp_account, expires_at,
                             tms_identity, rp_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
            .bind(client_id)
            .bind(rp_account)
            .bind(expires_at)
            .bind(tms_identity)
            .bind(rp_id)
            .fetch_one(&self.db_pool)
            .await?
            .get("id");

        let row = query("SELECT dlgs.*, cl.name FROM delegations dlgs LEFT JOIN clients cl ON dlgs.client_id = cl.client_id WHERE dlgs.id = $1")
            .bind(delegation_id)
            .fetch_one(&self.db_pool)
            .await?;

        Ok(Delegation::from(&row))
    }

    async fn list_by_client_id(&self, client_id: &str, tms_identity: &str) -> anyhow::Result<Vec<Delegation>> {
        let rows= query("SELECT dlgs.*, cl.name FROM delegations dlgs LEFT JOIN clients cl ON dlgs.client_id = cl.client_id WHERE client_id = $1 AND tms_identity = $2")
            .bind(client_id)
            .bind(tms_identity)
            .fetch_all(&self.db_pool)
            .await?;

        let mut delegations = Vec::with_capacity(rows.len());
        for row in rows {
            delegations.push(Delegation::from(&row));
        }

        Ok(delegations)
    }

    async fn list_all(&self, tms_identity: &str) -> anyhow::Result<Vec<Delegation>> {
        let rows = query("SELECT dlgs.*, cl.name FROM delegations dlgs LEFT JOIN clients cl ON dlgs.client_id = cl.client_id WHERE tms_identity = $1")
            .bind(tms_identity)
            .fetch_all(&self.db_pool)
            .await?;

        let mut delegations = Vec::with_capacity(rows.len());
        for row in rows {
            delegations.push(Delegation::from(&row));
        }

        Ok(delegations)
    }

    async fn delete(&self, tms_identity: &str, client_id: &str, delegation_id: i32) -> anyhow::Result<Delegation> {
        // on conflict do update ... really does nothing, but it ensures that a record is returned
        let row = query("DELETE FROM delegations dlgs USING clients cl WHERE dlgs.client_id = cl.client_id AND dlgs.tms_identity=$1 AND dlgs.client_id=$2 AND dlgs.id=$3 RETURNING *")
            .bind(tms_identity)
            .bind(client_id)
            .bind(delegation_id)
            .fetch_one(&self.db_pool)
            .await.map_err(|error| match error {
                sqlx::Error::RowNotFound => NotFound(format!("Deletgation id {} not found for client/identity", delegation_id)).into(),
                _ => anyhow::anyhow!(error),
            })?;

        Ok(Delegation::from(&row))
    }
}

impl From<&PgRow> for Delegation {
    fn from(row: &PgRow) -> Self {
        Delegation {
            id: row.get("id"),
            client_id: row.get("client_id"),
            client_name: row.get("name"),
            rp_account: row.get("rp_account"),
            expires_at: row.get("expires_at"),
            created: row.get("created"),
            updated: row.get("updated"),
            tms_identity: row.get("tms_identity"),
            rp_id: row.get("rp_id"),
        }
    }
}

pub async fn save_delegation(
    db_pool: &PgPool,
    client_id: &str,
    rp_account: &str,
    expires_at: &DateTime<Utc>,
    tms_identity: &str,
    rp_id: &str,
) -> anyhow::Result<Delegation> {
    // on conflict do update ... really does nothing, but it ensures that a record is returned
    let delegation_id: i32 = query(
        "INSERT INTO delegations (client_id, rp_account, expires_at,
                         tms_identity, rp_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(client_id)
    .bind(rp_account)
    .bind(expires_at)
    .bind(tms_identity)
    .bind(rp_id)
    .fetch_one(db_pool)
    .await?
    .get("id");

    let row = query("SELECT dlgs.*, cl.name FROM delegations dlgs LEFT JOIN clients cl ON dlgs.client_id = cl.client_id WHERE dlgs.id = $1")
        .bind(delegation_id)
        .fetch_one(db_pool)
        .await?;

    Ok(Delegation::from(&row))
}

pub async fn list_delegations<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: &Option<String>,
    tms_identity: &str,
) -> anyhow::Result<Vec<Delegation>> {
    let rows= match client_id {
        Some(client_id) => {
            query("SELECT dlgs.*, cl.name FROM delegations dlgs LEFT JOIN clients cl ON dlgs.client_id = cl.client_id WHERE client_id = $1 AND tms_identity = $2")
                .bind(client_id)
                .bind(tms_identity)
                .fetch_all(&mut **tx)
                .await?
        }
        None => {
            query("SELECT dlgs.*, cl.name FROM delegations dlgs LEFT JOIN clients cl ON dlgs.client_id = cl.client_id WHERE tms_identity = $1")
                .bind(tms_identity)
                .fetch_all(&mut **tx)
                .await?
        }
    };

    let mut delegations = Vec::with_capacity(rows.len());
    for row in rows {
        delegations.push(Delegation::from(&row));
    }

    Ok(delegations)
}

pub async fn db_delete_delegation<'a>(
    tx: &mut PgTransaction<'a>,
    tms_identity: &str,
    client_id: &str,
    delegation_id: i32,
) -> anyhow::Result<Delegation> {
    // on conflict do update ... really does nothing, but it ensures that a record is returned
    let row = query("DELETE FROM delegations dlgs USING clients cl WHERE dlgs.client_id = cl.client_id AND dlgs.tms_identity=$1 AND dlgs.client_id=$2 AND dlgs.id=$3 RETURNING *")
        .bind(tms_identity)
        .bind(client_id)
        .bind(delegation_id)
        .fetch_one(&mut **tx)
        .await.map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Deletgation id {} not found for client/identity", delegation_id)).into(),
            _ => anyhow::anyhow!(error),
        })?;

    Ok(Delegation::from(&row))
}
