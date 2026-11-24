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
import { useListInstitutions } from "@/tms-hooks"

export function LoginLanding() {
  const { data: institutions, isPending, isError } = useListInstitutions()
  const [selected, setSelected] = useState("")
  // only enabled institutions with a Globus redirect domain can steer the login
  const loginInstitutions = (institutions ?? []).filter(
    (institution) =>
      institution.enabled && !!institution.federated_identity_config.globus
  )
  const institutionId = selected || loginInstitutions[0]?.id

  return (
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
        {/* without a list (e.g. /institutions needs a token) fall back to the plain login */}
        {!isPending && (
          <Button
            role="link"
            className="bg-neutral-200 text-black hover:bg-highlight"
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
