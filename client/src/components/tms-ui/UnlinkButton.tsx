import { Button } from "@/components/ui/button"

import { Unplug } from "lucide-react"
import { useUnlinkProvider } from "@/tms-hooks"

export function UnlinkButton({
  providerLinkId,
}: React.PropsWithChildren<{ providerLinkId: number }>) {
  const { mutate } = useUnlinkProvider({ id: providerLinkId })
  return (
    <Button variant="destructive" onClick={() => mutate()}>
      <Unplug className="mr-2 size-4" />
      Disconnect
    </Button>
  )
}
