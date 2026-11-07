use chrono::{DateTime, Utc};

use crate::obj_model::delegation::Delegation;

pub struct DelegationBuilder {
    id: Option<i32>,
    client_id: Option<String>,
    client_name: Option<String>,
    rp_account: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    created: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
    tms_identity: Option<String>,
    rp_id: Option<String>,
}

impl DelegationBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            client_id: None,
            client_name: None,
            rp_account: None,
            expires_at: None,
            created: None,
            updated: None,
            tms_identity: None,
            rp_id: None,
        }
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_client_name(mut self, client_name: String) -> Self {
        self.client_name = Some(client_name);
        self
    }

    pub fn with_rp_account(mut self, rp_account: String) -> Self {
        self.rp_account = Some(rp_account);
        self
    }

    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
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

    pub fn with_tms_identity(mut self, tms_identity: String) -> Self {
        self.tms_identity = Some(tms_identity);
        self
    }

    pub fn with_rp_id(mut self, rp_id: String) -> Self {
        self.rp_id = Some(rp_id);
        self
    }

    pub fn build(&self) -> Delegation {
        let now = Utc::now();

        Delegation {
            id: self.id.unwrap_or(1),
            client_id: self
                .client_id
                .clone()
                .unwrap_or_else(|| "test-client-id".into()),
            client_name: self
                .client_name
                .clone()
                .unwrap_or_else(|| "Test Client".into()),
            rp_account: self
                .rp_account
                .clone()
                .unwrap_or_else(|| "test-rp-account".into()),
            expires_at: self.expires_at.unwrap_or(now),
            created: self.created.unwrap_or(now),
            updated: self.updated.unwrap_or(now),
            tms_identity: self
                .tms_identity
                .clone()
                .unwrap_or_else(|| "test-identity".into()),
            rp_id: self.rp_id.clone().unwrap_or_else(|| "test-rp-id".into()),
        }
    }
}
