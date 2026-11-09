use chrono::{DateTime, Utc};

use crate::obj_model::client::Client;

pub struct ClientBuilder {
    id: Option<i32>,
    client_id: Option<String>,
    secret: Option<String>,
    name: Option<String>,
    enabled: Option<bool>,
    created: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self { id: None, client_id: None, secret: None, name: None, enabled: None, created: None, updated: None }
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
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

    pub fn build(&self) -> Client {
        let now = Utc::now();

        Client {
            id: self.id.unwrap_or(1),
            client_id: self.client_id.clone().unwrap_or_else(|| "test-client-id".into()),
            secret: self.secret.clone().unwrap_or_else(|| "test-secret".into()),
            name: self.name.clone().unwrap_or_else(|| "Test Client".into()),
            enabled: self.enabled.unwrap_or(true),
            created: self.created.unwrap_or(now),
            updated: self.updated.unwrap_or(now),
        }
    }
}
