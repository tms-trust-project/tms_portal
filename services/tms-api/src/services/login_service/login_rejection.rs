/*
Why a login (the portal's own, or another client's OAuth2 login) didn't produce a TMS
token.  The portal's sign in page gets error_code() as ?error=; an OAuth2 client gets
oauth2_error() and description() on its redirect uri.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginRejection {
    // the login didn't name an institution
    InstitutionRequired,
    // the institution is unknown, disabled or has no Globus domain, or the identity that
    // came back isn't from the institution's domain
    UnapprovedInstitution,
    // the identity didn't authenticate for this login (an earlier session was reused)
    StaleAuthentication,
    // the user cancelled at the identity provider
    LoginCancelled,
    // the state is missing, doesn't match the state cookie, or has expired
    LoginExpired,
    // the identity provider sent an error, or the code couldn't be exchanged
    LoginFailed,
    // the login can't be started right now (e.g. the login identity provider is missing)
    LoginUnavailable,
}

impl LoginRejection {
    pub fn error_code(&self) -> &'static str {
        match self {
            LoginRejection::InstitutionRequired => "institution_required",
            LoginRejection::UnapprovedInstitution => "unapproved_institution",
            LoginRejection::StaleAuthentication => "stale_authentication",
            LoginRejection::LoginCancelled => "login_cancelled",
            LoginRejection::LoginExpired => "login_expired",
            LoginRejection::LoginFailed => "login_failed",
            LoginRejection::LoginUnavailable => "login_unavailable",
        }
    }

    // the RFC 6749 (4.1.2.1) error for an OAuth2 client's redirect uri
    pub fn oauth2_error(&self) -> &'static str {
        match self {
            LoginRejection::InstitutionRequired | LoginRejection::UnapprovedInstitution => {
                "invalid_request"
            }
            LoginRejection::LoginFailed => "server_error",
            LoginRejection::LoginUnavailable => "temporarily_unavailable",
            LoginRejection::StaleAuthentication
            | LoginRejection::LoginCancelled
            | LoginRejection::LoginExpired => "access_denied",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LoginRejection::InstitutionRequired => "An approved institution is required",
            LoginRejection::UnapprovedInstitution => {
                "The institution or the identity is not approved for login"
            }
            LoginRejection::StaleAuthentication => {
                "The user did not authenticate again for this login"
            }
            LoginRejection::LoginCancelled => "The user cancelled the login",
            LoginRejection::LoginExpired => "The login expired or was started again",
            LoginRejection::LoginFailed => "The login could not be completed",
            LoginRejection::LoginUnavailable => "Logging in is unavailable right now",
        }
    }
}
