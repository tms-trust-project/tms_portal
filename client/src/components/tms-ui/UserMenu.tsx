import { useState } from "react"
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
import { useAuth } from "@/tms-hooks/useAuth"
import { useWhoami } from "@/tms-hooks"
import { TokenClaimsModal } from "./TokenClaimsModal"

export function UserMenu() {
  const { data: isAuthenticated } = useAuth()
  const [claimsOpen, setClaimsOpen] = useState(false)
  const { data: whoami } = useWhoami({ enabled: !!isAuthenticated })
  // logged out, the home page shows the sign in card with its own Log In button
  if (!isAuthenticated) return null
  if (!whoami)
    return (
      <Button
        role="link"
        className="bg-neutral-200 text-black hover:bg-highlight hover:text-highlight-foreground"
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
