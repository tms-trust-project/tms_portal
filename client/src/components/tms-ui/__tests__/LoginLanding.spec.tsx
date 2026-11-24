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

describe("LoginLanding", () => {
  test("lists the institutions and logs in through the selected one", async () => {
    renderWithQueryClient(<LoginLanding />)

    const select = await screen.findByLabelText("Institution")
    expect(
      screen.getByText(
        "Choose your institution to go straight to its login page."
      )
    ).toBeInTheDocument()
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "The University of Texas at Austin",
      "University of Chicago",
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

  test("leaves out disabled institutions and ones without Globus", async () => {
    server.use(
      http.get("/institutions", () =>
        HttpResponse.json({
          result: [
            {
              id: "off",
              name: "Disabled",
              enabled: false,
              federated_identity_config: globus,
            },
            {
              id: "none",
              name: "No Globus",
              enabled: true,
              federated_identity_config: { globus: null },
            },
            {
              id: "ok",
              name: "Approved",
              enabled: true,
              federated_identity_config: globus,
            },
          ],
        })
      )
    )
    renderWithQueryClient(<LoginLanding />)

    await screen.findByLabelText("Institution")
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "Approved",
    ])
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      `${loginBase}&institution=ok`
    )
  })

  test.each([401, 403])(
    "falls back to the plain login, without an error, on a %i before login",
    async (status) => {
      let requests = 0
      server.use(
        http.get("/institutions", () => {
          requests++
          return HttpResponse.json(
            { status: `${status}`, result: "Unable to find TMS token" },
            { status }
          )
        })
      )
      renderWithQueryClient(<LoginLanding />)

      expect(
        await screen.findByRole("link", { name: "Log In" })
      ).toHaveAttribute("href", loginBase)
      expect(
        screen.queryByText("The list of institutions could not be loaded.")
      ).not.toBeInTheDocument()
      expect(screen.queryByLabelText("Institution")).not.toBeInTheDocument()
      expect(
        screen.getByText("Log in with Globus to manage resources.")
      ).toBeInTheDocument()
      expect(requests).toBe(1)
    }
  )

  // a 404 isn't retried, unlike a 5xx, so the error shows without waiting on retries
  test("says the list could not be loaded on other errors", async () => {
    server.use(
      http.get("/institutions", () =>
        HttpResponse.json({ status: "404 Not Found" }, { status: 404 })
      )
    )
    renderWithQueryClient(<LoginLanding />)

    expect(
      await screen.findByText("The list of institutions could not be loaded.")
    ).toBeInTheDocument()
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      loginBase
    )
    expect(
      screen.getByText("Log in with Globus to manage resources.")
    ).toBeInTheDocument()
  })

  test("falls back to the plain login when no institution is approved", async () => {
    server.use(
      http.get("/institutions", () => HttpResponse.json({ result: [] }))
    )
    renderWithQueryClient(<LoginLanding />)

    expect(await screen.findByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      loginBase
    )
    expect(screen.queryByLabelText("Institution")).not.toBeInTheDocument()
    expect(
      screen.getByText("Log in with Globus to manage resources.")
    ).toBeInTheDocument()
  })

  test("shows no Log In while the list loads", () => {
    renderWithQueryClient(<LoginLanding />)
    expect(screen.getByText("Loading institutions…")).toBeInTheDocument()
    expect(screen.queryByRole("link")).not.toBeInTheDocument()
  })
})
