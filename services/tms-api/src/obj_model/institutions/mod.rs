#[cfg(test)]
pub mod test_fixtures;

use serde::Deserialize;
use tms_lib::utils::service_error::ServiceError::{self, BadRequest};

#[derive(Debug, Clone, Deserialize)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub idp_config: IdpConfig,
}

impl Institution {
    /*
    The domain a login through this institution is sent to (redirect_idp_domain): Globus
    takes a user who isn't signed in yet straight to the institution's own login page, and
    the login callback only accepts an identity from the domain (identity_is_from_domain).
    An error, a bad request, when the institution can't be used to log in.
     */
    pub fn globus_login_domain(&self) -> Result<&str, ServiceError> {
        if !self.enabled {
            return Err(BadRequest(format!("Institution '{}' is not enabled for login", self.id)));
        }
        match &self.idp_config.globus {
            Some(globus) if !globus.redirect_idp_domain.trim().is_empty() => {
                Ok(&globus.redirect_idp_domain)
            }
            _ => Err(BadRequest(format!("Institution '{}' has no Globus login domain", self.id))),
        }
    }
}

/*
True when a login identity's username (e.g. jdoe@utexas.edu) is from the domain: the
part after the last @ is the domain, ignoring case.  A subdomain doesn't count, so
jdoe@cs.utexas.edu isn't from utexas.edu.  The same rule as the Globus redirect proof
of concept (idp_redirect_poc's identity_matches).
 */
pub fn identity_is_from_domain(username: &str, domain: &str) -> bool {
    username
        .rsplit_once('@')
        .is_some_and(|(_, username_domain)| username_domain.eq_ignore_ascii_case(domain.trim()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdpConfig {
    pub globus: Option<GlobusConfiguration>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobusConfiguration {
    pub redirect_idp_domain: String,
    pub idp_logout_url: String,
}

#[cfg(test)]
#[path = "institutions.test.rs"]
mod institutions_test;
