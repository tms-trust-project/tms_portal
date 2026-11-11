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
import { authorizeHref, getAuthorizeRequest } from "@/lib/authorizeRequest"
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

// the providers the portal can log in through (institution_service.rs skips any others)
const LOGIN_PROVIDERS = ["globus", "tacc_tapis"]

// another app's authorize request continues at /oauth2/authorize, the portal's own at /login
function loginHref(
  institutionId: string,
  authorizeRequest: URLSearchParams | undefined
) {
  if (authorizeRequest) return authorizeHref(authorizeRequest, institutionId)
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
    // drop the error so a reload doesn't show it again, keeping any authorize request
    const params = new URLSearchParams(window.location.search)
    params.delete("error")
    const query = params.toString()
    window.history.replaceState(
      null,
      "",
      query ? `${window.location.pathname}?${query}` : window.location.pathname
    )
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
  // Globus institutions and Tapis share the list; the portal picks the login idp by provider
  const loginInstitutions = (institutions ?? []).filter((institution) =>
    LOGIN_PROVIDERS.includes(institution.provider)
  )
  const institutionId = selected || loginInstitutions[0]?.id
  const authorizeRequest = getAuthorizeRequest(window.location.search)

  return (
    <>
      <LoginErrorAlert />
      <Card className="mx-auto max-w-md">
        <CardHeader>
          <CardTitle>
            {authorizeRequest ? "Log in with TMS" : "Log in to TMS"}
          </CardTitle>
          <CardDescription>
            Choose your institution to go straight to its login page.
            {authorizeRequest &&
              " You'll then return to the app that asked you to log in."}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {isPending && <p className="text-sm">Loading institutions…</p>}
          {isError && (
            <p className="text-sm text-destructive">
              The list of institutions could not be loaded.
            </p>
          )}
          {!!loginInstitutions.length && (
            <>
              <Label htmlFor="institution">Institution</Label>
              <select
                id="institution"
                className="w-full rounded-md border bg-background px-3 py-2 text-sm"
                value={institutionId}
                onChange={(event) => setSelected(event.target.value)}
              >
                {loginInstitutions.map((institution) => (
                  <option key={institution.id} value={institution.id}>
                    {institution.name}
                  </option>
                ))}
              </select>
            </>
          )}
        </CardContent>
        <CardFooter>
          {institutionId && (
            <Button
              role="link"
              className="bg-neutral-200 text-black hover:bg-burnt-orange"
              nativeButton={false}
              render={
                <a href={loginHref(institutionId, authorizeRequest)}>
                  <LogIn /> Log In
                </a>
              }
            />
          )}
        </CardFooter>
      </Card>
    </>
  )
}
