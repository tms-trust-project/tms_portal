/*
Another app's OAuth2 authorize request.  /oauth2/authorize sends one without an
institution to the sign in page, which picks the institution and sends it back.  The
authorize endpoint checks the client and redirect uri again, so nothing here is trusted.
 */
export function getAuthorizeRequest(
  search: string
): URLSearchParams | undefined {
  const params = new URLSearchParams(search)
  const clientId = params.get("client_id")
  const redirectUri = params.get("redirect_uri")
  if (!clientId || !redirectUri) return undefined

  const request = new URLSearchParams({
    response_type: "code",
    client_id: clientId,
    redirect_uri: redirectUri,
  })
  const state = params.get("state")
  if (state !== null) request.set("state", state)
  return request
}

// the authorize request again, now steered to the chosen institution
export function authorizeHref(request: URLSearchParams, institutionId: string) {
  const params = new URLSearchParams(request)
  params.set("institution", institutionId)
  return `/oauth2/authorize?${params}`
}
