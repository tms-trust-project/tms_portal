import { useState } from "react"
import { UserCircle2, LogOut, EllipsisVertical, LogIn } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useAuth } from "@/tms-hooks/useAuth"
import { useWhoami } from "@/tms-hooks"
import { TokenClaimsModal } from "./TokenClaimsModal"

export function UserMenu() {
  const { data: isAuthenticated } = useAuth()
  const [claimsOpen, setClaimsOpen] = useState(false)
  const { data: whoami } = useWhoami({ enabled: !!isAuthenticated })
  if (!isAuthenticated || !whoami)
    return (
      <Button
        variant="outline"
        role="link"
        nativeButton={false}
        render={
          <a href="/signin">
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
            <Button variant="outline" className="h-12">
              <UserCircle2 className="size-6 text-muted-foreground" />
              <span className="hidden md:block">{whoami.name}</span>

              <EllipsisVertical className="size-5 text-muted-foreground md:ml-3" />
            </Button>
          }
        />
        <DropdownMenuContent>
          <DropdownMenuGroup>
            <DropdownMenuItem
              className="text-primary underline-offset-4 hover:underline"
              onClick={() => setClaimsOpen(true)}
            >
              Provider: {whoami.organization}
            </DropdownMenuItem>
            <DropdownMenuItem>
              {/* a POST, so another site can't log the user out with a link */}
              <form
                method="post"
                action="/logout"
                onSubmit={() => {
                  if (whoami.logoutUrl)
                    window.open(whoami.logoutUrl, "_blank", "noopener")
                }}
              >
                <button type="submit" className="flex items-center gap-2">
                  <LogOut /> Log Out
                </button>
              </form>
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
