use std::sync::Arc;

use chrono::Utc;

use crate::infra::persistence::client_repository::test_fixtures::MockClientRepo;
use crate::obj_model::allowed_redirect::AllowedRedirect;
use crate::obj_model::client::test_fixtures::ClientBuilder;
use crate::ports::allowed_redirect::AllowedRedirectRepository;
use crate::services::client_service::ClientService;

struct MockAllowedRedirectRepo {
    redirects: Vec<AllowedRedirect>,
}

impl MockAllowedRedirectRepo {
    fn new(redirects: Vec<AllowedRedirect>) -> Self {
        Self { redirects }
    }
}

#[async_trait::async_trait]
impl AllowedRedirectRepository for MockAllowedRedirectRepo {
    async fn list_by_client_id(&self, client_id: &str) -> anyhow::Result<Vec<AllowedRedirect>> {
        Ok(self
            .redirects
            .iter()
            .filter(|redirect| redirect.client_id == client_id)
            .cloned()
            .collect())
    }

    async fn get_by_client_id_and_uri(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<AllowedRedirect> {
        self.redirects
            .iter()
            .find(|redirect| redirect.client_id == client_id && redirect.uri == redirect_uri)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Allowed redirect not found"))
    }

    async fn get_by_client_name_and_uri(
        &self,
        _client_name: &str,
        _redirect_uri: &str,
    ) -> anyhow::Result<AllowedRedirect> {
        Err(anyhow::anyhow!("Client-name lookup is not configured for this test repository"))
    }
}

#[tokio::test]
async fn lists_clients() {
    let first_client = ClientBuilder::new()
        .with_client_id("first-client".into())
        .build();
    let second_client = ClientBuilder::new()
        .with_client_id("second-client".into())
        .build();
    let service = ClientService::new(
        Arc::new(MockClientRepo::new(vec![first_client, second_client])),
        Arc::new(MockAllowedRedirectRepo::new(vec![])),
    );

    let clients = service
        .list_clients()
        .await
        .expect("clients should be listed");

    assert_eq!(clients.len(), 2);
    assert_eq!(clients[0].client_id, "first-client");
    assert_eq!(clients[1].client_id, "second-client");
}

#[tokio::test]
async fn finds_client_by_client_id() {
    let client = ClientBuilder::new()
        .with_client_id("requested-client".into())
        .build();
    let now = Utc::now();
    let redirect = AllowedRedirect {
        uri: "https://example.com/oauth/callback".into(),
        client_id: "requested-client".into(),
        created: now,
        updated: now,
    };
    let service = ClientService::new(
        Arc::new(MockClientRepo::new(vec![client])),
        Arc::new(MockAllowedRedirectRepo::new(vec![redirect])),
    );

    let client_detail = service
        .find_client_by_id("requested-client")
        .await
        .expect("the client should be found");

    assert_eq!(client_detail.client.client_id, "requested-client");
    assert_eq!(client_detail.allowed_redirects.len(), 1);
    assert_eq!(client_detail.allowed_redirects[0].uri, "https://example.com/oauth/callback");
}
