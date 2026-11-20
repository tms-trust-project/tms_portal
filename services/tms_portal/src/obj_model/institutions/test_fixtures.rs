use crate::obj_model::institutions::Institution;

pub struct InstitutionBuilder {
    id: Option<String>,
    name: Option<String>,
}

impl InstitutionBuilder {
    pub fn new() -> Self {
        Self { id: None, name: None }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn build(&self) -> Institution {
        Institution {
            id: self
                .id
                .clone()
                .unwrap_or_else(|| "test-institution-id".into()),
            name: self
                .name
                .clone()
                .unwrap_or_else(|| "Test Institution".into()),
        }
    }
}
