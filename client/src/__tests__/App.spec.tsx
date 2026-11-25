import { screen, waitFor } from "@testing-library/react"
import Cookies from "js-cookie"
import App from "../App"
import { renderWithQueryClient } from "../components/tms-ui/__tests__/renderWithQueryClient"

const visit = (url: string) => window.history.replaceState(null, "", url)

beforeEach(() => Cookies.remove("tmstoken"))
afterEach(() => {
  Cookies.remove("tmstoken")
  visit("/")
})

describe("App", () => {
  test("shows the sign in card when logged out", async () => {
    renderWithQueryClient(<App />)

    expect(await screen.findByText("Log in to TMS")).toBeInTheDocument()
    expect(screen.queryByText("Linked Providers")).not.toBeInTheDocument()
  })

  test("shows a logged in user's pages", async () => {
    Cookies.set("tmstoken", "test-token")
    renderWithQueryClient(<App />)

    expect(await screen.findByText("Linked Providers")).toBeInTheDocument()
    expect(screen.queryByText("Log in to TMS")).not.toBeInTheDocument()
  })

  test("shows another app's login even to a user logged in to the portal", async () => {
    Cookies.set("tmstoken", "test-token")
    visit(
      "/?response_type=code&client_id=other-portal&redirect_uri=https%3A%2F%2Fother.example.edu%2Fcallback"
    )
    renderWithQueryClient(<App />)

    expect(await screen.findByText("Log in with TMS")).toBeInTheDocument()
    await waitFor(() =>
      expect(screen.queryByText("Linked Providers")).not.toBeInTheDocument()
    )
  })
})
