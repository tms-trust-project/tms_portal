use crate::db::auth_code_data_dao::{db_delete_auth_code_data, db_insert_auth_code_data};
use crate::db::client_dao::db_get_client_by_credentials;
use crate::db::identity_provider_dao::db_get_login_provider_by_id;
use crate::obj_model::identity_provider::IdentityProvider;
use crate::services::login_service::{
    CallbackCheck, LoginIdentity, LoginRejection, LoginTarget, check_login_callback,
    epoch_seconds_now, get_login_authorize_url, get_login_identity, get_login_target,
    get_signin_url,
};
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{JwtClaims, get_tms_token_claims, make_auth_token};
use crate::utils::oauth2_authorization_code_utils::{LoginCallbackQueryParams, OAuth2State};
use crate::utils::redirect_utils::check_allowed_redirects;
use crate::utils::state_utils::encode_state;
use anyhow::Context;
use chrono::TimeDelta;
use rand::RngExt;
use rand::distr::Alphanumeric;
use rand::rngs::ThreadRng;
use serde_json::{Value, from_value};
use sqlx::PgPool;
use std::time::SystemTime;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::ServiceError::Internal;
use url::Url;

pub struct AuthorizationResult {
    pub location: String,
    // None when the login can't start and the client is sent an error instead
    pub encoded_state: Option<String>,
}

pub struct TokenResponse {
    pub access_token: String,
    pub expires_at: String,
    pub expires_in: i64,
    pub id_token: String,
    pub jti: String,
}

pub struct AuthorizationCallbackResult {
    pub location: String,
}

