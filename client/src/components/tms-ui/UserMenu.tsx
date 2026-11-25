import { useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import Cookies from "js-cookie"
import {
  UserCircle2,
  LogOut,
  EllipsisVertical,
  LogIn,
  FileJson,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { loginHref } from "@/lib/loginHref"
import { REQUIRE_INSTITUTION } from "@/lib/loginOptions"
import { useAuth } from "@/tms-hooks/useAuth"
import { useWhoami } from "@/tms-hooks"
import { TokenClaimsModal } from "./TokenClaimsModal"

const loginButtonClass =
  "bg-neutral-200 text-black hover:bg-highlight hover:text-highlight-foreground"

// requireInstitution: never offer the plain Globus login (see src/lib/loginOptions.ts)
export function UserMenu({
  requireInstitution = REQUIRE_INSTITUTION,
}: { requireInstitution?: boolean } = {}) {
  const queryClient = useQueryClient()
  const { data: isAuthenticated } = useAuth()
  const [claimsOpen, setClaimsOpen] = useState(false)
  const { data: whoami, isError } = useWhoami({ enabled: !!isAuthenticated })
  // logged out, the home page shows the sign in card with its own Log In button
  if (!isAuthenticated) return null
  // still loading (or the token was just dropped): show nothing rather than flash Log In
  // at a user who is logged in
  if (!whoami && !isError) return null
  // whoami failed for another reason (e.g. a 5xx), so offer a fresh login.  With
  // requireInstitution that means the sign in card: drop the token cookie, as
  // useWhoami does on a 401, and the home page shows the card in place of the user's
  // pages.  (/logout isn't used because it needs the server to accept the token.)
  if (!whoami && requireInstitution)
    return (
      <Button
        className={loginButtonClass}
        onClick={() => {
          Cookies.remove("tmstoken", { path: "/" })
          queryClient.setQueryData(["auth-cookie"], null)
        }}
      >
        <LogIn /> Log In
      </Button>
    )
  if (!whoami)
    return (
      <Button
        role="link"
        className={loginButtonClass}
        nativeButton={false}
        render={
          <a href={loginHref()}>
            <LogIn /> Log In
          </a>
        }
      />
    )
  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button variant="outline" className="h-12 dark:hover:bg-black/50">
              <UserCircle2 className="size-6 text-muted-foreground" />
              <span className="hidden md:block">{whoami.name}</span>

              <EllipsisVertical className="size-5 text-muted-foreground md:ml-3" />
            </Button>
          }
        />
        <DropdownMenuContent>
          <DropdownMenuGroup>
            <DropdownMenuLabel>
              Provider: {whoami.organization}
            </DropdownMenuLabel>
            <DropdownMenuItem
              className="hover:text-highlight focus:text-highlight not-data-[variant=destructive]:focus:**:text-highlight dark:focus:bg-black/50"
              onClick={() => setClaimsOpen(true)}
            >
              <FileJson /> View token claims
            </DropdownMenuItem>
            {/* the item is the link, so Enter and clicks anywhere on it log out; the server only routes GET /logout */}
            <DropdownMenuItem
              className="hover:text-highlight focus:text-highlight not-data-[variant=destructive]:focus:**:text-highlight dark:focus:bg-black/50"
              render={<a href="/logout" />}
            >
              <LogOut /> Log Out
            </DropdownMenuItem>
          </DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
      <TokenClaimsModal
        token={isAuthenticated}
        open={claimsOpen}
        onOpenChange={setClaimsOpen}
      />
    </>
  )
}
