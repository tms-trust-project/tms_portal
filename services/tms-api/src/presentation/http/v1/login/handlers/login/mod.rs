use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Context;
use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use log::warn;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::ServiceError;
use url::Url;

use crate::AppState;
use crate::infra::persistence::identity_provider_repository::identity_provider_dao::get_login_provider_by_id;
use crate::obj_model::identity_provider::IdentityProviderType;
use crate::presentation::http::v1::login::requests::AuthorizeRequest;
use crate::presentation::http::v1::utils::tms_response::TmsResponse;
use crate::services::login_service::{LoginRejection, get_login_authorize_url, get_signin_url};
use crate::utils::app_error::AppError;
use crate::utils::oauth2_authorization_code_utils::{
    CLIENT_ID_TMS, OAuth2State, ROOT_COOKIE_PATH, STATE_COOKIE_NAME,
};
use crate::utils::redirect_utils::{
    check_allowed_redirects, check_allowed_redirects_by_client_name,
};
use crate::utils::state_utils::encode_state;

type LoginResponse = (CookieJar, TmsResponse<()>);

/*
Accepts a form with information about the login (idp and redirect uri).  There's an
associated table of redirect uris that are allowed.  The redirect url must be in that
table.  This tells us where to redirect back to after the login.  Having multiple allowed
redirects allows for easier debugging - redirect back to localhost if debugging, etc.
Every login names an institution (from /institutions).  A login that can't start (no or
an unusable institution, a bad redirect, the login idp missing)
goes back to the sign in page with ?error=<code>, which says why.
 */
#[axum::debug_handler]
pub async fn login(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Form(form_data): Form<AuthorizeRequest>,
) -> Result<LoginResponse, AppError> {
    let rejection = match start_login(&app_state, jar.clone(), &form_data).await {
        Ok(Ok(response)) => return Ok(response),
        Ok(Err(rejection)) => rejection,
        Err(error) => {
            warn!("Unable to start a login: {error:#}");
            LoginRejection::LoginFailed
        }
    };
    // 303, so the browser loads the sign in page with a GET (the login may have been a POST)
    let location = get_signin_url(&app_state.config.http_config, Some(rejection));
    let headers = HashMap::from([(LOCATION.to_string(), location)]);
    Ok((jar, TmsResponse::builder(StatusCode::SEE_OTHER).headers(headers).build()))
}

async fn start_login(
    app_state: &AppState,
    jar: CookieJar,
    form_data: &AuthorizeRequest,
) -> anyhow::Result<Result<LoginResponse, LoginRejection>> {
    // Portal login will always be the tms client id
    let client_id = String::from(CLIENT_ID_TMS);
    let configuration = &app_state.config;

    // always use the login idp from our configuration
    let mut tx = app_state.db_pool.begin().await?;
    let login_idp_id = &configuration.oauth_config.login_oauth_provider;
    let idp = match get_login_provider_by_id(&mut tx, login_idp_id).await {
        Ok(idp) => idp,
        Err(error) => {
            warn!("Unable to get the login idp {login_idp_id}: {error:#}");
            return Ok(Err(LoginRejection::LoginUnavailable));
        }
    };

    // we will not use this value, but we need to make sure this redirect uri is in the database.
    check_allowed_redirects(&mut tx, &client_id, &form_data.redirect_uri)
        .await
        .with_context(|| "Invalid login redirect")?;
    tx.commit().await?;

    // Every login goes through an approved institution, straight to its own identity
    // provider: Globus is given the institution's redirect_idp_domain, and the callback
    // then requires the identity to be from it.  Checked before anything else, so a bad
    // institution never starts a login.
    let Some(institution_id) = &form_data.institution else {
        warn!("Login started without an institution");
        return Ok(Err(LoginRejection::InstitutionRequired));
    };
    if idp.identity_provider_type != IdentityProviderType::Globus {
        warn!("Institution logins need a Globus login idp, and {0} isn't one", idp.id);
        return Ok(Err(LoginRejection::LoginUnavailable));
    }
    let login_domain = match app_state.institution_service.globus_login_domain(institution_id).await
    {
        Ok(domain) => domain,
        Err(error)
            if matches!(error.downcast_ref::<ServiceError>(), Some(ServiceError::BadRequest(_))) =>
        {
            warn!("Login started with an unusable institution: {error:#}");
            return Ok(Err(LoginRejection::UnapprovedInstitution));
        }
        Err(error) => return Err(error),
    };

    let mut redirect_uri = Url::parse(&form_data.redirect_uri)?;
    if let Some(client_return_uri) = &form_data.client_return_uri {
        let Some(return_client_name) = &form_data.client_name else {
            // if the client name was not specified, it's an error
            warn!("Unable to lookup client return uri.  No client name specified");
            return Ok(Err(LoginRejection::LoginFailed));
        };
        // check to make sure the client name / return uri are in the allowed list
        let mut tx = app_state.db_pool.begin().await?;
        check_allowed_redirects_by_client_name(&mut tx, return_client_name, client_return_uri)
            .await
            .with_context(|| "Invalid redirect url for login")?;
        tx.commit().await?;
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

    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    // sent to Globus as the OIDC nonce, and checked against the id token's
    let nonce = generate_nonce();
    let oauth_state = OAuth2State {
        tms_identity: String::default(), // we don't have a tms identity at this point
        client_id,
        idp_id: idp.id.clone(),
        redirect_uri: redirect_uri.to_string(),
        exp: now + 300,
        nonce,
        client_state: None,
        institution_id: Some(institution_id.clone()),
        login_started: now,
    };
    let encoded_state = encode_state(&configuration.state_key, oauth_state).await?;

    let callback_url = &configuration.http_config.get_identity_provider_callback_url();
    let location = get_login_authorize_url(
        &idp,
        callback_url,
        &encoded_state,
        &nonce.to_string(),
        &login_domain,
    )?;

    let updated_jar = jar.add(
        Cookie::build((STATE_COOKIE_NAME, encoded_state))
            .path(ROOT_COOKIE_PATH)
            .http_only(true)
            .secure(configuration.http_config.uses_https()),
    );

    // This response goes to the browser, so it must not carry the idp's client secret.  The
    // secret is only sent server to server, when the code is exchanged for a token.
    let headers = HashMap::from([(LOCATION.to_string(), location.to_string())]);

    Ok(Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    )))
}
