use std::sync::Arc;

use crate::obj_model::institutions::Institution;
use crate::ports::institution::InstitutionRepository;

#[derive(Clone)]
pub struct InstitutionService {
    institution_repo: Arc<dyn InstitutionRepository>,
}

impl InstitutionService {
    pub fn new(institution_repo: Arc<dyn InstitutionRepository>) -> Self {
        Self { institution_repo }
    }

    pub async fn list_institutions(&self) -> anyhow::Result<Vec<Institution>> {
        self.institution_repo.list().await
    }

    pub async fn find_institution_by_id(
        &self,
        institution_id: &str,
    ) -> anyhow::Result<Institution> {
        self.institution_repo.get_by_id(institution_id).await
    }
}

#[cfg(test)]
#[path = "institution_service.test.rs"]
mod institution_service_test;
