import { screen } from "@testing-library/react"
import { ClientSelect } from "../ClientSelect"
import userEvent from "@testing-library/user-event"

import { renderWithQueryClient } from "./renderWithQueryClient"

describe("ClientSelect", () => {
  test("renders dropdown", async () => {
    const user = userEvent.setup()
    renderWithQueryClient(<ClientSelect />)
    expect(
      await screen.findByRole("button", { name: /Select Client/ })
    ).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /Select Client/ }))
    expect(
      await screen.findByRole("menuitem", { name: /Link Test/ })
    ).toHaveAttribute(
      "href",
      "/?client_return_uri=http%3A%2F%2Flocalhost%3A8000&client_name=Link%20Test"
    )
  })

  test("renders message when no client_name is present", () => {
    renderWithQueryClient(<ClientSelect />)
    expect(screen.getByText(/no client gateway/i)).toBeInTheDocument()
  })

  test("renders message when client_name is present in client_name param", () => {
    window.history.replaceState({}, "", `/?client_name=Link%20Test))}`)
    renderWithQueryClient(<ClientSelect />)
    expect(screen.getByText(/Link Test/i)).toBeInTheDocument()
  })

  test("renders message when client_name is present in state param", () => {
    window.history.replaceState(
      {},
      "",
      `/?state=${encodeURIComponent(JSON.stringify({ clientName: "Link Test" }))}`
    )
    renderWithQueryClient(<ClientSelect />)
    expect(screen.getByText(/Link Test/i)).toBeInTheDocument()
  })
})
