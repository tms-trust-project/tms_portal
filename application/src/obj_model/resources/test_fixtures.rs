use chrono::{DateTime, Duration, Utc};

use crate::obj_model::resources::ResourceProviderLogin;

pub struct ResourceProviderLoginBuilder {
    id: Option<i32>,
    tms_identity: Option<String>,
    enabled: Option<bool>,
    created: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
    rp_id: Option<String>,
    rp_account: Option<String>,
    last_login: Option<DateTime<Utc>>,
    rp_token: Option<String>,
    rp_token_refresh: Option<String>,
}

impl ResourceProviderLoginBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            tms_identity: None,
            enabled: None,
            created: None,
            updated: None,
            rp_id: None,
            rp_account: None,
            last_login: None,
            rp_token: None,
            rp_token_refresh: None,
        }
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_tms_identity(mut self, tms_identity: String) -> Self {
        self.tms_identity = Some(tms_identity);
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn with_created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn with_updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = Some(updated);
        self
    }

    pub fn with_rp_id(mut self, rp_id: String) -> Self {
        self.rp_id = Some(rp_id);
        self
    }

    pub fn with_rp_account(mut self, rp_account: String) -> Self {
        self.rp_account = Some(rp_account);
        self
    }

    pub fn with_last_login(mut self, last_login: DateTime<Utc>) -> Self {
        self.last_login = Some(last_login);
        self
    }

    pub fn with_rp_token(mut self, rp_token: String) -> Self {
        self.rp_token = Some(rp_token);
        self
    }

    pub fn with_rp_token_refresh(mut self, rp_token_refresh: String) -> Self {
        self.rp_token_refresh = Some(rp_token_refresh);
        self
    }

    pub fn with_unexpired_last_login(mut self, earliest_login: DateTime<Utc>) -> Self {
        let last_login = earliest_login + Duration::minutes(5);

        self.with_last_login(last_login)
    }

    pub fn build(&self) -> ResourceProviderLogin {
        let now = Utc::now();

        ResourceProviderLogin {
            id: self.id.unwrap_or(1),
            tms_identity: self
                .tms_identity
                .clone()
                .unwrap_or_else(|| "test-identity".into()),
            enabled: self.enabled.unwrap_or(true),
            created: self.created.unwrap_or(now),
            updated: self.updated.unwrap_or(now),
            rp_id: self.rp_id.clone().unwrap_or_else(|| "test-rp-id".into()),
            rp_account: self
                .rp_account
                .clone()
                .unwrap_or_else(|| "test-rp-account".into()),
            last_login: self.last_login.unwrap_or(now),
            rp_token: self
                .rp_token
                .clone()
                .unwrap_or_else(|| "test-rp-token".into()),
            rp_token_refresh: self
                .rp_token_refresh
                .clone()
                .unwrap_or_else(|| "test-rp-refresh-token".into()),
        }
    }
}
