use crate::obj_model::client::Client;

#[async_trait::async_trait]
pub trait ClientRepository: Send + Sync {
    async fn get_by_id(&self, client_id: &str) -> anyhow::Result<Client>;

    async fn get_by_name(&self, client_name: &str) -> anyhow::Result<Client>;

    async fn get_by_credentials(&self, client_id: &str, secret: &str) -> anyhow::Result<Client>;
}
