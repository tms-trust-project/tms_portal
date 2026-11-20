use crate::obj_model;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct Institution {
    pub id: String,
    pub name: String,
}

impl From<obj_model::institutions::Institution> for Institution {
    fn from(value: obj_model::institutions::Institution) -> Self {
        Institution { id: value.id, name: value.name }
    }
}
