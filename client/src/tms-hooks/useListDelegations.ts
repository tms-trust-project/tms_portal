import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type Delegation = {
  id: number
  client_id: string
  client_name: string
  rp_account: string
  expires_at: string
  created: string
  updated: string
  tms_identity: string
  rp_id: string
}

const fetchDelegations = async () => {
  const { data } = await httpClient.get<{ result: Delegation[] }>(
    "/delegations"
  )
  return data?.result
}

export const useListDelegations = () => {
  return useQuery({
    queryKey: ["delegations"],
    queryFn: () => fetchDelegations(),
  })
}
