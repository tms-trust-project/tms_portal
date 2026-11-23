import { useQuery, useQueryClient } from "@tanstack/react-query"
import { isAxiosError } from "axios"
import Cookies from "js-cookie"
import { httpClient } from "./httpClient"

export type Whoami = {
  name: string
  user_name: string
  idpDisplayName: string
  organization: string
  // sign-out page to open in a new tab (Tapis), since it can't redirect back
  logoutUrl?: string
}

const fetchWhoami = async () => {
  const { data } = await httpClient.get<{ result: Whoami }>("/login/whoami")
  return data?.result
}

export const useWhoami = ({ enabled = true }: { enabled?: boolean } = {}) => {
  const queryClient = useQueryClient()
  return useQuery({
    queryKey: ["auth", "whoami"],
    queryFn: async () => {
      try {
        return await fetchWhoami()
      } catch (error) {
        // The token cookie holds a token the portal no longer accepts (revoked, unknown after
        // a database reset, or signed with an old key).  Drop it, so the user is shown as
        // logged out and can log in again instead of being stuck until it expires.
        if (isAxiosError(error) && error.response?.status === 401) {
          Cookies.remove("tmstoken", { path: "/" })
          queryClient.setQueryData(["auth-cookie"], null)
          return null
        }
        throw error
      }
    },
    enabled,
  })
}
