use crate::{obj_model::client::Client, ports::client::ClientRepository};

pub struct MockClientRepo {
    clients: Vec<Client>,
    error: Option<String>,
}

impl MockClientRepo {
    pub fn new(initial_state: Vec<Client>) -> Self {
        Self { clients: initial_state, error: None }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    fn configured_error(&self) -> Option<anyhow::Error> {
        self.error
            .as_ref()
            .map(|error| anyhow::Error::msg(error.clone()))
    }
}

#[async_trait::async_trait]
impl ClientRepository for MockClientRepo {
    async fn get_by_id(&self, client_id: &str) -> anyhow::Result<Client> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        self.clients
            .iter()
            .find(|c| c.client_id == client_id)
            .cloned()
            .ok_or_else(|| anyhow::Error::msg("Client not found"))
    }

    async fn get_by_name(&self, client_name: &str) -> anyhow::Result<Client> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        self.clients
            .iter()
            .find(|c| c.name == client_name)
            .cloned()
            .ok_or_else(|| anyhow::Error::msg("Client not found"))
    }

    async fn get_by_credentials(&self, client_id: &str, secret: &str) -> anyhow::Result<Client> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        self.clients
            .iter()
            .find(|c| c.client_id == client_id && c.secret == secret)
            .cloned()
            .ok_or_else(|| anyhow::Error::msg("Client not found"))
    }
}
