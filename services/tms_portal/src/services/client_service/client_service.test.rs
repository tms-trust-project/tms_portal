use std::sync::Arc;

use crate::infra::persistence::client_repository::test_fixtures::MockClientRepo;
use crate::obj_model::client::test_fixtures::ClientBuilder;
use crate::services::client_service::ClientService;

#[tokio::test]
async fn lists_clients() {
    let first_client = ClientBuilder::new()
        .with_client_id("first-client".into())
        .build();
    let second_client = ClientBuilder::new()
        .with_client_id("second-client".into())
        .build();
    let service =
        ClientService::new(Arc::new(MockClientRepo::new(vec![first_client, second_client])));

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
    let service = ClientService::new(Arc::new(MockClientRepo::new(vec![client])));

    let client = service
        .find_client("requested-client")
        .await
        .expect("the client should be found");

    assert_eq!(client.client_id, "requested-client");
}
