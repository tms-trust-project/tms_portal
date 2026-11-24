import { useQuery } from "@tanstack/react-query"
import { isAxiosError } from "axios"
import { httpClient } from "./httpClient"

export type GlobusConfiguration = {
  redirect_idp_domain: string
  idp_logout_url: string
}

export type Institution = {
  id: string
  name: string
  enabled: boolean
  federated_identity_config: { globus: GlobusConfiguration | null }
}

const fetchInstitutions = async () => {
  const { data } = await httpClient.get<{ result: Institution[] }>(
    "/institutions"
  )
  return data?.result
}

export const useListInstitutions = () => {
  return useQuery({
    queryKey: ["institutions"],
    queryFn: () => fetchInstitutions(),
    // the list rarely changes, and before login every fetch is a 401, so fetch it once
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    // a 4xx (e.g. 401 before login) won't change on retry
    retry: (failureCount, error) =>
      !(isAxiosError(error) && (error.response?.status ?? 500) < 500) &&
      failureCount < 3,
  })
}
