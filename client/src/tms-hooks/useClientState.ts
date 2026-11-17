type LinkState = {
  result?: string
  returnUri?: string
  clientName?: string
  providerName?: string
  providerId?: string
}

export function useClientState() {
  const searchParams = new URLSearchParams(window.location.search)
  const stateParam = searchParams.get("state")
  const stateParamJSON = stateParam
    ? (JSON.parse(stateParam) as LinkState)
    : null

  const returnUri: string | undefined =
    searchParams.get("client_return_uri") ?? stateParamJSON?.returnUri
  const clientName: string | undefined =
    searchParams.get("client_name") ?? stateParamJSON?.clientName
  const hasLinkSuccess = stateParamJSON?.result === "success"

  const providerId: string | undefined = stateParamJSON?.providerId
  const providerName: string | undefined = stateParamJSON?.providerName

  return { returnUri, clientName, providerId, providerName, hasLinkSuccess }
}
