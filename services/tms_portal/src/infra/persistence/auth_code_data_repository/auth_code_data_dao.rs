use crate::obj_model::auth_code_data::AuthCodeData;
use crate::obj_model::identity_provider::IdentityProviderType;
use crate::utils::jwt_utils::TmsTokenClaims;
use anyhow::anyhow;
use chrono::{TimeDelta, Utc};
use serde_json::to_value;
use sqlx::{Error, PgTransaction, query};
use tms_lib::utils::service_error::ServiceError::BadRequest;

pub async fn delete_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    redirect_uri: &String,
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
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::try_from(&row)?),
        Err(Error::RowNotFound) => Err(BadRequest("Auth code data not found".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}

pub async fn insert_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    redirect_uri: &String,
    claims: &TmsTokenClaims,
    idp_id: &String,
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
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::try_from(&row)?),
        Err(error) => Err(anyhow!(error)),
    }
}
