/*
The portal's own login link.  Each value is URL encoded, so a redirect uri with its own
query string can't spill into the login's parameters.  The server ignores institution
until /login supports it, which is harmless.
 */
export function loginHref(institutionId?: string) {
  const params = new URLSearchParams({
    idp_id: "globus_idp",
    redirect_uri: __REDIRECT_URI__,
  })
  if (institutionId) params.set("institution", institutionId)
  return `/login?${params}`
}
