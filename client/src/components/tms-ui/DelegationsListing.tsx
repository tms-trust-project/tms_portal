import React from "react"
import {
  useDeleteDelegation,
  useListDelegations,
  type Delegation,
} from "@/tms-hooks"
import {
  Card,
  CardHeader,
  CardTitle,
  //CardDescription,
  CardContent,
  CardAction,
} from "@/components/ui/card"

import { Badge } from "@/components/ui/badge"
import { Alert, AlertDescription, AlertTitle } from "../ui/alert"
import { InfoIcon, Unplug } from "lucide-react"
import { Button } from "../ui/button"

export function DelegationCard({ delegation }: { delegation: Delegation }) {
  const expiration = new Date(delegation.expires_at)
  const hasValidExpiration = !Number.isNaN(expiration.getTime())
  const [currentDate] = React.useState(() => Date.now())
  const expired = hasValidExpiration && expiration.getTime() < currentDate
  const { mutate: deleteDelegation } = useDeleteDelegation({
    clientName: delegation.client_name,
    id: delegation.id,
  })

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-start justify-between gap-4">
        <div className="flex min-w-0 items-center gap-2">
          <CardTitle className="truncate">{delegation.client_name}</CardTitle>
          <Badge variant={expired ? "destructive" : "default"}>
            {hasValidExpiration ? (expired ? "Expired" : "Active") : "Unknown"}
          </Badge>
        </div>

        <CardAction className="flex items-center gap-2">
          <Button
            variant="destructive"
            size="sm"
            onClick={() => deleteDelegation()}
          >
            <Unplug className="mr-2 size-4" />
            Revoke
          </Button>
        </CardAction>
      </CardHeader>

      <CardContent>
        {/* <div className="flex flex-col gap-3 text-sm sm:flex-row sm:gap-6"> */}
        <div className="flex flex-wrap gap-6">
          {[
            ["Created", delegation.created],
            ["Updated", delegation.updated],
            ["Expires", delegation.expires_at],
          ].map(([label, value]) => (
            <div key={label} className="shrink-0">
              <p className="text-xs font-medium text-muted-foreground">
                {label}
              </p>
              <time dateTime={value} className="mt-1 block text-nowrap">
                {formatDate(value)}
              </time>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

function formatDate(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(date)
}

export function DelegationsListing() {
  const { data: delegations } = useListDelegations()

  if (!delegations) return null
  if (delegations.length === 0)
    return (
      <Alert>
        <InfoIcon />
        <AlertTitle>No current delegations</AlertTitle>
        <AlertDescription>
          According to our records, you have not delegated access to resources
          from this provider.
        </AlertDescription>
      </Alert>
    )
  return (
    <ul>
      {delegations?.map((d) => (
        <DelegationCard key={d.id} delegation={d} />
      ))}
    </ul>
  )
}
