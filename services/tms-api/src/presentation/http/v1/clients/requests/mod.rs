use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ListAllowedRedirectsForClientPathParams {
    pub client_id: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct GetClientByIdPathParams {
    pub client_id: String,
}