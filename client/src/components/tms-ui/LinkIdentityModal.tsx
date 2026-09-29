import { useState } from "react"
import { Plus, RefreshCcw } from "lucide-react"
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

import { useListProviderLinks, useListProviders } from "@/tms-hooks"

import { Separator } from "@/components/ui/separator"
import { UnlinkButton } from "@/components/tms-ui/UnlinkButton"

type ProviderRow = {
  id: string
  name: string
  providerLinkId?: number
}

export function LinkIdentityModal() {
  const { data: providerList } = useListProviders()
  const { data: providerLinkList } = useListProviderLinks()
  const searchParams = new URLSearchParams(window.location.search)
  const returnUri = searchParams.get("client_return_uri")
  const clientName = searchParams.get("client_name")
  const linkStateRawParam = searchParams.get("state")
  const linkStateParams = linkStateRawParam
    ? JSON.parse(linkStateRawParam)
    : undefined

  const linkedProviderIds = new Set(
    (providerLinkList ?? []).map((link) => link.rp_id)
  )
  const providers: ProviderRow[] = [
    ...(providerLinkList ?? []).map((link) => ({
      id: link.rp_id,
      name: link.resource_provider_name,
      providerLinkId: link.id,
    })),
    ...(providerList ?? [])
      .filter((provider) => !linkedProviderIds.has(provider.id))
      .map((provider) => ({ id: provider.id, name: provider.name })),
  ]

  function authorizationHref(provider: ProviderRow) {
    const state = encodeURIComponent(
      JSON.stringify({
        result: returnUri ? "success" : undefined,
        providerId: provider.id,
        providerName: provider.name,
        returnUri,
        clientName,
      })
    )
    return (
      `/resources/providers/authorize?provider_id=${provider.id}` +
      `&redirect_url=${window.location.origin}` +
      `&state=${state}`
    )
  }

  const [openState, setOpenState] = useState(
    !!returnUri && linkStateParams?.result !== "success"
  )

  return (
    <Dialog open={openState} onOpenChange={setOpenState}>
      {clientName && returnUri && (
        <DialogTrigger
          render={
            <Button size="sm" variant="outline">
              <Plus /> Add Provider
            </Button>
          }
        />
      )}
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Link a New Provider Identity</DialogTitle>
        </DialogHeader>
        <Separator />
        {providers.map((provider) => {
          const isLinked = provider.providerLinkId !== undefined
          return (
            <div
              key={provider.providerLinkId ?? provider.id}
              className="flex items-center justify-between gap-4"
            >
              <span>
                {provider.name} ({provider.id})
              </span>
              <div className="inline-flex gap-1">
                <Button
                  nativeButton={false}
                  variant={isLinked ? "outline" : undefined}
                  render={
                    <a href={authorizationHref(provider)}>
                      {isLinked && <RefreshCcw className="mr-1 size-4" />}
                      {isLinked ? "Refresh" : "Connect"}
                    </a>
                  }
                />
                {provider.providerLinkId !== undefined && (
                  <UnlinkButton providerLinkId={provider.providerLinkId} />
                )}
              </div>
            </div>
          )
        })}

        <DialogFooter>
          <DialogClose render={<Button variant="outline">Cancel</Button>} />
          {returnUri && (
            <Button
              nativeButton={false}
              render={<a href={returnUri}>Return to Science Gateway</a>}
            />
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
