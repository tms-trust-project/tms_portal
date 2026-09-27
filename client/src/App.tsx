import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert"

import { useState } from "react"
import { InfoIcon, Plus, RefreshCcw } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

import { useListProviderLinks, useListProviders } from "./tms-hooks"
import { ProviderCard } from "./components/tms-ui/ProviderCard"
import { UserMenu } from "./components/tms-ui/UserMenu"
import { useAuth } from "./tms-hooks/useAuth"
import { Separator } from "./components/ui/separator"
import { LinkSuccessAlert } from "./components/tms-ui/LinkSuccessAlert"
import { UnlinkButton } from "./components/tms-ui/UnlinkButton"
import { DelegationsListing } from "./components/tms-ui/DelegationsListing"
//import { ProviderWizard } from "./components/tms-ui/ProviderWizard"

function LinkIdentityModal() {
  const { data: providerList } = useListProviders()
  const { data: providerLinkList } = useListProviderLinks()
  const returnUri = new URLSearchParams(window.location.search).get(
    "client_return_uri"
  )
  const clientName = new URLSearchParams(window.location.search).get(
    "client_name"
  )

  function constructLinkState(provider: { id: string; name: string }) {
    return encodeURIComponent(
      JSON.stringify({
        result: returnUri ? "success" : undefined,
        providerId: provider.id,
        providerName: provider.name,
        returnUri,
        clientName,
      })
    )
  }

  const linkStateRawParam = new URLSearchParams(window.location.search).get(
    "state"
  )
  const linkStateParams = linkStateRawParam
    ? JSON.parse(linkStateRawParam)
    : undefined

  const [openState, setOpenState] = useState(
    !!returnUri && linkStateParams?.result !== "success"
  )

  return (
    <Dialog open={openState} onOpenChange={setOpenState}>
      {clientName && returnUri && (
        <DialogTrigger asChild>
          <Button size="sm" variant="outline">
            <Plus /> Add Provider
          </Button>
        </DialogTrigger>
      )}
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Link a New Provider Identity</DialogTitle>
        </DialogHeader>
        <Separator />
        {providerLinkList?.map((providerLink) => (
          <div
            key={providerLink.id}
            className="flex items-center justify-between gap-4"
          >
            <span>
              {providerLink.resource_provider_name} ({providerLink.rp_id})
            </span>

            <div className="inline-flex gap-1">
              <Button variant="outline" asChild>
                <a
                  href={
                    `/resources/providers/authorize?provider_id=${providerLink.rp_id}` +
                    `&redirect_url=${window.location.origin}` +
                    `&state=${constructLinkState({ id: providerLink.rp_id, name: providerLink.resource_provider_name })}`
                  }
                >
                  <RefreshCcw className="mr-1 size-4" />
                  Refresh
                </a>
              </Button>
              <UnlinkButton providerLinkId={providerLink.id} />
            </div>
          </div>
        ))}
        {providerList
          ?.filter(
            (p) => !providerLinkList?.some((link) => link.rp_id === p.id) // Iterate over providers that haven't been linked.
          )
          .map((provider) => (
            <div
              key={provider.id}
              className="flex items-center justify-between gap-4"
            >
              <span>
                {provider.name} ({provider.id})
              </span>

              <div className="inline-flex gap-1">
                <Button asChild>
                  <a
                    href={
                      `/resources/providers/authorize?provider_id=${provider.id}` +
                      `&redirect_url=${window.location.origin}` +
                      `&state=${constructLinkState(provider)}`
                    }
                  >
                    Connect
                  </a>
                </Button>
              </div>
            </div>
          ))}

        <Separator />
        {returnUri && (
          <Button asChild>
            <a href={returnUri}>Return to Science Gateway</a>
          </Button>
        )}
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function ProviderCardList() {
  const { data: providerList } = useListProviderLinks()

  if (!providerList) return null

  if (!providerList.length)
    return (
      <Alert>
        <InfoIcon />
        <AlertTitle>No Linked Providers</AlertTitle>
        <AlertDescription>
          We have no linked providers on record for your account.
        </AlertDescription>
      </Alert>
    )

  return providerList.map((provider) => (
    <ProviderCard key={`${provider.id}`} provider={provider}>
      <DelegationsListing />
      {/* <ResourceCardSelector providerId={provider.resource_provider_id} /> */}
    </ProviderCard>
  ))
}

function App() {
  const { data: isAuthenticated } = useAuth()
  return (
    <div className="min-h-screen bg-muted/30">
      <header className="border-b bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60">
        <div className="mx-auto flex max-w-5xl flex-row items-center justify-between gap-4 px-4 py-4 sm:px-6 lg:px-8">
          <h1 className="text-2xl font-semibold tracking-tight">
            Trust Manager System
          </h1>

          <UserMenu />
        </div>
      </header>

      <main className="mx-auto max-w-5xl space-y-2 px-4 py-6 sm:px-6 lg:px-8">
        {!isAuthenticated && (
          <div>
            Please{" "}
            <a
              href="/login?idp_id=globus_idp&redirect_uri=https://tms-portal.savanna.tacc.cloud/"
              className="text-primary underline-offset-4 hover:underline"
            >
              log in
            </a>{" "}
            to manage resources.
          </div>
        )}
        {!!isAuthenticated && (
          <>
            <h2 className="flex items-center gap-2 text-lg font-semibold">
              Linked Providers <LinkIdentityModal />
            </h2>
            <ProviderCardList></ProviderCardList>
            {/* <ProviderWizard /> */}
          </>
        )}
      </main>
      <LinkSuccessAlert />
    </div>
  )
}

export default App
