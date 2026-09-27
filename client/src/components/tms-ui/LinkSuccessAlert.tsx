import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { useState } from "react"
import { Button } from "@/components/ui/button"
import {
  Alert,
  AlertAction,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert"
import { CheckCircle2Icon, InfoIcon } from "lucide-react"
import { useDelegateProvider } from "@/tms-hooks/useDelegate"
import { Spinner } from "../ui/spinner"
import { useListDelegations, useListProviderLinks } from "@/tms-hooks"
import { DelegationCard } from "./DelegationsListing"

export function LinkSuccessAlert() {
  const linkStateRawParam = new URLSearchParams(window.location.search).get(
    "state"
  )

  const linkStateParams = linkStateRawParam
    ? JSON.parse(linkStateRawParam)
    : undefined
  const returnUrl = linkStateParams?.returnUri
  const clientName = linkStateParams?.clientName
  const isSuccessState = linkStateParams?.result === "success"
  const providerName = linkStateParams?.providerName
  const providerId = linkStateParams?.providerId
  const [isOpen, setIsOpen] = useState(isSuccessState)

  const { data: delegations } = useListDelegations()
  const { data: linkedProviders } = useListProviderLinks()
  const mostRecentProvider = (linkedProviders ?? [])
    .filter((p) => p.rp_id === providerId)
    .sort((p1, p2) =>
      new Date(p1.last_login) > new Date(p2.last_login) ? 1 : -1
    )[0]

  const existingDelegation = (delegations ?? []).filter(
    (d) => d.client_name === clientName && d.rp_id === providerId
  )

  const { mutate, isSuccess, isError, isIdle, isPending } = useDelegateProvider(
    {
      clientName,
      providerId,
      providerAccount: mostRecentProvider?.rp_account ?? "",
    }
  )
  if (!mostRecentProvider) return null

  return (
    <AlertDialog open={isOpen} onOpenChange={setIsOpen}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Authentication Successful</AlertDialogTitle>
          <AlertDialogDescription>
            You can now delegate access to resources at{" "}
            {providerName ?? "this provider"} so that they are available to user{" "}
            {mostRecentProvider.rp_account} at {clientName} (
            {new URL(returnUrl).origin}).{" "}
            <strong>
              This action will permit the client to submit jobs and manage data
              on your behalf.
            </strong>
          </AlertDialogDescription>
        </AlertDialogHeader>

        {existingDelegation.map((d) => (
          <DelegationCard delegation={d} key={d.id} />
        ))}

        {(isIdle || isPending) && !(existingDelegation.length > 0) && (
          <Button onClick={() => mutate()}>
            Confirm Delegation {isPending && <Spinner />}
          </Button>
        )}
        {isSuccess && (
          <Alert className="text-green-700">
            <CheckCircle2Icon />
            <AlertTitle>Delegation successful</AlertTitle>
            <AlertDescription>
              You have successfully delegated {providerName} access to the{" "}
              {clientName} client.
            </AlertDescription>
          </Alert>
        )}
        {isError && (
          <Alert className="text-red-700">
            <InfoIcon />
            <AlertTitle>Delegation Error</AlertTitle>
            <AlertDescription>
              An error occurred while attempting to delegate resource access.
            </AlertDescription>
            <AlertAction>
              <Button size="xs" variant="secondary" onClick={() => mutate()}>
                Try again?
              </Button>
            </AlertAction>
          </Alert>
        )}

        <AlertDialogFooter>
          <AlertDialogCancel>Close Dialog</AlertDialogCancel>
          <Button asChild>
            <a href={returnUrl}>Return to Science Gateway</a>
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
