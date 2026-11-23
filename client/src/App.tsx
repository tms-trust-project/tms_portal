import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert"

import { InfoIcon } from "lucide-react"

import { useListProviderLinks } from "./tms-hooks"
import { ProviderCard } from "./components/tms-ui/ProviderCard"
import { UserMenu } from "./components/tms-ui/UserMenu"
import { useAuth } from "./tms-hooks/useAuth"
import { LinkSuccessAlert } from "./components/tms-ui/LinkSuccessAlert"
import { DelegationsListing } from "./components/tms-ui/DelegationsListing"
import { LinkIdentityModal } from "./components/tms-ui/LinkIdentityModal"
import { LoginLanding } from "./components/tms-ui/LoginLanding"
//import { ProviderWizard } from "./components/tms-ui/ProviderWizard"

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
            <a href="/" className="hover:text-highlight">
              Trust Manager System
            </a>
          </h1>

          <UserMenu />
        </div>
      </header>

      <main className="mx-auto max-w-5xl space-y-2 px-4 py-6 sm:px-6 lg:px-8">
        {/* the server has no /signin route, so the home page is the sign in page */}
        {!isAuthenticated && <LoginLanding />}
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
