use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug)]
pub struct TmsIdentity {
    pub seq_id: i32,
    pub tms_identity: String,
    pub created: DateTime<Utc>,
}
