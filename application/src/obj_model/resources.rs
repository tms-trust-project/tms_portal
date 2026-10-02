use chrono::{DateTime, Utc};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceProviderLogin {
    pub id:i32,
    pub tms_identity:String,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
    pub rp_id:String,
    pub rp_account:String,
    pub last_login:DateTime<Utc>,
    pub rp_token: String,
    pub rp_token_refresh: String
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Resource {
    pub id: i32,
    pub resource_local_id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub rp_id: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Username {
    pub id: i32,
    pub resource_id: i32,
    pub resource_provider_login_id: i32,
    pub username: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLink {
    pub id:i32,
    pub tms_identity:String,
    pub rp_account:String,
    pub rp_id:String,
    pub rp_name:String,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}
