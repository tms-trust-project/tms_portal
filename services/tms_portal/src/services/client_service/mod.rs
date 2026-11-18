use std::sync::Arc;

use crate::obj_model::client::Client;
use crate::ports::client::ClientRepository;

#[derive(Clone)]
pub struct ClientService {
    client_repo: Arc<dyn ClientRepository>,
}

impl ClientService {
    pub fn new(client_repo: Arc<dyn ClientRepository>) -> Self {
        Self { client_repo }
    }

    pub async fn list_clients(&self) -> anyhow::Result<Vec<Client>> {
        self.client_repo.list().await
    }

    pub async fn find_client(&self, client_id: &str) -> anyhow::Result<Client> {
        self.client_repo.get_by_id(client_id).await
    }
}

#[cfg(test)]
#[path = "client_service.test.rs"]
mod client_service_test;
