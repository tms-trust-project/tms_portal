import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type Institution = {
  id: string
  name: string
  provider: "globus" | "tacc_tapis"
}

const fetchInstitutions = async () => {
  const { data } = await httpClient.get<{ result: Institution[] }>(
    "/login/institutions"
  )
  return data?.result
}

export const useListInstitutions = () => {
  return useQuery({
    queryKey: ["login", "institutions"],
    queryFn: () => fetchInstitutions(),
  })
}
