use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub redirect_uri: String,
    pub client_return_uri: Option<String>,
    pub client_name: Option<String>,
}
