import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type ProviderLink = {
  id: number
  tms_identity: string
  rp_account: string
  rp_id: string
  resource_provider_name: string
  last_login: string
  enabled: boolean
}

const fetchProviderLinks = async () => {
  const { data } = await httpClient.get<{ result: ProviderLink[] }>(
    "/resources/providers/links"
  )
  return data?.result
}

export const useListProviderLinks = () => {
  return useQuery({
    queryKey: ["providerLinks"],
    queryFn: () => fetchProviderLinks(),
  })
}
