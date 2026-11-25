import { fireEvent, screen } from "@testing-library/react"
import { http, HttpResponse } from "msw"
import { server } from "@/stubs/server"
import { LoginLanding } from "../LoginLanding"
import { renderWithQueryClient } from "./renderWithQueryClient"

const loginBase =
  "/login?idp_id=globus_idp&redirect_uri=https%3A%2F%2Ftms-portal.savanna.tacc.cloud%2F"

const globus = {
  globus: { redirect_idp_domain: "example.edu", idp_logout_url: "" },
}

const listError = "The list of institutions could not be loaded."
const noneApproved = "No institutions are available for login."
const chooseText = "Choose your institution to go straight to its login page."

const loginLink = () => screen.getByRole("link", { name: "Log In" })
// Log In is a disabled button until it has an institution
const disabledLogIn = () => screen.getByRole("button", { name: "Log In" })
// no rendered link may ever be a login without an institution
const expectNoLoginLink = () =>
  expect(screen.queryByRole("link")).not.toBeInTheDocument()

// answer /institutions with an error status, counting the requests
function failInstitutions(status: number) {
  const calls = { requests: 0 }
  server.use(
    http.get("/institutions", () => {
      calls.requests++
      return HttpResponse.json(
        { status: `${status}`, result: "error" },
        { status }
      )
    })
  )
  return calls
}

describe("LoginLanding", () => {
  test("lists the institutions without a login and logs in through the selected one", async () => {
    let authorization: string | null = "unset"
    server.use(
      http.get("/institutions", ({ request }) => {
        authorization = request.headers.get("Authorization")
        // fall through to the default stub
        return undefined
      })
    )
    renderWithQueryClient(<LoginLanding />)

    const select = await screen.findByLabelText("Institution")
    // /institutions is public: no tmstoken cookie, so no bearer token was sent
    expect(authorization).toBeNull()
    expect(screen.getByText(chooseText)).toBeInTheDocument()
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "The University of Texas at Austin",
      "University of Chicago",
    ])
    expect(loginLink()).toHaveAttribute(
      "href",
      `${loginBase}&institution=utexas`
    )

    fireEvent.change(select, { target: { value: "uchicago" } })
    expect(loginLink()).toHaveAttribute(
      "href",
      `${loginBase}&institution=uchicago`
    )
  })

  // the server's own envelope and field names (institutions/responses/mod.rs)
  test("reads the server's response, leaving out entries that can't steer the login", async () => {
    server.use(
      http.get("/institutions", () =>
        HttpResponse.json({
          status: "200 OK",
          result: [
            { id: "off", name: "Disabled", enabled: false, idp_config: globus },
            {
              id: "none",
              name: "No Globus",
              enabled: true,
              idp_config: { globus: null },
            },
            // malformed: no idp_config at all, which must not crash the page
            { id: "bad", name: "Malformed", enabled: true },
            { id: "ok", name: "Approved", enabled: true, idp_config: globus },
          ],
        })
      )
    )
    renderWithQueryClient(<LoginLanding />)

    await screen.findByLabelText("Institution")
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "Approved",
    ])
    expect(loginLink()).toHaveAttribute("href", `${loginBase}&institution=ok`)
  })

  test("keeps Log In disabled while the list loads, then links to the institution", async () => {
    renderWithQueryClient(<LoginLanding />)

    expect(screen.getByText(chooseText)).toBeInTheDocument()
    expect(screen.getByText("Loading institutions…")).toBeInTheDocument()
    expect(disabledLogIn()).toBeDisabled()
    expect(disabledLogIn()).toHaveAttribute("aria-describedby", "login-status")
    expectNoLoginLink()

    expect(await screen.findByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=utexas`
    )
    expect(screen.queryByText("Loading institutions…")).not.toBeInTheDocument()
    expect(
      screen.queryByRole("button", { name: "Log In" })
    ).not.toBeInTheDocument()
  })

  // a 4xx isn't retried, so the error shows after one request; a 5xx is retried once
  test.each([
    [401, 1],
    [403, 1],
    [404, 1],
    [503, 2],
  ])(
    "leaves Log In disabled when the list fails with a %i",
    async (status, requests) => {
      const calls = failInstitutions(status)
      // the hook's own retry rule still applies; only the backoff between tries is skipped
      renderWithQueryClient(<LoginLanding />, { queries: { retryDelay: 0 } })

      expect(await screen.findByText(listError)).toBeInTheDocument()
      expect(disabledLogIn()).toBeDisabled()
      expect(screen.queryByLabelText("Institution")).not.toBeInTheDocument()
      expectNoLoginLink()
      expect(screen.getByText(chooseText)).toBeInTheDocument()
      expect(calls.requests).toBe(requests)
    }
  )

  test.each([
    ["is empty", []],
    [
      "has nothing approved",
      [
        { id: "off", name: "Disabled", enabled: false, idp_config: globus },
        {
          id: "none",
          name: "No Globus",
          enabled: true,
          idp_config: { globus: null },
        },
      ],
    ],
  ])("leaves Log In disabled when the list %s", async (_, result) => {
    server.use(
      http.get("/institutions", () =>
        HttpResponse.json({ status: "200 OK", result })
      )
    )
    renderWithQueryClient(<LoginLanding />)

    expect(await screen.findByText(noneApproved)).toBeInTheDocument()
    expect(disabledLogIn()).toBeDisabled()
    expect(screen.queryByLabelText("Institution")).not.toBeInTheDocument()
    expect(screen.queryByText(listError)).not.toBeInTheDocument()
    expectNoLoginLink()
  })
})

describe("LoginLanding login errors", () => {
  const visit = (url: string) => window.history.replaceState(null, "", url)
  afterEach(() => visit("/"))

  test("explains a refused login and drops the error from the url", async () => {
    visit("/?error=unapproved_institution")
    renderWithQueryClient(<LoginLanding />)

    expect(screen.getByRole("alert")).toHaveTextContent(
      "That login isn't from an approved institution."
    )
    // a reload wouldn't show it again
    expect(window.location.search).toBe("")
    await screen.findByLabelText("Institution")
  })

  test("keeps the rest of the query string", () => {
    visit("/?error=login_cancelled&client_id=other&state=s")
    renderWithQueryClient(<LoginLanding />)

    expect(screen.getByRole("alert")).toHaveTextContent(
      "The login was cancelled."
    )
    expect(window.location.search).toBe("?client_id=other&state=s")
  })

  test("can be dismissed", () => {
    visit("/?error=login_expired")
    renderWithQueryClient(<LoginLanding />)

    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }))
    expect(screen.queryByRole("alert")).not.toBeInTheDocument()
  })

  test("shows nothing without an error", () => {
    renderWithQueryClient(<LoginLanding />)

    expect(screen.queryByRole("alert")).not.toBeInTheDocument()
  })
})
