use crate::obj_model::institution::{Institution, InstitutionsResponse};
use crate::utils::configuration::Configuration;
use anyhow::{Context, Result};
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tms_lib::utils::service_error::ServiceError::{BadRequest, Internal};

const INSTITUTIONS_ENDPOINT: &str = "v1/tms/institutions";
// the login flows the portal supports for institutions
pub const PROVIDER_GLOBUS: &str = "globus";
pub const PROVIDER_TACC_TAPIS: &str = "tacc_tapis";
// how long a list fetched from tms_server is reused before fetching it again
const CACHE_TTL: Duration = Duration::from_secs(60);

static INSTITUTIONS_CACHE: RwLock<Option<(Instant, Vec<Institution>)>> = RwLock::new(None);

/*
Get the approved institutions from tms_server, cached for CACHE_TTL.  Entries for
providers the portal can't log in through are skipped, as the tms_server spec asks
of clients.  Logging in requires an approved institution, so without a tms_server url
this is an error and nobody can log in.
 */
pub async fn get_institutions(configuration: &Configuration) -> Result<Vec<Institution>> {
    let http_config = &configuration.http_config;
    let Some(tms_server_url) = &http_config.tms_server_url else {
        return Err(Internal(
            "No tms_server url is configured (TMS_PORTAL_TMS_SERVER_URL), so there are no \
             approved institutions to log in through"
                .to_string(),
        )
        .into());
    };

    if let Some((fetched, institutions)) = INSTITUTIONS_CACHE
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        && fetched.elapsed() < CACHE_TTL
    {
        return Ok(institutions.clone());
    }

    let url = http_config.make_tms_url(tms_server_url, INSTITUTIONS_ENDPOINT);
    let response_text = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Error requesting institutions from {url}"))?
        .error_for_status()
        .context("Error response from tms_server institutions endpoint")?
        .text()
        .await
        .context("Error getting institutions response body")?;

    let response: InstitutionsResponse = serde_json::from_str(&response_text)
        .context("Error deserializing institutions response body")?;
    if response.result_code != "0" {
        return Err(Internal(format!(
            "tms_server institutions error: {}",
            response.result_msg
        ))
        .into());
    }

    let institutions: Vec<Institution> = response
        .institutions
        .into_iter()
        .filter(|institution| {
            institution.provider == PROVIDER_GLOBUS || institution.provider == PROVIDER_TACC_TAPIS
        })
        .collect();
    *INSTITUTIONS_CACHE
        .write()
        .unwrap_or_else(|error| error.into_inner()) = Some((Instant::now(), institutions.clone()));
    Ok(institutions)
}

/*
Look up an institution by id.  The redirect domain always comes from tms_server,
never from the browser, so only approved institutions can be used.
 */
pub async fn get_institution_by_id(
    configuration: &Configuration,
    institution_id: &str,
) -> Result<Institution> {
    get_institutions(configuration)
        .await?
        .into_iter()
        .find(|institution| institution.id == institution_id)
        .ok_or_else(|| {
            BadRequest(format!("{institution_id} is not an approved institution")).into()
        })
}

/*
Whether a Globus identity (user@domain) belongs to one of the approved Globus
institutions.  redirect_idp_domain only steers the login, it does not restrict it,
so a user already signed in to Globus can come back with any identity.  Without a
tms_server url there is no list to check against, and this is an error.
 */
pub async fn is_approved_globus_identity(
    configuration: &Configuration,
    username: &str,
) -> Result<bool> {
    let Some((_, domain)) = username.rsplit_once('@') else {
        return Ok(false);
    };
    let domain = domain.to_ascii_lowercase();
    Ok(get_institutions(configuration)
        .await?
        .iter()
        .any(|institution| {
            institution.provider == PROVIDER_GLOBUS
                && institution.redirect_idp_domain.as_deref() == Some(domain.as_str())
        }))
}
