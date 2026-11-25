import { useEffect, useState } from "react"
import { CircleAlert, LogIn, X } from "lucide-react"
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
import { loginErrorMessage, withoutLoginError } from "@/lib/loginError"
import { loginHref } from "@/lib/loginHref"
import { useListInstitutions } from "@/tms-hooks"

/*
Why the last login didn't work, from the ?error=<code> the server adds when it sends a
refused or failed login back here.  The error is dropped from the url straight away, so a
reload doesn't show it again; the message stays until it's dismissed.
 */
function LoginErrorAlert() {
  const [message, setMessage] = useState(() =>
    loginErrorMessage(window.location.search)
  )
  useEffect(() => {
    const { pathname, search, hash } = window.location
    if (!new URLSearchParams(search).has("error")) return
    window.history.replaceState(
      window.history.state,
      "",
      `${pathname}${withoutLoginError(search)}${hash}`
    )
  }, [])
  if (!message) return null

  return (
    <Alert variant="destructive">
      <CircleAlert />
      <AlertDescription>{message}</AlertDescription>
      <AlertAction>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Dismiss"
          onClick={() => setMessage(undefined)}
        >
          <X />
        </Button>
      </AlertAction>
    </Alert>
  )
}

const loginButtonClass =
  "bg-neutral-200 text-black hover:bg-highlight hover:text-highlight-foreground"

// Every login goes through an institution (the server refuses one without), so Log In
// waits on (and needs) the list.
export function LoginLanding() {
  // /institutions is public, so any failure (a 401 included) means the list is unavailable
  const { data: institutions, isPending, isError } = useListInstitutions()
  const [selected, setSelected] = useState("")
  // only enabled institutions with a Globus redirect domain can steer the login;
  // the ?. keeps one malformed entry from crashing the sign in page
  const loginInstitutions = (institutions ?? []).filter(
    (institution) => institution.enabled && !!institution.idp_config?.globus
  )
  const institutionId = selected || loginInstitutions[0]?.id
  const showPicker = loginInstitutions.length > 0
  const showError = isError
  // the list loaded, but nothing in it can be used to log in
  const showNoneApproved = !isPending && !isError && !showPicker

  return (
    <Card className="mx-auto max-w-md">
      <CardHeader>
        <CardTitle>Log in to TMS</CardTitle>
        <CardDescription>
          {/* /login sends the chosen institution's logins to its own domain (see loginHref) */}
          Choose your institution to go straight to its login page.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2">
        <LoginErrorAlert />
        {/* the picker and Log In both wait on the list */}
        <div id="login-status" aria-live="polite">
          {isPending && (
            <p className="text-sm text-muted-foreground">
              Loading institutions…
            </p>
          )}
          {showError && (
            <p className="text-sm text-destructive">
              The list of institutions could not be loaded.
            </p>
          )}
          {showNoneApproved && (
            <p className="text-sm text-destructive">
              No institutions are available for login.
            </p>
          )}
        </div>
        {showPicker && (
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
        {/* the chosen institution's login; until there is one (while the list loads, or for
            good on an error or when no institution is approved) Log In is a disabled
            button, with no href */}
        {institutionId ? (
          <Button
            role="link"
            className={loginButtonClass}
            nativeButton={false}
            render={
              <a href={loginHref(institutionId)}>
                <LogIn /> Log In
              </a>
            }
          />
        ) : (
          <Button
            disabled
            aria-describedby="login-status"
            className={loginButtonClass}
          >
            <LogIn /> Log In
          </Button>
        )}
      </CardFooter>
    </Card>
  )
}
