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

  test("falls back to the plain login when the list needs a token", async () => {
    let requests = 0
    server.use(
      http.get("/institutions", () => {
        requests++
        return HttpResponse.json(
          { status: "401 Unauthorized", result: "Unable to find TMS token" },
          { status: 401 }
        )
      })
    )
    renderWithQueryClient(<LoginLanding />)

    expect(
      await screen.findByText("The list of institutions could not be loaded.")
    ).toBeInTheDocument()
    expect(screen.queryByLabelText("Institution")).not.toBeInTheDocument()
    expect(screen.getByRole("link", { name: "Log In" })).toHaveAttribute(
      "href",
      loginBase
    )
    expect(requests).toBe(1)
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
  })

  test("shows no Log In while the list loads", () => {
    renderWithQueryClient(<LoginLanding />)
    expect(screen.getByText("Loading institutions…")).toBeInTheDocument()
    expect(screen.queryByRole("link")).not.toBeInTheDocument()
  })
})
