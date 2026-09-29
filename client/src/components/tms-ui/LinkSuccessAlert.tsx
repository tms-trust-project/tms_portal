import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"
import {
  Alert,
  AlertAction,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert"
import { CheckCircle2Icon, InfoIcon } from "lucide-react"
import { useState } from "react"
import { useDelegateProvider } from "@/tms-hooks/useDelegate"
import { useListDelegations, useListProviderLinks } from "@/tms-hooks"
import { Spinner } from "@/components/ui/spinner"
import { DelegationCard } from "./DelegationsListing"

type LinkState = {
  result?: string
  returnUri?: string
  clientName?: string
  providerName?: string
  providerId?: string
}

export function LinkSuccessAlert() {
  const stateParam = new URLSearchParams(window.location.search).get("state")
  const linkState = stateParam ? (JSON.parse(stateParam) as LinkState) : null
  const { clientName, providerId, providerName, returnUri } = linkState ?? {}
  const [isOpen, setIsOpen] = useState(linkState?.result === "success")

  const { data: delegations } = useListDelegations()
  const { data: linkedProviders } = useListProviderLinks()
  const mostRecentProvider = linkedProviders
    ?.filter((provider) => provider.rp_id === providerId)
    .sort(
      (left, right) =>
        Date.parse(right.last_login) - Date.parse(left.last_login)
    )[0]
  const existingDelegation = delegations?.find(
    (delegation) =>
      delegation.client_name === clientName && delegation.rp_id === providerId
  )

  const { mutate, isSuccess, isError, isPending } = useDelegateProvider({
    clientName: clientName ?? "",
    providerId: providerId ?? "",
    providerAccount: mostRecentProvider?.rp_account ?? "",
  })
  if (!mostRecentProvider || !returnUri || !clientName) return null

  const returnOrigin = new URL(returnUri).origin
  const confirmDelegation = () => mutate()

  return (
    <AlertDialog open={isOpen} onOpenChange={setIsOpen}>
      <AlertDialogContent className="data-[size=default]:sm:max-w-xl">
        <AlertDialogHeader>
          <AlertDialogTitle>Authentication Successful</AlertDialogTitle>
          <AlertDialogDescription>
            You can now delegate access to resources at{" "}
            {providerName ?? "this provider"} so that they are available to user{" "}
            {mostRecentProvider.rp_account} at {clientName} ({returnOrigin}).{" "}
            <strong>
              This action will permit the client to submit jobs and manage data
              on your behalf.
            </strong>
          </AlertDialogDescription>
        </AlertDialogHeader>
        {isSuccess && !!existingDelegation && (
          <Alert className="text-green-700">
            <CheckCircle2Icon />
            <AlertTitle>Delegation successful</AlertTitle>
            <AlertDescription>
              You have successfully delegated {providerName} access to the{" "}
              {clientName} client.
            </AlertDescription>
          </Alert>
        )}

        {existingDelegation && (
          <DelegationCard delegation={existingDelegation} />
        )}

        {!existingDelegation && (
          <Button onClick={confirmDelegation}>
            Confirm Delegation {isPending && <Spinner />}
          </Button>
        )}

        {isError && (
          <Alert className="text-red-700">
            <InfoIcon />
            <AlertTitle>Delegation Error</AlertTitle>
            <AlertDescription>
              An error occurred while attempting to delegate resource access.
            </AlertDescription>
            <AlertAction>
              <Button size="xs" variant="secondary" onClick={confirmDelegation}>
                Try again?
              </Button>
            </AlertAction>
          </Alert>
        )}

        <AlertDialogFooter>
          <AlertDialogCancel>Close Dialog</AlertDialogCancel>
          <Button
            nativeButton={false}
            render={<a href={returnUri}>Return to Science Gateway</a>}
          />
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
