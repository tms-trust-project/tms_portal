#[cfg(test)]
pub mod test_fixtures;

use crate::obj_model::client::Client;
use crate::ports::client::ClientRepository;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, PgTransaction, Row, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub struct PostgresClientRepository {
    db_pool: PgPool,
}

impl PostgresClientRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl ClientRepository for PostgresClientRepository {
    async fn get_by_id(&self, client_id: &str) -> anyhow::Result<Client> {
        let row = query("select * from clients where client_id = $1")
            .bind(client_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => {
                    NotFound(format!("Client id {} not found", client_id)).into()
                }
                _ => anyhow::anyhow!(error),
            })?;

        Ok(Client::from(&row))
    }

    async fn get_by_name(&self, client_name: &str) -> anyhow::Result<Client> {
        let row = query("select * from clients where name = $1")
            .bind(client_name)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => {
                    NotFound(format!("Client with name '{}' not found", client_name)).into()
                }
                _ => anyhow::anyhow!(error),
            })?;

        Ok(Client::from(&row))
    }

    async fn get_by_credentials(&self, client_id: &str, secret: &str) -> anyhow::Result<Client> {
        let row = query("select * from clients where client_id = $1 and secret = $2")
            .bind(client_id)
            .bind(secret)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => {
                    NotFound(format!("Client id {} not found", client_id)).into()
                }
                _ => anyhow::anyhow!(error),
            })?;

        Ok(Client::from(&row))
    }
}

impl From<&PgRow> for Client {
    fn from(row: &PgRow) -> Self {
        Client {
            id: row.get("id"),
            client_id: row.get("client_id"),
            secret: row.get("secret"),
            name: row.get("name"),
            enabled: row.get("enabled"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}
pub async fn get_client_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: &str,
) -> anyhow::Result<Client> {
    let row = query("select * from clients where client_id = $1")
        .bind(client_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => {
                NotFound(format!("Client id {} not found", client_id)).into()
            }
            _ => anyhow::anyhow!(error),
        })?;
    Ok(Client::from(&row))
}
pub async fn get_client_by_name(db_pool: &PgPool, client_name: &str) -> anyhow::Result<Client> {
    let row = query("select * from clients where name = $1")
        .bind(client_name)
        .fetch_one(db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => {
                NotFound(format!("Client with name '{}' not found", client_name)).into()
            }
            _ => anyhow::anyhow!(error),
        })?;
    Ok(Client::from(&row))
}
pub async fn get_client_by_credentials<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: &str,
    secret: &str,
) -> anyhow::Result<Client> {
    let row = query("select * from clients where client_id = $1 and secret = $2")
        .bind(client_id)
        .bind(secret)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => {
                NotFound(format!("Client id {} not found", client_id)).into()
            }
            _ => anyhow::anyhow!(error),
        })?;
    Ok(Client::from(&row))
}
