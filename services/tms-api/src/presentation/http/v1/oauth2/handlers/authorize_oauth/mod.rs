use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use axum::extract::{Query, State};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use http::StatusCode;
use http::header::LOCATION;
use serde::Deserialize;
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::BadRequest;

use crate::AppState;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::oauth2_service::{AuthorizeCodeRequest, AuthorizeOutcome, authorize_code};
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{ROOT_COOKIE_PATH, STATE_COOKIE_NAME};

// ResponseType enum for authorize endpoint
#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum ResponseType {
    #[serde(rename = "code")]
    Code,
}

// Convert string to ResponseType enum
impl TryFrom<&str> for ResponseType {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            v if v.eq_ignore_ascii_case("code") => Ok(ResponseType::Code),
            _ => Err(BadRequest(format!("Unknown ResponseType {}", value))),
        }
    }
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseType::Code => write!(f, "code"),
        }
    }
}

// params for authorize request
#[derive(Debug, Deserialize)]
pub(crate) struct OAuthAuthorizeRequest {
    response_type: ResponseType,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    // an institution from /institutions.  Without one the browser goes to the portal's
    // institution picker, which sends the request back with it.
    institution: Option<String>,
}

#[axum::debug_handler]
pub async fn authorize_oauth(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<OAuthAuthorizeRequest>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Match exporession for response type to determine what kind of response is expected
    match query_params.response_type {
        // code - oauth authorization code flow
        ResponseType::Code => {
            let request = AuthorizeCodeRequest {
                client_id: &query_params.client_id,
                redirect_uri: &query_params.redirect_uri,
                state: &query_params.state,
                scope: &query_params.scope,
                institution: &query_params.institution,
            };
            let outcome = authorize_code(
                &app_state.config,
                &app_state.db_pool,
                &app_state.institution_service,
                &request,
            )
            .await?;

            match outcome {
                AuthorizeOutcome::Login { location, encoded_state } => {
                    let updated_jar = jar.add(
                        Cookie::build((STATE_COOKIE_NAME, encoded_state))
                            .path(ROOT_COOKIE_PATH)
                            .http_only(true),
                    );
                    let headers = HashMap::from([(LOCATION.to_string(), location)]);
                    Ok((
                        updated_jar,
                        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                            .headers(headers)
                            .build(),
                    ))
                }
                AuthorizeOutcome::Redirect { location } => {
                    let headers = HashMap::from([(LOCATION.to_string(), location)]);
                    Ok((jar, TmsResponse::builder(StatusCode::SEE_OTHER).headers(headers).build()))
                }
            }
        } // TODO:  at some point we probably need to handle refresh tokens
    }
}

#[cfg(test)]
#[path = "authorize_oauth.test.rs"]
mod authorize_oauth_test;
