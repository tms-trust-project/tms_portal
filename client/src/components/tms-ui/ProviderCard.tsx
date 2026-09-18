import {
  Card,
  CardHeader,
  CardTitle,
  //CardDescription,
  CardContent,
  CardAction,
  CardDescription,
} from "@/components/ui/card"

import { Building2, UserRound } from "lucide-react"
import { type ProviderLink } from "@/tms-hooks"
import { UnlinkButton } from "./UnlinkButton"

export function ProviderCard({
  provider,
  children,
}: React.PropsWithChildren<{ provider: ProviderLink; identity?: string }>) {
  return (
    <Card className="border-border/60 shadow-sm">
      <CardHeader>
        <CardTitle className="min-w-0 gap-2">
          <p className="flex items-center gap-1">
            <Building2 className="mr-1 inline" />
            <span>{provider.resource_provider_name}</span>
          </p>
          {/* <p>{provider.description}</p> */}
        </CardTitle>
        <CardDescription className="flex min-w-0 gap-2">
          <UserRound />
          <span className="break-all">
            {provider.resource_provider_account}
          </span>
        </CardDescription>

        <CardAction>
          <UnlinkButton providerLinkId={provider.id} />
        </CardAction>
      </CardHeader>

      <CardContent className="space-y-4">
        <div>
          <h2 className="text-base font-semibold">Available Resources</h2>
          <p className="text-sm text-muted-foreground">
            Link or unlink resources associated with this facility.
          </p>
        </div>

        {children}
      </CardContent>
    </Card>
  )
}
