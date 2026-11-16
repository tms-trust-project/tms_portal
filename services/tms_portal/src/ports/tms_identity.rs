use crate::obj_model::tms_identity::TmsIdentity;

#[async_trait::async_trait]
pub trait TmsIdentityRepository: Send + Sync {
    async fn get(&self, tms_identity: &String) -> anyhow::Result<TmsIdentity>;

    async fn insert_if_absent(&self, tms_identity: &String) -> anyhow::Result<TmsIdentity>;
}
