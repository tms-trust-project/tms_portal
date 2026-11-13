use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddDelegationRequestBody {
    pub client_name: String,
    pub resource_provider_id: String,
    pub resource_provider_account: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ListDelegationByClientPathParams {
    pub client_name: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct DeleteDelegationPathParams {
    pub delegation_id: i32,
    pub client_name: String,
}
