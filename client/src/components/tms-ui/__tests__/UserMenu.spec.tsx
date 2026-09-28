import { fireEvent, screen } from "@testing-library/react"
import Cookies from "js-cookie"
import { UserMenu } from "../UserMenu"
import { renderWithQueryClient } from "./renderWithQueryClient"

beforeEach(() => Cookies.remove("tmstoken"))
afterEach(() => Cookies.remove("tmstoken"))

describe("UserMenu", () => {
  test("shows Log In when the tmstoken cookie is absent", async () => {
    renderWithQueryClient(<UserMenu />)

    expect(
      await screen.findByRole("link", { name: /Log In/i })
    ).toHaveAttribute(
      "href",
      "/login?idp_id=globus_idp&redirect_uri=https://tms-portal.savanna.tacc.cloud/"
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
    expect(screen.getByRole("link", { name: /Log Out/i })).toHaveAttribute(
      "href",
      "/logout"
    )
  })
})
