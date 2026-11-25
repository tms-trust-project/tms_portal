/*
The sign in page's ?error=<code>, which the server sets when it refuses or can't complete
a login (LoginRejection::error_code in
services/tms-api/src/services/login_service/login_rejection.rs).
 */
const LOGIN_ERRORS: Record<string, string> = {
  // /login without an institution
  institution_required: "Choose your institution to log in.",
  // the institution is unknown or disabled, or the identity isn't from its domain
  unapproved_institution:
    "That login isn't from an approved institution. Choose your institution and log in with your identity from it.",
  // Globus reused an earlier session instead of authenticating the user again
  stale_authentication:
    "Your login could not be confirmed as new. Please log in again.",
  // the user cancelled at Globus or at the institution's login page
  login_cancelled:
    "The login was cancelled. Choose your institution to try again.",
  // the login took more than 5 minutes, or another login was started since
  login_expired:
    "Your login took too long or was started again in another tab. Please log in again.",
  // the identity provider sent back an error, or the login code didn't work
  login_failed: "The login could not be completed. Please try again.",
  // the login provider isn't available
  login_unavailable:
    "Logging in is unavailable right now. Please try again later.",
}

// the message for the query string's error, if it has one (an unknown code gets the
// general one)
export function loginErrorMessage(search: string): string | undefined {
  const code = new URLSearchParams(search).get("error")
  if (!code) return undefined
  return LOGIN_ERRORS[code] ?? LOGIN_ERRORS.login_failed
}

// the query string without its error, so a reload doesn't show it again ("" when nothing
// else is left)
export function withoutLoginError(search: string): string {
  const params = new URLSearchParams(search)
  params.delete("error")
  const query = params.toString()
  return query ? `?${query}` : ""
}
