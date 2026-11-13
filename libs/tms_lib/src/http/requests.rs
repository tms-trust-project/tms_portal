use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TmsErrorResponse {
    pub status: String,
    pub message: String,
}
