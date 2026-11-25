use crate::obj_model::institutions::Institution;
use crate::ports::institution::InstitutionRepository;

pub struct MockInstitutionRepo {
    institutions: Vec<Institution>,
    error: Option<String>,
}

impl MockInstitutionRepo {
    pub fn new(initial_state: Vec<Institution>) -> Self {
        Self { institutions: initial_state, error: None }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    fn configured_error(&self) -> Option<anyhow::Error> {
        self.error
            .as_ref()
            .map(|error| anyhow::Error::msg(error.clone()))
    }
}

#[async_trait::async_trait]
impl InstitutionRepository for MockInstitutionRepo {
    async fn list(&self) -> anyhow::Result<Vec<Institution>> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        Ok(self.institutions.clone())
    }

    async fn get_by_id(&self, institution_id: &str) -> anyhow::Result<Institution> {
        if let Some(error) = self.configured_error() {
            return Err(error);
        }

        self.institutions
            .iter()
            .find(|institution| institution.id == institution_id)
            .cloned()
            // the same error as InMemoryInstitutionRepository's, so callers see real behavior
            .ok_or_else(|| {
                tms_lib::utils::service_error::ServiceError::NotFound(format!(
                    "Institution id '{}' not found",
                    institution_id
                ))
                .into()
            })
    }
}
