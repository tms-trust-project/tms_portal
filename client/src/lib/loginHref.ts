/*
The portal's own login link.  Each value is URL encoded, so a redirect uri with its own
query string can't spill into the login's parameters.  /login requires an institution:
Globus sends the user straight to the institution's login page (redirect_idp_domain) and
requires an identity from its domain (session_required_single_domain), and the server checks
the identity again when Globus redirects back.  A login without an institution, or with one
that's unknown, disabled or has no Globus domain, comes back to the sign in page with
?error=<code> instead.
 */
export function loginHref(institutionId?: string) {
  const params = new URLSearchParams({
    idp_id: "globus_idp",
    redirect_uri: __REDIRECT_URI__,
  })
  if (institutionId) params.set("institution", institutionId)
  return `/login?${params}`
}
