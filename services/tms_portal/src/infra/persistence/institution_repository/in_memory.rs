use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use tms_lib::utils::service_error::ServiceError::NotFound;

use crate::obj_model::institutions::Institution;
use crate::ports::institution::InstitutionRepository;

pub struct InMemoryInstitutionRepository {
    institutions: Vec<Institution>,
}

impl InMemoryInstitutionRepository {
    pub fn new(config_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_path = config_path.as_ref();
        let config = fs::read_to_string(config_path).with_context(|| {
            format!("Unable to read institution configuration from '{}'", config_path.display())
        })?;

        Self::from_json(&config)
    }

    fn from_json(config: &str) -> anyhow::Result<Self> {
        let configuration: InstitutionsConfiguration = serde_json::from_str(config)
            .context("Unable to deserialize institution configuration")?;
        Ok(Self { institutions: configuration.institutions })
    }
}

#[async_trait::async_trait]
impl InstitutionRepository for InMemoryInstitutionRepository {
    async fn list(&self) -> anyhow::Result<Vec<Institution>> {
        Ok(self.institutions.clone())
    }

    async fn get_by_id(&self, institution_id: &str) -> anyhow::Result<Institution> {
        self.institutions
            .iter()
            .find(|institution| institution.id == institution_id)
            .cloned()
            .ok_or_else(|| {
                NotFound(format!("Institution id '{}' not found", institution_id)).into()
            })
    }
}

#[derive(Deserialize)]
struct InstitutionsConfiguration {
    institutions: Vec<Institution>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn loads_institutions_from_configuration() {
        let repository = InMemoryInstitutionRepository::from_json(
            r#"{
                "institutions": [
                    {
                        "id": "utexas",
                        "name": "The University of Texas at Austin",
                        "enabled": true,
                        "idp_config": {
                            "globus": {
                                "redirect_idp_domain": "utexas.edu",
                                "idp_logout_url": "https://enterprise.login.utexas.edu/idp/profile/Logout"
                            }
                        }
                    }
                ]
            }"#,
        )
        .unwrap();

        let institutions = repository.list().await.unwrap();

        assert_eq!(institutions.len(), 1);
        assert_eq!(institutions[0].id, "utexas");
        assert!(institutions[0].enabled);
        assert_eq!(
            institutions[0]
                .idp_config
                .globus
                .as_ref()
                .unwrap()
                .redirect_idp_domain,
            "utexas.edu"
        );
    }

    #[tokio::test]
    async fn returns_not_found_for_an_unknown_institution() {
        let repository =
            InMemoryInstitutionRepository::from_json(r#"{"institutions": []}"#).unwrap();

        let error = repository.get_by_id("missing").await.unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Institution id 'missing' not found")
        );
    }
}
