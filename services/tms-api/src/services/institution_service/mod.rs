use std::sync::Arc;

use tms_lib::utils::service_error::ServiceError::{self, BadRequest};

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

    /*
    The Globus domain to send a login through this institution to.  An unknown, disabled
    or unconfigured institution is a bad request: the id came from the login request.
    Other repository errors pass through unchanged.
     */
    pub async fn globus_login_domain(&self, institution_id: &str) -> anyhow::Result<String> {
        let institution =
            self.institution_repo.get_by_id(institution_id).await.map_err(|error| {
                match error.downcast_ref::<ServiceError>() {
                    Some(ServiceError::NotFound(_)) => {
                        BadRequest(format!("Unknown institution '{}'", institution_id)).into()
                    }
                    _ => error,
                }
            })?;
        Ok(institution.globus_login_domain()?.to_string())
    }
}

#[cfg(test)]
#[path = "institution_service.test.rs"]
mod institution_service_test;
