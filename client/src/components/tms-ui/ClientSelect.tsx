import {
  Alert,
  AlertAction,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

import { Button } from "@/components/ui/button"
import { useListClients } from "@/tms-hooks/useListClients"
import { useClientState } from "@/tms-hooks"

export function ClientSelect() {
  const { clientName } = useClientState()
  const { data: clientListing } = useListClients()

  return (
    <Alert className="mb-2">
      <AlertTitle>Welcome!</AlertTitle>
      <AlertDescription>
        {clientName ? (
          <span>You are currently using client {clientName}.</span>
        ) : (
          <span>No client gateway is selected.</span>
        )}
      </AlertDescription>
      <AlertAction>
        {!!clientListing && (
          <DropdownMenu>
            <DropdownMenuTrigger
              render={<Button variant="outline">Select Client</Button>}
            />
            <DropdownMenuContent className="w-fit">
              <DropdownMenuGroup>
                <DropdownMenuLabel>Available Clients</DropdownMenuLabel>
                {clientListing.map((client) => (
                  <DropdownMenuItem
                    key={client.name}
                    render={
                      <a
                        href={`/?client_return_uri=${encodeURIComponent(client.uri)}&client_name=${encodeURIComponent(client.name)}`}
                      >
                        {client.name} ({client.uri})
                      </a>
                    }
                  ></DropdownMenuItem>
                ))}
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </AlertAction>
    </Alert>
  )
}
