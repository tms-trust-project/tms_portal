use crate::{
    obj_model::client::{
        Client,
        test_fixtures::ClientBuilder,
    },
    ports::client::ClientRepository
};

pub struct MockClientRepo {
    return_type: MockClientReturnType
}

impl MockClientRepo {
    pub fn new(return_type: MockClientReturnType) -> Self {
        Self { return_type }
    }
}

pub enum MockClientReturnType {
    Error,
    Client
}

impl MockClientReturnType {
    pub fn client(client_id: Option<&str>, client_name: Option<&str>, secret: Option<&str>) -> Client {
        let mut builder = ClientBuilder::new();

        if let Some(cid) = client_id {
            builder = builder.with_client_id(cid.into());
        }

        if let Some(name) = client_name {
            builder = builder.with_name(name.into());
        }

        if let Some(sec) = secret {
            builder = builder.with_secret(sec.into());
        }

        builder.build()
    }

    pub fn error(msg: Option<&str>) -> anyhow::Error {
        anyhow::Error::msg(msg.unwrap_or("Error").to_string())
    }
}

#[async_trait::async_trait]
impl ClientRepository for MockClientRepo {
    async fn get_by_id(&self, client_id: &str) -> anyhow::Result<Client> {
        match self.return_type {
            MockClientReturnType::Client => Ok(MockClientReturnType::client(Some(client_id), None, None)),
            MockClientReturnType::Error => Err(MockClientReturnType::error(Some("Failed to get client by client_id")))
        }
    }

    async fn get_by_name(&self, client_name: &str) -> anyhow::Result<Client> {
        match self.return_type {
            MockClientReturnType::Client => Ok(MockClientReturnType::client(None, Some(client_name), None)),
            MockClientReturnType::Error => Err(MockClientReturnType::error(Some("Failed to get client by name")))
        }
    }

    async fn get_by_credentials(&self, client_id: &str, secret: &str) -> anyhow::Result<Client> {
        match self.return_type {
            MockClientReturnType::Client => Ok(MockClientReturnType::client(Some(client_id), None, Some(secret))),
            MockClientReturnType::Error => Err(MockClientReturnType::error(Some("Failed to get client by client_id and secret")))
        }
    }
}