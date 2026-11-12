use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Context;
use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::ServiceError::{BadRequest, Internal};
use url::Url;

use crate::AppState;
use crate::presentation::http::v1::login::requests::AuthorizeRequest;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::infra::persistence::identity_provider_repository::get_login_provider_by_id;
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{
    CLIENT_ID_TMS, OAuth2State, ROOT_COOKIE_PATH, STATE_COOKIE_NAME,
};
use crate::utils::redirect_utils::{
    check_allowed_redirects, check_allowed_redirects_by_client_name,
};
use crate::utils::state_utils::encode_state;

/*
Accepts a form with information about the login (idp and redirect uri).  There's an
associated table of redirect uris that are allowed.  The redirect url must be in that
table.  This tells us where to redirect back to after the login.  Having multiple allowed
redirects allows for easier debugging - redirect back to localhost if debugging, etc.
 */
#[axum::debug_handler]
pub async fn login(
    State(app_state): State<AppState>,
    jar: CookieJar,
    form_data: Form<AuthorizeRequest>,
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    // Portal login will always be the tms client id
    let mut tx = app_state.db_pool.begin().await?;
    let client_id = String::from(CLIENT_ID_TMS);
    let configuration = &app_state.config;
    // always use the login idp from our configuration
    let login_idp_id = &configuration.oauth_config.login_oauth_provider;
    let login_idp = get_login_provider_by_id(&mut tx, login_idp_id).await;

    // we will not use this value, but we need to make sure this redirect uri is in the database.
    check_allowed_redirects(&mut tx, &client_id, &form_data.redirect_uri)
        .await
        .with_context(|| "Invalid login redirect")?;
    tx.commit().await?;

    match login_idp {
        Ok(idp) => {
            let mut redirect_uri = Url::parse(&form_data.redirect_uri)?;
            if let Some(client_return_uri) = &form_data.client_return_uri {
                if let Some(return_client_name) = &form_data.client_name {
                    // check to make sure the client name / return uri are in the allowed list
                    let mut tx = app_state.db_pool.begin().await?;
                    check_allowed_redirects_by_client_name(
                        &mut tx,
                        return_client_name,
                        client_return_uri,
                    )
                    .await
                    .with_context(|| "Invalid redirect url for login")?;
                    tx.commit().await?;
                } else {
                    // if the client name was not specified, it's an error
                    return Err(BadRequest(
                        "Unable to lookup client return uri.  No client name specified".to_string(),
                    )
                    .into());
                }
                redirect_uri
                    .query_pairs_mut()
                    .append_pair("client_return_uri", client_return_uri.as_str());
            }
            // It's ok if they give the client_name but no return uri
            if let Some(client_name) = &form_data.client_name {
                redirect_uri
                    .query_pairs_mut()
                    .append_pair("client_name", client_name.as_str());
            }

            let oauth_state = OAuth2State {
                tms_identity: String::default(), // we don't have a tms identity at this point
                client_id,
                idp_id: idp.id,
                redirect_uri: redirect_uri.to_string(),
                exp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs()
                    + 300,
                nonce: generate_nonce(),
                client_state: None,
            };

            let encoded_state = match encode_state(&app_state.config.state_key, oauth_state).await {
                Ok(state_string) => state_string,
                Err(error) => return Err(Internal(error.to_string()).into()),
            };

            let callback_url = &configuration
                .http_config
                .get_identity_provider_callback_url();
            let encoded_nonce = BASE64_STANDARD.encode(generate_nonce().to_ne_bytes());
            let mut query_params = vec![
                ("response_type", "code"),
                ("client_id", &idp.client_id),
                ("redirect_uri", callback_url),
                ("state", &encoded_state),
                ("nonce", &encoded_nonce),
                ("access_type", "offline"),
            ];

            if let Some(scope) = &idp.scope {
                query_params.push(("scope", scope.as_str()))
            }

            let location = Url::parse_with_params(&idp.identity_redirect_url, query_params)?;

            let updated_jar = jar.add(
                Cookie::build((STATE_COOKIE_NAME, encoded_state))
                    .path(ROOT_COOKIE_PATH)
                    .http_only(true),
            );

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            let creds = format!("{}:{}", idp.client_id, idp.client_secret);
            let authorization = format!("Basic {}", BASE64_STANDARD.encode(&creds));
            headers.insert("Authorization".to_string(), authorization);

            Ok((
                updated_jar,
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            ))
        }

        Err(error) => Err(BadRequest(error.to_string()).into()),
    }
}
