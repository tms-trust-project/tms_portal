import { fireEvent, screen, waitFor } from "@testing-library/react"
import Cookies from "js-cookie"
import { http, HttpResponse } from "msw"
import { server } from "@/stubs/server"
import { UserMenu } from "../UserMenu"
import { renderWithQueryClient } from "./renderWithQueryClient"

beforeEach(() => Cookies.remove("tmstoken"))
afterEach(() => Cookies.remove("tmstoken"))

describe("UserMenu", () => {
  test("shows Log In when the tmstoken cookie is absent", async () => {
    renderWithQueryClient(<UserMenu />)

    expect(
      await screen.findByRole("link", { name: /Log In/i })
    ).toHaveAttribute("href", "/signin")
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
    const logout = screen.getByRole("button", { name: /Log Out/i })
    expect(logout).toHaveAttribute("type", "submit")
    expect(logout.closest("form")).toHaveAttribute("method", "post")
    expect(logout.closest("form")).toHaveAttribute("action", "/logout")
  })

  test("opens the Tapis sign-out page when logging out of a Tapis login", async () => {
    Cookies.set("tmstoken", "test-token")
    server.use(
      http.get("/login/whoami", () =>
        HttpResponse.json({
          result: {
            name: "jdoe",
            username: "jdoe@tacc@tapis_login",
            organization: "tacc",
            logoutUrl: "https://tacc.tapis.io/v3/oauth2/logout",
          },
        })
      )
    )
    const open = vi.spyOn(window, "open").mockReturnValue(null)
    renderWithQueryClient(<UserMenu />)

    fireEvent.click(await screen.findByRole("button", { name: "jdoe" }))
    const logout = await screen.findByRole("button", { name: /Log Out/i })
    const form = logout.closest("form")!
    form.addEventListener("submit", (event) => event.preventDefault())
    fireEvent.submit(form)
    expect(open).toHaveBeenCalledWith(
      "https://tacc.tapis.io/v3/oauth2/logout",
      "_blank",
      "noopener"
    )
    open.mockRestore()
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
    expect(screen.getByRole("link", { name: /Log In/i })).toHaveAttribute(
      "href",
      "/signin"
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
    expect(screen.getByRole("link", { name: /Log In/i })).toBeInTheDocument()
  })
})
