use crate::obj_model::institutions::Institution;

#[async_trait::async_trait]
pub trait InstitutionRepository: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<Institution>>;

    async fn get_by_id(&self, institution_id: &str) -> anyhow::Result<Institution>;
}
