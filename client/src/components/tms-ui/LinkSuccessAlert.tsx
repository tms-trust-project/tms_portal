import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { useState } from "react"
import { Button } from "@/components/ui/button"

export function LinkSuccessAlert() {
  const linkStateRawParam = new URLSearchParams(window.location.search).get(
    "state"
  )
  const linkStateParams = linkStateRawParam
    ? JSON.parse(linkStateRawParam)
    : undefined

  const returnUrl = linkStateParams?.returnUri
  const isSuccessState = linkStateParams?.result === "success"
  const providerId = linkStateParams?.providerId
  const [isOpen, setIsOpen] = useState(isSuccessState)
  return (
    <AlertDialog open={isOpen} onOpenChange={setIsOpen}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            Account Linking Successful{" "}
            {providerId && (
              <span>
                for <code>{providerId}</code>
              </span>
            )}
          </AlertDialogTitle>
          <AlertDialogDescription>
            You can proceed to the science gateway, or remain on this page to
            continue managing your linked providers.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Close Dialog</AlertDialogCancel>
          <Button asChild>
            <a href={returnUrl}>Return to Science Gateway</a>
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
