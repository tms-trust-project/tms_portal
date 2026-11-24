import { useState } from "react"
import { isAxiosError } from "axios"
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
import { useListInstitutions } from "@/tms-hooks"

// The base server's /institutions needs a TMS token, so before login it always answers
// 401 (or 403).  That's expected, not a failure: there's just no list to choose from.
function isUnauthorized(error: unknown) {
  const status = isAxiosError(error) ? error.response?.status : undefined
  return status === 401 || status === 403
}

export function LoginLanding() {
  const {
    data: institutions,
    isPending,
    isError,
    error,
  } = useListInstitutions()
  const [selected, setSelected] = useState("")
  // only enabled institutions with a Globus redirect domain can steer the login
  const loginInstitutions = (institutions ?? []).filter(
    (institution) =>
      institution.enabled && !!institution.federated_identity_config.globus
  )
  const institutionId = selected || loginInstitutions[0]?.id
  const showPicker = loginInstitutions.length > 0
  const showError = isError && !isUnauthorized(error)

  return (
    <Card className="mx-auto max-w-md">
      <CardHeader>
        <CardTitle>Log in to TMS</CardTitle>
        <CardDescription>
          {/* the base server's /login ignores institution for now (see loginHref) */}
          {showPicker
            ? "Choose your institution to go straight to its login page."
            : "Log in with Globus to manage resources."}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2">
        {isPending && <p className="text-sm">Loading institutions…</p>}
        {showError && (
          <p className="text-sm text-destructive">
            The list of institutions could not be loaded.
          </p>
        )}
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
        {/* without a list (before login, or on an error) fall back to the plain login */}
        {!isPending && (
          <Button
            role="link"
            className="bg-neutral-200 text-black hover:bg-highlight hover:text-highlight-foreground"
            nativeButton={false}
            render={
              <a href={loginHref(institutionId)}>
                <LogIn /> Log In
              </a>
            }
          />
        )}
      </CardFooter>
    </Card>
  )
}
