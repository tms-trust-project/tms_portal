import { fireEvent, screen } from "@testing-library/react"
import { http, HttpResponse } from "msw"
import { server } from "@/stubs/server"
import { LoginLanding } from "../LoginLanding"
import { renderWithQueryClient } from "./renderWithQueryClient"

const loginBase =
  "/login?idp_id=globus_idp&redirect_uri=https://tms-portal.savanna.tacc.cloud/"
const bannerText =
  "Please login with one of the approved institutions in the list"

afterEach(() => window.history.replaceState(null, "", "/"))

describe("LoginLanding", () => {
  test("lists the institutions and logs in through the selected one", async () => {
    renderWithQueryClient(<LoginLanding />)

    const select = await screen.findByLabelText("Institution")
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "The University of Texas at Austin",
      "University of Chicago",
      "Texas Advanced Computing Center",
    ])
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=utexas`
    )

    fireEvent.change(select, { target: { value: "uchicago" } })
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=uchicago`
    )
  })

  test("logs in with Tapis through its entry in the list", async () => {
    renderWithQueryClient(<LoginLanding />)

    fireEvent.change(await screen.findByLabelText("Institution"), {
      target: { value: "tacc" },
    })
    expect(screen.getAllByRole("link")).toHaveLength(1)
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=tacc`
    )
  })

  test("leaves out institutions with providers the portal can't log in through", async () => {
    server.use(
      http.get("/login/institutions", () =>
        HttpResponse.json({
          result: [
            { id: "other", name: "Other Provider", provider: "other" },
            {
              id: "tacc",
              name: "Texas Advanced Computing Center",
              provider: "tacc_tapis",
            },
          ],
        })
      )
    )
    renderWithQueryClient(<LoginLanding />)

    await screen.findByLabelText("Institution")
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "Texas Advanced Computing Center",
    ])
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=tacc`
    )
  })

  test("sends another app's authorize request back with the chosen institution", async () => {
    window.history.replaceState(
      null,
      "",
      "/signin?response_type=code&client_id=gateway&redirect_uri=https%3A%2F%2Fgateway.example.org%2Fcallback%3Fkeep%3D1&state=a%26b%3Dc&scope=ignored"
    )
    renderWithQueryClient(<LoginLanding />)

    expect(screen.getByText("Log in with TMS")).toBeInTheDocument()
    const authorizeBase =
      "/oauth2/authorize?response_type=code&client_id=gateway&redirect_uri=https%3A%2F%2Fgateway.example.org%2Fcallback%3Fkeep%3D1&state=a%26b%3Dc"
    expect(await screen.findByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${authorizeBase}&institution=utexas`
    )
    fireEvent.change(screen.getByLabelText("Institution"), {
      target: { value: "tacc" },
    })
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${authorizeBase}&institution=tacc`
    )
  })

  test("needs a client and redirect uri to be another app's request", async () => {
    window.history.replaceState(null, "", "/signin?client_id=gateway")
    renderWithQueryClient(<LoginLanding />)

    expect(screen.getByText("Log in to TMS")).toBeInTheDocument()
    expect(await screen.findByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=utexas`
    )
  })

  test("keeps another app's authorize request when an error is dismissed", () => {
    window.history.replaceState(
      null,
      "",
      "/signin?client_id=gateway&redirect_uri=https%3A%2F%2Fgateway.example.org%2F&error=login_failed"
    )
    renderWithQueryClient(<LoginLanding />)

    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }))
    expect(window.location.search).toBe(
      "?client_id=gateway&redirect_uri=https%3A%2F%2Fgateway.example.org%2F"
    )
  })

  test("shows no login links when the list fails to load", async () => {
    server.use(
      http.get("/login/institutions", () =>
        HttpResponse.json({}, { status: 500 })
      )
    )
    renderWithQueryClient(<LoginLanding />)

    expect(
      await screen.findByText("The list of institutions could not be loaded.")
    ).toBeInTheDocument()
    expect(screen.queryByRole("link")).not.toBeInTheDocument()
  })

  test("shows a dismissable banner after an unapproved institution login", async () => {
    window.history.replaceState(
      null,
      "",
      "/signin?error=unapproved_institution"
    )
    renderWithQueryClient(<LoginLanding />)

    expect(screen.getByText(bannerText)).toBeInTheDocument()
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }))
    expect(screen.queryByText(bannerText)).not.toBeInTheDocument()
    expect(window.location.search).toBe("")
  })

  test("asks to log in again after a reused Globus session", () => {
    window.history.replaceState(null, "", "/signin?error=stale_authentication")
    renderWithQueryClient(<LoginLanding />)

    expect(
      screen.getByText(
        "Your login could not be confirmed as new. Please log in again."
      )
    ).toBeInTheDocument()
    expect(screen.queryByText(bannerText)).not.toBeInTheDocument()
  })

  test("shows no banner without the error", () => {
    renderWithQueryClient(<LoginLanding />)
    expect(screen.queryByText(bannerText)).not.toBeInTheDocument()
  })

  test.each([
    [
      "login_cancelled",
      "The login was cancelled. Choose your institution to try again.",
    ],
    ["login_failed", "The login could not be completed. Please try again."],
    [
      "login_expired",
      "Your login took too long or was started again in another tab. Please log in again.",
    ],
    [
      "login_unavailable",
      "Logging in is unavailable right now. Please try again later.",
    ],
  ])("explains a %s login", (error, message) => {
    window.history.replaceState(null, "", `/signin?error=${error}`)
    renderWithQueryClient(<LoginLanding />)
    expect(screen.getByText(message)).toBeInTheDocument()
  })

  test("shows no banner for an unknown error", () => {
    window.history.replaceState(null, "", "/signin?error=something_else")
    renderWithQueryClient(<LoginLanding />)
    expect(screen.queryByRole("button", { name: "Dismiss" })).toBeNull()
  })
})
