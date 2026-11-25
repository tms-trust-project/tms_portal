import { useState } from "react"
import { LogIn } from "lucide-react"
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
import { loginHref } from "@/lib/loginHref"
import { REQUIRE_INSTITUTION } from "@/lib/loginOptions"
import { useListInstitutions } from "@/tms-hooks"

const loginButtonClass =
  "bg-neutral-200 text-black hover:bg-highlight hover:text-highlight-foreground"

// requireInstitution: no plain Globus login, so Log In waits on (and needs) the list
export function LoginLanding({
  requireInstitution = REQUIRE_INSTITUTION,
}: { requireInstitution?: boolean } = {}) {
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
  const showNoneApproved =
    requireInstitution && !isPending && !isError && !showPicker
  const canLogIn = !requireInstitution || !!institutionId

  return (
    <Card className="mx-auto max-w-md">
      <CardHeader>
        <CardTitle>Log in to TMS</CardTitle>
        <CardDescription>
          {/* the base server's /login ignores institution for now (see loginHref) */}
          {showPicker || requireInstitution
            ? "Choose your institution to go straight to its login page."
            : "Log in with Globus to manage resources."}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2">
        {/* the picker always waits on the list; Log In waits only with requireInstitution */}
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
        {/* the plain login until a list arrives (and if none does: on an error, or when no
            institution is approved), then the chosen institution's.  With
            requireInstitution there's no plain login: Log In is a disabled button, with
            no href, until an institution can be chosen. */}
        {canLogIn ? (
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
