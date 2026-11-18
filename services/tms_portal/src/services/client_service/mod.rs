use std::sync::Arc;

use crate::obj_model::allowed_redirect::AllowedRedirect;
use crate::obj_model::client::Client;
use crate::ports::allowed_redirect::AllowedRedirectRepository;
use crate::ports::client::ClientRepository;

#[derive(Clone)]
pub struct ClientService {
    client_repo: Arc<dyn ClientRepository>,
    allowed_redirect_repo: Arc<dyn AllowedRedirectRepository>,
}

impl ClientService {
    pub fn new(
        client_repo: Arc<dyn ClientRepository>,
        allowed_redirect_repo: Arc<dyn AllowedRedirectRepository>
    ) -> Self {
        Self {
            client_repo,
            allowed_redirect_repo
        }
    }

    pub async fn list_clients(&self) -> anyhow::Result<Vec<Client>> {
        self.client_repo.list().await
    }

    pub async fn find_client_by_id(&self, client_id: &str) -> anyhow::Result<ClientDetail> {
        let client = self.client_repo.get_by_id(client_id).await?;

        let redirects = self.allowed_redirect_repo
            .list_by_client_id(&client.client_id)
            .await?;

        Ok(ClientDetail {
            client,
            allowed_redirects: redirects,
        })
    }
}

pub struct ClientDetail {
    pub client: Client,
    pub allowed_redirects: Vec<AllowedRedirect>,
}

#[cfg(test)]
#[path = "client_service.test.rs"]
mod client_service_test;
