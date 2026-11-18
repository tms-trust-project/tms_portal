use crate::obj_model;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: i32,
    pub client_id: String,
    pub name: String,
    pub enabled: bool,
    pub created: String,
    pub updated: String,
}

impl From<obj_model::client::Client> for Client {
    fn from(value: obj_model::client::Client) -> Self {
        Client {
            id: value.id,
            client_id: value.client_id,
            name: value.name,
            enabled: value.enabled,
            created: value.created.to_rfc3339(),
            updated: value.updated.to_rfc3339(),
        }
    }
}
