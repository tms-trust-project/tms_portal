use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ResourceProviderAuthorizeRequest {
    pub provider_id: String,
    pub redirect_url: String,
    pub state: Option<String>,
}
