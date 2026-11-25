use crate::obj_model;
use crate::services::client_service::ClientDetail as ClientDetailOutput;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub created: String,
    pub updated: String,
}

impl From<obj_model::client::Client> for Client {
    fn from(value: obj_model::client::Client) -> Self {
        Client {
            id: value.client_id,
            name: value.name,
            enabled: value.enabled,
            created: value.created.to_rfc3339(),
            updated: value.updated.to_rfc3339(),
        }
    }
}

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct ClientDetails {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub created: String,
    pub updated: String,
    pub allowed_redirects: Vec<String>
}

impl From<ClientDetailOutput> for ClientDetails {
    fn from(value: ClientDetailOutput) -> Self {
        ClientDetails {
            id: value.client.client_id,
            name: value.client.name,
            enabled: value.client.enabled,
            created: value.client.created.to_rfc3339(),
            updated: value.client.updated.to_rfc3339(),
            allowed_redirects: value.allowed_redirects
                .iter()
                .map(|r| r.uri.clone())
                .collect()
        }
    }
}

#[derive(Debug, Hash, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct AllowedRedirect {
    pub uri: String,
    pub client_id: String,
    pub created: String,
    pub updated: String,
}

impl From<obj_model::allowed_redirect::AllowedRedirect> for AllowedRedirect {
    fn from(value: obj_model::allowed_redirect::AllowedRedirect) -> Self {
        AllowedRedirect {
            uri: value.uri,
            client_id: value.client_id,
            created: value.created.to_rfc3339(),
            updated: value.updated.to_rfc3339(),
        }
    }
}
