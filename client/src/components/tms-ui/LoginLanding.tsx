import { useState } from "react"
import { InfoIcon, LogIn, X } from "lucide-react"
import { Alert, AlertAction, AlertDescription } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { useListInstitutions } from "@/tms-hooks"

// set by the login callback when it rejects a login (LoginRejection::error_code)
const LOGIN_ERRORS: Record<string, string> = {
  // the Globus identity isn't from an approved institution
  unapproved_institution:
    "Please login with one of the approved institutions in the list",
  // Globus reused an earlier session instead of authenticating the user again
  stale_authentication:
    "Your login could not be confirmed as new. Please log in again.",
  // the user cancelled the login at Globus or Tapis
  login_cancelled:
    "The login was cancelled. Choose your institution to try again.",
  // the identity provider sent back an error, or the login code didn't work
  login_failed: "The login could not be completed. Please try again.",
  // the login took more than 5 minutes, or another login was started since
  login_expired:
    "Your login took too long or was started again in another tab. Please log in again.",
  // the institution list (tms_server) or the login provider is unavailable
  login_unavailable:
    "Logging in is unavailable right now. Please try again later.",
}

function loginHref(institutionId: string) {
  return `/login?idp_id=globus_idp&redirect_uri=${__REDIRECT_URI__}&institution=${encodeURIComponent(institutionId)}`
}

function LoginErrorAlert() {
  const searchParams = new URLSearchParams(window.location.search)
  const [message, setMessage] = useState<string | undefined>(
    LOGIN_ERRORS[searchParams.get("error") ?? ""]
  )
  if (!message) return null

  const dismiss = () => {
    setMessage(undefined)
    // drop the error so a reload doesn't show it again
    window.history.replaceState(null, "", window.location.pathname)
  }
  return (
    <Alert variant="destructive" className="mx-auto mb-4 max-w-md">
      <InfoIcon />
      <AlertDescription>{message}</AlertDescription>
      <AlertAction>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Dismiss"
          onClick={dismiss}
        >
          <X />
        </Button>
      </AlertAction>
    </Alert>
  )
}

export function LoginLanding() {
  const { data: institutions, isPending, isError } = useListInstitutions()
  const [selected, setSelected] = useState("")
  const globusInstitutions = (institutions ?? []).filter(
    (institution) => institution.provider === "globus"
  )
  const tapis = institutions?.find(
    (institution) => institution.provider === "tacc_tapis"
  )
  const institutionId = selected || globusInstitutions[0]?.id

  return (
    <>
      <LoginErrorAlert />
      <Card className="mx-auto max-w-md">
        <CardHeader>
          <CardTitle>Log in to TMS</CardTitle>
          <CardDescription>
            Choose your institution to go straight to its login page.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {isPending && <p className="text-sm">Loading institutions…</p>}
          {isError && (
            <p className="text-sm text-destructive">
              The list of institutions could not be loaded.
            </p>
          )}
          {!!globusInstitutions.length && (
            <>
              <Label htmlFor="institution">Institution</Label>
              <select
                id="institution"
                className="w-full rounded-md border bg-background px-3 py-2 text-sm"
                value={institutionId}
                onChange={(event) => setSelected(event.target.value)}
              >
                {globusInstitutions.map((institution) => (
                  <option key={institution.id} value={institution.id}>
                    {institution.name}
                  </option>
                ))}
              </select>
            </>
          )}
        </CardContent>
        <CardFooter className="flex flex-col items-start gap-3">
          {institutionId && (
            <Button
              role="link"
              nativeButton={false}
              render={
                <a href={loginHref(institutionId)}>
                  <LogIn /> Log In
                </a>
              }
            />
          )}
          {tapis && (
            <a
              href={loginHref(tapis.id)}
              className="text-sm text-primary underline-offset-4 hover:underline"
            >
              Log in with Tapis
            </a>
          )}
        </CardFooter>
      </Card>
    </>
  )
}
