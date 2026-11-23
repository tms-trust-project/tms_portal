import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type TClient = {
  id: number
  name: string
  description: string
  uri: string
}

const fetchClients = async () => {
  const { data } = await httpClient.get<{ result: TClient[] }>("/clients")
  return data?.result
}

export function useListClients() {
  return useQuery({ queryKey: ["clients"], queryFn: () => fetchClients() })
}