fn generate_code() -> String {
    ThreadRng::default()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub async fn authorize_code(
    configuration: &Configuration,
    db_pool: &PgPool,
    state: &Option<String>,
    client_id: &str,
    redirect_uri: &str,
    institution_id: Option<&str>,
) -> anyhow::Result<AuthorizationResult> {
    // Check redirect uri - this fails if the redirect doesnt exist.  It's checked first: an
    // error can only be sent to a redirect uri the client registered (RFC 6749, 4.1.2.1).
    let mut tx = db_pool.begin().await?;
    check_allowed_redirects(&mut tx, client_id, redirect_uri)
        .await
        .with_context(|| "OAuth redirect is invalid")?;
    tx.commit().await?;

    // the login idp, and the approved institution the client asked to steer the login to.
    // Without one the login can't start, so tell the client why.
    let login_target = match get_login_target(configuration, db_pool, institution_id).await {
        Ok(login_target) => login_target,
        Err(rejection) => {
            return Ok(AuthorizationResult {
                location: error_redirect_url(redirect_uri, state, rejection)?.to_string(),
                encoded_state: None,
            });
        }
    };

    // encode our state
    let encoded_state =
        get_state(configuration, client_id, redirect_uri, &login_target, state).await?;

    // compute the location url that will be used for the redirect
    let location = get_login_authorize_url(
        &login_target,
        &configuration.http_config.get_oauth_provider_callback_url(),
        &encoded_state,
    )?;

    Ok(AuthorizationResult {
        location: location.to_string(),
        encoded_state: Some(encoded_state),
    })
}

pub async fn generate_code_and_redirect(
    pool: &PgPool,
    configuration: &Configuration,
    identity_provider: &IdentityProvider,
    state: &OAuth2State,
    claims: &JwtClaims,
) -> anyhow::Result<Url> {
    let auth_code = generate_code();

    let mut tms_token_claims = get_tms_token_claims(
        configuration,
        &state.client_id,
        &identity_provider.id,
        &identity_provider.identity_provider_type,
        claims,
    )
    .await?;
    // remembered so logout can end the institution's IdP session too
    tms_token_claims.tms_institution = state.institution_id.clone().map(Value::from);

    let mut tx = pool.begin().await?;
    db_insert_auth_code_data(
        &mut tx,
        &auth_code,
        &state.client_id,
        &state.redirect_uri,
        &tms_token_claims,
        &identity_provider.id,
        &identity_provider.identity_provider_type,
    )
    .await?;
    tx.commit().await?;

    let mut location = Url::parse(&state.redirect_uri)?;
    if let Some(state) = &state.client_state {
        location.query_pairs_mut().append_pair("state", state);
    }
    location.query_pairs_mut().append_pair("code", &auth_code);
    Ok(location)
}

/*
The login was rejected, cancelled or failed, so the client gets no code: send the browser
back to it with an error (access_denied, or server_error when the login failed), as an OAuth2
authorization server does (RFC 6749, section 4.1.2.1).
 */
fn get_error_redirect(state: &OAuth2State, rejection: LoginRejection) -> anyhow::Result<Url> {
    error_redirect_url(&state.redirect_uri, &state.client_state, rejection)
}

// the client's (already validated) redirect uri, with the error and the client's state
fn error_redirect_url(
    redirect_uri: &str,
    client_state: &Option<String>,
    rejection: LoginRejection,
) -> anyhow::Result<Url> {
    let mut location = Url::parse(redirect_uri)?;
    {
        let mut query_pairs = location.query_pairs_mut();
        query_pairs.append_pair("error", rejection.oauth2_error());
        query_pairs.append_pair("error_description", rejection.description());
        if let Some(client_state) = client_state {
            query_pairs.append_pair("state", client_state);
        }
    }
    Ok(location)
}

async fn get_state(
    configuration: &Configuration,
    client_id: &str,
    redirect_uri: &str,
    login_target: &LoginTarget,
    state: &Option<String>,
) -> anyhow::Result<String> {
    // populate state structure
    let oauth_state = OAuth2State {
        tms_identity: String::default(), // we don't have a tms identity at this point
        client_id: client_id.to_owned(),
        idp_id: login_target.idp.id.clone(),
        redirect_uri: redirect_uri.to_owned(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            // TODO:  This should be a config setting
            + 300,
        nonce: generate_nonce(),
        client_state: state.clone(),
        institution_id: Some(login_target.institution.id.clone()),
        login_started: Some(epoch_seconds_now()?),
    };

    // encode state structure
    match encode_state(configuration, oauth_state)
        .await
        .context("Unable to encode state")
    {
        Ok(state_string) => Ok(state_string),
        Err(error) => Err(Internal(error.to_string()).into()),
    }
}

pub async fn get_access_token_from_code(
    configuration: &Configuration,
    db_pool: &PgPool,
    client_id: &String,
    client_secret: &String,
    code: &String,
    redirect_uri: &String,
) -> anyhow::Result<TokenResponse> {
    // validate client id
    let mut tx = db_pool.begin().await?;
    let client = db_get_client_by_credentials(&mut tx, client_id, client_secret).await?;

    // validate redirect uri
    check_allowed_redirects(&mut tx, &client.client_id, redirect_uri)
        .await
        .with_context(|| "OAuth redirect is invalid")?;
    // TODO: get time delta fron config (how recently the auth code must have been issued)
    let time_delta = TimeDelta::seconds(30);

    // this will remove the auth code data, but return it.  It's only valid for a single use, so we
    // don't want it hanging around.
    // TODO: we could invalidate the record insted if we think it might help debuging / tracking errors down.  They would still need to be removed at some point, but we could do that once a day or whatever.  I prefer just deleting them I think.
    let auth_code_data =
        db_delete_auth_code_data(&mut tx, code, client_id, redirect_uri, time_delta).await?;
    // let idp = db_get_login_provider_by_id(&mut tx, &auth_code_data.idp_id).await?;
    tx.commit().await?;

    let tms_token_claims = from_value(auth_code_data.claims)?;
    let tms_token_string = make_auth_token(configuration, db_pool, &tms_token_claims).await?;

    Ok(TokenResponse {
        access_token: tms_token_string.clone(),
        expires_at: tms_token_claims.get_expires_at()?,
        expires_in: tms_token_claims.get_expires_in()?,
        // TODO: why are we creating an empty `id_token`
        id_token: String::from(""),
        jti: tms_token_claims.get_jti()?,
    })
}

pub async fn process_authorization_callback(
    configuration: &Configuration,
    db_pool: &PgPool,
    query: &LoginCallbackQueryParams,
    cookie_state: Option<&str>,
) -> anyhow::Result<AuthorizationCallbackResult> {
    // redirect browser back to the client (taken from state - validated in authorize step).
    let (decoded_internal_state, code) =
        match check_login_callback(configuration, query, cookie_state).await {
            CallbackCheck::Valid { state, code } => (state, code),
            // the client's redirect uri comes from the state, so it can be told
            CallbackCheck::Rejected {
                state: Some(state),
                rejection,
            } => {
                return Ok(AuthorizationCallbackResult {
                    location: get_error_redirect(&state, rejection)?.to_string(),
                });
            }
            // without a valid state we don't know (or can't trust) the client's redirect uri
            CallbackCheck::Rejected {
                state: None,
                rejection,
            } => {
                return Ok(AuthorizationCallbackResult {
                    location: get_signin_url(configuration, Some(rejection)),
                });
            }
        };

    let mut tx = db_pool.begin().await?;
    let identity_provider = db_get_login_provider_by_id(&mut tx, &decoded_internal_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    tx.commit().await?;

    // exchange code for the user's identity, checked against the approved institutions
    let callback_url = configuration.http_config.get_oauth_provider_callback_url();
    let location = match get_login_identity(
        configuration,
        &identity_provider,
        &decoded_internal_state,
        &callback_url,
        &code,
    )
    .await?
    {
        LoginIdentity::Approved(claims) => {
            generate_code_and_redirect(
                db_pool,
                configuration,
                &identity_provider,
                &decoded_internal_state,
                &claims,
            )
            .await?
        }
        LoginIdentity::Rejected(rejection) => {
            get_error_redirect(&decoded_internal_state, rejection)?
        }
    };
    Ok(AuthorizationCallbackResult {
        location: location.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::services::login_service::LoginRejection;
    use crate::services::oauth2_service::{error_redirect_url, get_error_redirect};
    use crate::utils::oauth2_authorization_code_utils::OAuth2State;
    use std::collections::HashMap;

    #[test]
    fn test_error_redirect() -> anyhow::Result<()> {
        let state = OAuth2State {
            tms_identity: String::default(),
            idp_id: "globus_idp".to_string(),
            client_id: "gateway".to_string(),
            exp: 0,
            redirect_uri: "https://gateway.example.org/callback?keep=1".to_string(),
            client_state: Some("client-state".to_string()),
            nonce: 0,
            institution_id: Some("utexas".to_string()),
            login_started: Some(0),
        };
        let url = get_error_redirect(&state, LoginRejection::UnapprovedInstitution)?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(url.path(), "/callback");
        assert_eq!(query["keep"], "1");
        assert_eq!(query["error"], "access_denied");
        assert_eq!(
            query["error_description"],
            "The identity is not from an approved institution"
        );
        assert_eq!(query["state"], "client-state");
        assert!(!query.contains_key("code"));

        // a failed login is the server's error, not the user's
        let url = get_error_redirect(&state, LoginRejection::LoginFailed)?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["error"], "server_error");
        assert_eq!(
            query["error_description"],
            "The login could not be completed"
        );

        // a login that can't start: built from the client's validated redirect uri and state
        let url = error_redirect_url(
            "https://gateway.example.org/callback",
            &Some("client-state".to_string()),
            LoginRejection::InvalidInstitution,
        )?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["error"], "invalid_request");
        assert_eq!(query["state"], "client-state");
        let url = error_redirect_url(
            "https://gateway.example.org/callback",
            &None,
            LoginRejection::LoginUnavailable,
        )?;
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["error"], "temporarily_unavailable");
        assert!(!query.contains_key("state"));
        Ok(())
    }
}
