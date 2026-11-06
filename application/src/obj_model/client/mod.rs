pub mod test_fixtures;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Client {
    #[allow(dead_code)]
    pub id: i32,
    pub client_id: String,
    #[allow(dead_code)]
    pub secret: String,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub enabled: bool,
    #[allow(dead_code)]
    pub created: DateTime<Utc>,
    #[allow(dead_code)]
    pub updated: DateTime<Utc>,
}
