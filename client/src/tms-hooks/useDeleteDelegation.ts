import { useMutation, useQueryClient } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

const deleteDelegation = async ({
  clientName,
  id,
}: {
  clientName: string
  id: number
}) => {
  await httpClient.delete(
    `/delegations/${encodeURIComponent(clientName)}/${id}`
  )
}

export const useDeleteDelegation = ({
  clientName,
  id,
}: {
  clientName: string
  id: number
}) => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => deleteDelegation({ clientName, id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["delegations"] })
    },
  })
}
