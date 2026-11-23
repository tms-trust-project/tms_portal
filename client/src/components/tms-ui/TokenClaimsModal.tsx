import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { decodeJwtClaims } from "@/lib/jwt"

// the claims in the logged in user's TMS token, as JSON
export function TokenClaimsModal({
  token,
  open,
  onOpenChange,
}: {
  token: string
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const claims = decodeJwtClaims(token)
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle className="text-highlight">Token claims</DialogTitle>
          <DialogDescription>
            Decoded from your TMS token. The portal verifies the token, not this
            page.
          </DialogDescription>
        </DialogHeader>
        {claims ? (
          <pre className="max-h-[60vh] overflow-auto rounded-md border bg-muted p-3 text-xs">
            {JSON.stringify(claims, null, 2)}
          </pre>
        ) : (
          <p className="text-sm text-destructive">
            The token could not be decoded.
          </p>
        )}
      </DialogContent>
    </Dialog>
  )
}
