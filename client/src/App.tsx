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

import {
  useListProviderLinks,
  useListProviders,
  useListResources,
} from "./tms-hooks"
import { ResourceCard } from "./components/tms-ui/ResourceCard"
import { ProviderCard } from "./components/tms-ui/ProviderCard"
import { UserMenu } from "./components/tms-ui/UserMenu"
import { useAuth } from "./tms-hooks/useAuth"
import { Separator } from "./components/ui/separator"
import { LinkSuccessAlert } from "./components/tms-ui/LinkSuccessAlert"
import { UnlinkButton } from "./components/tms-ui/UnlinkButton"
//import { ProviderWizard } from "./components/tms-ui/ProviderWizard"

function ResourceCardGrid({
  userId,
  providerId,
}: {
  userId: string
  providerId: string
}) {
  const { data: ResourceList } = useListResources({ providerId, userId })
  if (!ResourceList) return null
  return (
    <>
      <div className="grid gap-2 sm:grid-cols-[repeat(auto-fit,minmax(400px,1fr))]">
        {ResourceList.map((resource) => (
          <ResourceCard resource={resource} key={resource.id} />
        ))}
      </div>
    </>
  )
}

function LinkIdentityModal() {
  const { data: providerList } = useListProviders()
  const { data: providerLinkList } = useListProviderLinks()
  const returnUri = new URLSearchParams(window.location.search).get(
    "client_return_uri"
  )
  const clientName = new URLSearchParams(window.location.search).get(
    "client_name"
  )

  function constructLinkState(providerId: string) {
    return encodeURIComponent(
      JSON.stringify({
        result: "success",
        providerId,
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
      <DialogTrigger asChild>
        <Button size="sm" variant="outline">
          <Plus /> Add Provider
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Link a New Provider Identity</DialogTitle>
        </DialogHeader>
        <Separator />
        {providerList?.map((provider) => {
          return (
            <div
              key={provider.id}
              className="flex items-center justify-between gap-4"
            >
              <span>
                {provider.name} ({provider.id})
              </span>
              {providerLinkList?.some(
                (p) => p.resource_provider_id == provider.id
              ) ? (
                <div className="inline-flex gap-1">
                  <Button variant="outline" asChild>
                    <a
                      href={`/resources/providers/authorize?provider_id=${provider.id}&redirect_url=${window.location.origin}&state=${constructLinkState(provider.id)}`}
                    >
                      <RefreshCcw className="mr-1 size-4" />
                      Refresh
                    </a>
                  </Button>
                  <UnlinkButton
                    providerLinkId={
                      providerLinkList.find(
                        (p) => p.resource_provider_id === provider.id
                      )?.id ?? -1
                    }
                  />
                </div>
              ) : (
                <Button size="sm" asChild>
                  <a
                    href={`/resources/providers/authorize?provider_id=${provider.id}&redirect_url=${window.location.origin}&state=${constructLinkState(provider.id)}`}
                  >
                    Connect
                  </a>
                </Button>
              )}
            </div>
          )
        })}
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
          We have no linked providers on record for your account. Please click
          "Add Provider" above to begin the account linking process.
        </AlertDescription>
      </Alert>
    )

  return providerList.map((provider) => (
    <ProviderCard key={`${provider.id}`} provider={provider}>
      <ResourceCardGrid
        userId={"ok"}
        providerId={provider.resource_provider_id}
      />
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
