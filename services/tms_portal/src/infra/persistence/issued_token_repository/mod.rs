pub mod issued_tokens_dao;

use crate::obj_model::oauth::IssuedToken;
use crate::ports::issued_token::IssuedTokenRepository;
use chrono::{DateTime, Utc};
use log::info;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub struct PostgresIssuedTokenRepository {
    db_pool: PgPool,
}

impl PostgresIssuedTokenRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl From<&PgRow> for IssuedToken {
    fn from(row: &PgRow) -> Self {
        IssuedToken {
            access_token: row.get("access_token"),
            expiration: row.get("expiration"),
            revoked: row.get("revoked"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}
#[async_trait::async_trait]
impl IssuedTokenRepository for PostgresIssuedTokenRepository {
    async fn get(&self, token: &String) -> anyhow::Result<IssuedToken> {
        let row = query("select * from issued_tokens where access_token = $1")
            .bind(token)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
                _ => anyhow::anyhow!(error),
            })?;
        Ok(IssuedToken::from(&row))
    }

    async fn delete(&self, token: &String) -> anyhow::Result<IssuedToken> {
        let row = query("delete from issued_tokens where access_token = $1")
            .bind(token)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
                _ => anyhow::anyhow!(error),
            })?;
        Ok(IssuedToken::from(&row))
    }

    async fn revoke(&self, token: &String) -> anyhow::Result<IssuedToken> {
        let row =
            query("update issued_tokens set revoked = true where access_token = $1 returning *")
                .bind(token)
                .fetch_one(&self.db_pool)
                .await
                .map_err(|error| match error {
                    sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
                    _ => anyhow::anyhow!(error),
                })?;
        Ok(IssuedToken::from(&row))
    }

    async fn insert(
        &self,
        token: &String,
        expiration: &DateTime<Utc>,
    ) -> anyhow::Result<IssuedToken> {
        let row = query("insert into issued_tokens (access_token, expiration, revoked) values ($1, $2, false) returning *")
            .bind(token)
            .bind(expiration)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
                _ => anyhow::anyhow!(error),
            })?;
        Ok(IssuedToken::from(&row))
    }

    async fn cleanup(&self, expired_before: &DateTime<Utc>) -> anyhow::Result<u64> {
        let row_result = query("delete from issued_tokens where expiration < $1")
            .bind(expired_before)
            .execute(&self.db_pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
                _ => anyhow::anyhow!(error),
            })?;
        info!("Deleted {} expired tokens", &row_result.rows_affected());
        Ok(row_result.rows_affected())
    }
}
