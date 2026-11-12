use axum::Json;
use axum::extract::State;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::BadRequest;

use crate::AppState;
use crate::api::http::v1::responses::tms_response::TmsResponse;
use crate::services::oauth2_service::{TokenResponse, get_access_token_from_code};
use crate::services::resource_service::AccessToken;
use crate::utils::app_error::AppError;

// Grant Type Enum for tokens request
#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

// Convert string to GrantType enum (ignore case)
impl TryFrom<&str> for GrantType {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            v if v.eq_ignore_ascii_case("authorization_code") => Ok(GrantType::AuthorizationCode),
            v if v.eq_ignore_ascii_case("refresh_token") => Ok(GrantType::RefreshToken),
            _ => Err(BadRequest(format!("Unknown GrantType {}", value))),
        }
    }
}

// params for tokens request
#[derive(Debug, Deserialize)]
pub(crate) struct OAuthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: GrantType,
    pub redirect_uri: String,
    pub code: Option<String>,
    #[allow(dead_code)]
    pub refresh_token: Option<String>,
}

// tokens response
#[derive(Debug, Serialize)]
pub(crate) struct OAuthTokenResponse {
    pub access_token: AccessToken,
}

impl From<TokenResponse> for OAuthTokenResponse {
    fn from(value: TokenResponse) -> Self {
        let access_token = AccessToken {
            access_token: value.access_token,
            expires_in: value.expires_in,
            expires_at: value.expires_at,
            id_token: value.id_token,
            jti: value.jti,
        };

        OAuthTokenResponse { access_token }
    }
}

// hanlder for tokens request
#[axum::debug_handler]
pub async fn create_oauth_token(
    State(app_state): State<AppState>,
    Json(oauth_token_request): Json<OAuthTokenRequest>,
) -> anyhow::Result<TmsResponse<OAuthTokenResponse>, AppError> {
    match oauth_token_request.grant_type {
        // handle authorization code flow
        GrantType::AuthorizationCode => {
            if let Some(code) = &oauth_token_request.code {
                let token_response = get_access_token_from_code(
                    &app_state.config,
                    &app_state.db_pool,
                    &oauth_token_request.client_id,
                    &oauth_token_request.client_secret,
                    code,
                    &oauth_token_request.redirect_uri,
                )
                .await?;
                Ok(TmsResponse::builder(StatusCode::OK)
                    .entity(OAuthTokenResponse::from(token_response))
                    .build())
            } else {
                Err(BadRequest("Authorization code was not provided".to_string()).into())
            }
        }

        _ => Err(BadRequest("Invalid grant type".to_string()).into()),
    }
}

#[cfg(test)]
#[path = "create_oauth_token.test.rs"]
mod create_oauth_token_test;
