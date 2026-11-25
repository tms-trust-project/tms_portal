/*
Another app's OAuth2 authorize request.  /oauth2/authorize sends one without an
institution to the portal's sign in page (the home page), carrying response_type,
client_id, redirect_uri, state and scope.  The page picks the institution and sends the
same request back with it.  The authorize endpoint checks the client and redirect uri
again, so nothing here is trusted.
 */
export function getAuthorizeRequest(
  search: string
): URLSearchParams | undefined {
  const params = new URLSearchParams(search)
  const clientId = params.get("client_id")
  const redirectUri = params.get("redirect_uri")
  if (params.get("response_type") !== "code" || !clientId || !redirectUri)
    return undefined

  const request = new URLSearchParams({
    response_type: "code",
    client_id: clientId,
    redirect_uri: redirectUri,
  })
  for (const name of ["state", "scope"]) {
    const value = params.get(name)
    if (value !== null) request.set(name, value)
  }
  return request
}

// the authorize request again, now steered to the chosen institution
export function authorizeHref(request: URLSearchParams, institutionId: string) {
  const params = new URLSearchParams(request)
  params.set("institution", institutionId)
  return `/oauth2/authorize?${params}`
}
