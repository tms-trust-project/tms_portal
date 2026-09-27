import { useMutation, useQueryClient } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

const delegateProvider = async ({
  clientName,
  providerId,
  providerAccount,
}: {
  clientName: string
  providerId: string
  providerAccount: string
}) => {
  await httpClient.post(`/delegations`, {
    client_name: clientName,
    resource_provider_id: providerId,
    resource_provider_account: providerAccount,
  })
}

export const useDelegateProvider = ({
  clientName,
  providerId,
  providerAccount,
}: {
  clientName: string
  providerId: string
  providerAccount: string
}) => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () =>
      delegateProvider({ clientName, providerAccount, providerId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["delegations"] })
    },
  })
}
