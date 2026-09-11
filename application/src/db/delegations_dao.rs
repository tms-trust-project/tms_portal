use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::delegation::Delegation;

impl From<&PgRow> for Delegation {
    fn from(row: &PgRow) -> Self {
        Delegation {
            id: row.get("id"),
            client_id: row.get("client_id"),
            rp_account: row.get("rp_account"),
            expires_at: row.get("expires_at"),
            created: row.get("created"),
            updated: row.get("updated"),
            tms_identity: row.get("tms_identity"),
            rp_id: row.get("rp_id"),
        }
    }
}
pub async fn db_insert_delegation<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: & String, rp_account: & String, expires_at: &DateTime<Utc>,
    tms_identity: &String, rp_id: &String) -> anyhow::Result<Delegation> {
    // on conflict do update ... really does nothing, but it ensures that a record is returned
    let row = query("INSERT INTO delegations (client_id, rp_account, expires_at,
                         tms_identity, rp_id) VALUES ($1, $2, $3, $4, $5) RETURNING *")
        .bind(client_id)
        .bind(rp_account)
        .bind(expires_at)
        .bind(tms_identity)
        .bind(rp_id)
        .fetch_one(&mut **tx)
        .await?;
    Ok(Delegation::from(&row))
}
pub async fn db_get_delegations<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: & Option<String>, tms_identity: &String) -> anyhow::Result<Vec<Delegation>> {

    let rows= match client_id {
        Some(client_id) => {
            query("SELECT * FROM delegations WHERE client_id = $1 AND tms_identity = $2")
                .bind(client_id)
                .bind(tms_identity)
                .fetch_all(&mut **tx)
                .await?
        }
        None => {
            query("SELECT * FROM delegations WHERE tms_identity = $1")
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
    tms_identity:&String, client_id: & String, delegation_id: i32) -> anyhow::Result<Delegation> {
    // on conflict do update ... really does nothing, but it ensures that a record is returned
    let row = query("DELETE FROM delegations WHERE tms_identity=$1 AND client_id=$2 AND id=$3 RETURNING *")
        .bind(tms_identity)
        .bind(client_id)
        .bind(delegation_id)
        .fetch_one(&mut **tx)
        .await?;
    Ok(Delegation::from(&row))
}
