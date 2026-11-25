use crate::obj_model::key::Key;

#[async_trait::async_trait]
pub trait KeyRepository: Send + Sync {
    async fn get_by_id(&self, kid: &String) -> anyhow::Result<Key>;
}
