use crate::obj_model::key::Key;
use sqlx::{PgTransaction, query};
use tms_lib::utils::service_error::ServiceError::NotFound;

pub async fn get_key_by_id<'a>(tx: &mut PgTransaction<'a>, kid: &String) -> anyhow::Result<Key> {
    let row = query("select * from keys where kid = $1")
        .bind(kid)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Key kid {} not found", kid)).into(),
            _ => anyhow::anyhow!(error),
        })?;
    Ok(Key::from(&row))
}
