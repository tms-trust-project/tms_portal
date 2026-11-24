import { fireEvent, screen, waitFor } from "@testing-library/react"
import Cookies from "js-cookie"
import { http, HttpResponse } from "msw"
import { server } from "@/stubs/server"
import { UserMenu } from "../UserMenu"
import { renderWithQueryClient } from "./renderWithQueryClient"

beforeEach(() => Cookies.remove("tmstoken"))
afterEach(() => Cookies.remove("tmstoken"))

describe("UserMenu", () => {
  test("leaves Log In to the sign in card when logged out", async () => {
    renderWithQueryClient(<UserMenu />)

    await waitFor(() =>
      expect(
        screen.queryByRole("link", { name: /Log In/i })
      ).not.toBeInTheDocument()
    )
    expect(screen.queryByText("Jake Rosenberg")).not.toBeInTheDocument()
  })

  test("shows the user menu when the tmstoken cookie is present", async () => {
    Cookies.set("tmstoken", "test-token")
    renderWithQueryClient(<UserMenu />)

    const trigger = await screen.findByRole("button", {
      name: "Jake Rosenberg",
    })
    expect(
      screen.queryByRole("link", { name: /Log In/i })
    ).not.toBeInTheDocument()

    fireEvent.click(trigger)
    expect(
      await screen.findByText("Provider: University of Texas at Austin")
    ).toBeInTheDocument()
    // the menu item is the <a> itself, so its role is menuitem, not link
    expect(screen.getByRole("menuitem", { name: /Log Out/i })).toHaveAttribute(
      "href",
      "/logout"
    )
  })

  test("shows the token's claims from View token claims", async () => {
    const claims = {
      sub: "jrosenberg@utexas.edu",
      tms_institution: "utexas",
      name: "Jake Rosenbérg",
      exp: 1794337246,
    }
    // base64url, from UTF-8 bytes, without padding
    const payload = btoa(
      String.fromCharCode(...new TextEncoder().encode(JSON.stringify(claims)))
    )
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "")
    Cookies.set("tmstoken", `eyJhbGciOiJSUzI1NiJ9.${payload}.signature`)
    renderWithQueryClient(<UserMenu />)

    fireEvent.click(
      await screen.findByRole("button", { name: "Jake Rosenberg" })
    )
    fireEvent.click(
      await screen.findByRole("menuitem", { name: "View token claims" })
    )
    const dialog = await screen.findByRole("dialog", { name: "Token claims" })
    expect(JSON.parse(dialog.querySelector("pre")!.textContent!)).toEqual(
      claims
    )
  })

  test("says when the token can't be decoded", async () => {
    Cookies.set("tmstoken", "test-token")
    renderWithQueryClient(<UserMenu />)

    fireEvent.click(
      await screen.findByRole("button", { name: "Jake Rosenberg" })
    )
    fireEvent.click(
      await screen.findByRole("menuitem", { name: "View token claims" })
    )
    expect(
      await screen.findByText("The token could not be decoded.")
    ).toBeInTheDocument()
  })

  test("drops a token cookie the portal no longer accepts", async () => {
    Cookies.set("tmstoken", "revoked-token")
    server.use(
      http.get("/login/whoami", () =>
        HttpResponse.json(
          {
            status: "401 Unauthorized",
            result: "Unauthorized: Token not found",
          },
          { status: 401 }
        )
      )
    )
    renderWithQueryClient(<UserMenu />)

    await waitFor(() => expect(Cookies.get("tmstoken")).toBeUndefined())
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "Jake Rosenberg" })
      ).not.toBeInTheDocument()
    )
  })

  test("keeps the token cookie when whoami fails for another reason", async () => {
    Cookies.set("tmstoken", "test-token")
    let answered = false
    server.use(
      http.get("/login/whoami", () => {
        answered = true
        return HttpResponse.json({}, { status: 500 })
      })
    )
    const { queryClient } = renderWithQueryClient(<UserMenu />)

    await waitFor(() => expect(answered).toBe(true))
    await waitFor(() =>
      expect(queryClient.getQueryState(["auth", "whoami"])?.status).toBe(
        "error"
      )
    )
    expect(Cookies.get("tmstoken")).toBe("test-token")
    expect(screen.getByRole("link", { name: /Log In/i })).toHaveAttribute(
      "href",
      "/login?idp_id=globus_idp&redirect_uri=https%3A%2F%2Ftms-portal.savanna.tacc.cloud%2F"
    )
  })
})
