pub mod auth_code_data_dao;

use crate::obj_model::auth_code_data::AuthCodeData;
use crate::obj_model::identity_provider::IdentityProviderType;
use crate::ports::auth_code_data::AuthCodeDataRepository;
use crate::utils::jwt_utils::TmsTokenClaims;
use anyhow::anyhow;
use chrono::{TimeDelta, Utc};
use serde_json::to_value;
use sqlx::postgres::PgRow;
use sqlx::{Error, PgPool, Row, query};
use tms_lib::utils::service_error::ServiceError::BadRequest;

pub struct PostgresAuthCodeDataRepository {
    db_pool: PgPool,
}

impl PostgresAuthCodeDataRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

impl TryFrom<&PgRow> for AuthCodeData {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("idp_type");
        Ok(AuthCodeData {
            auth_code: row.get("auth_code"),
            client_id: row.get("client_id"),
            idp_id: row.get("idp_id"),
            redirect_uri: row.get("redirect_uri"),
            idp_type: provider.parse::<IdentityProviderType>()?,
            claims: row.get("claims"),
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}

#[async_trait::async_trait]
impl AuthCodeDataRepository for PostgresAuthCodeDataRepository {
    async fn delete(
        &self,
        auth_code: &str,
        client_id: &str,
        redirect_uri: &str,
        expiration: TimeDelta,
    ) -> anyhow::Result<AuthCodeData> {
        let current_time = Utc::now();
        let earliest_good_date_time = current_time - expiration;
        match query(
            "DELETE FROM auth_code_data WHERE auth_code = $1 AND client_id = $2 AND redirect_uri = $3 AND updated > $4 returning *",
        )
            .bind(auth_code)
            .bind(client_id)
            .bind(redirect_uri)
            .bind(earliest_good_date_time)
            .fetch_one(&self.db_pool)
            .await {
            Ok(row) => Ok(AuthCodeData::try_from(&row)?),
            Err(Error::RowNotFound) => Err(BadRequest("Auth code data not found".to_string()).into()),
            Err(error) => Err(anyhow!(error)),
        }
    }

    async fn insert(
        &self,
        auth_code: &str,
        client_id: &str,
        redirect_uri: &str,
        claims: &TmsTokenClaims,
        idp_id: &str,
        idp_type: &IdentityProviderType,
    ) -> anyhow::Result<AuthCodeData> {
        let claims_json = to_value(claims)?;
        match query(
            "insert into auth_code_data (auth_code, client_id, redirect_uri, claims, idp_id, idp_type) VALUES ($1, $2, $3, $4, $5, $6) returning *",
        )
            .bind(auth_code)
            .bind(client_id)
            .bind(redirect_uri)
            .bind(claims_json)
            .bind(idp_id)
            .bind(idp_type.to_string())
            .fetch_one(&self.db_pool)
            .await {
            Ok(row) => Ok(AuthCodeData::try_from(&row)?),
            Err(error) => Err(anyhow!(error)),
        }
    }
}
