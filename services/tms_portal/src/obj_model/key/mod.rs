use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Deserialize, Default)]
pub struct Key {
    pub kid: String,
    pub jwt_public_key: String,
    pub jwt_private_key: String,
    #[serde(skip)]
    pub created: DateTime<Utc>,
    #[serde(skip)]
    pub updated: DateTime<Utc>,
}

impl Key {
    pub fn from_env(kid: &str, pub_var: &str, sec_var: &str) -> anyhow::Result<Key> {
        Ok(Key {
            kid: kid.into(),
            jwt_public_key: std::env::var(pub_var)?,
            jwt_private_key: std::env::var(sec_var)?,
            created: Default::default(),
            updated: Default::default(),
        })
    }
}
