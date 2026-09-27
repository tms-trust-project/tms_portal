import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { initializeCollections } from "@/stubs/handlers"
import { LinkIdentityModal } from "../LinkIdentityModal"

const returnUri = "https://gateway.example/jobs/?mode=success"
const clientName = "Link Test"

function expectAuthorizationHref(
  button: HTMLElement,
  providerId: string,
  providerName: string
) {
  const href = button.getAttribute("href")
  expect(href).toBeTruthy()

  const url = new URL(href ?? "", window.location.origin)
  expect(url.pathname).toBe("/resources/providers/authorize")
  expect(url.searchParams.get("provider_id")).toBe(providerId)
  expect(url.searchParams.get("redirect_url")).toBe(window.location.origin)
  expect(JSON.parse(url.searchParams.get("state") ?? "null")).toMatchObject({
    result: "success",
    providerId,
    providerName,
    returnUri,
    clientName,
  })
}

function renderModal({
  clientReturnUri = returnUri,
  state,
}: {
  clientReturnUri?: string | null
  state?: { result: string }
} = {}) {
  const params = new URLSearchParams({ client_name: clientName })
  if (clientReturnUri) params.set("client_return_uri", clientReturnUri)
  if (state) params.set("state", JSON.stringify(state))
  window.history.replaceState({}, "", `/?${params}`)

  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  render(
    <QueryClientProvider client={queryClient}>
      <LinkIdentityModal />
    </QueryClientProvider>
  )
  return queryClient
}

describe("link identity modal", () => {
  beforeEach(() => initializeCollections())
  afterEach(() => window.history.replaceState({}, "", "/"))

  test("renders list of linked identities", async () => {
    renderModal()

    expect(
      await screen.findByText(/TACC Resource Provider/i)
    ).toBeInTheDocument()

    expect(
      screen.getByRole("button", { name: "Return to Science Gateway" })
    ).toHaveAttribute("href", returnUri)

    expectAuthorizationHref(
      screen.getByRole("button", { name: /refresh/i }),
      "tacc",
      "TACC Resource Provider"
    )

    expect(screen.getByText(/Add Provider/i)).toBeInTheDocument()
  })

  test("unlink and re-link", async () => {
    renderModal()
    const unlinkButton = await screen.findByRole("button", {
      name: /Disconnect/,
    })
    fireEvent.click(unlinkButton)

    const connectButton = await screen.findByRole("button", { name: /Connect/ })
    expectAuthorizationHref(connectButton, "tacc", "TACC Resource Provider")

    expect(screen.queryByText(/Disconnect/)).not.toBeInTheDocument()
  })

  test("stays closed after a successful callback until Add Provider is clicked", async () => {
    renderModal({ state: { result: "success" } })

    const addProvider = screen.getByRole("button", { name: /Add Provider/i })
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument()

    fireEvent.click(addProvider)
    expect(await screen.findByRole("dialog")).toBeInTheDocument()
  })

  test("does not prompt linking without a client return URI", async () => {
    const queryClient = renderModal({ clientReturnUri: null })
    await waitFor(() =>
      expect(queryClient.getQueryData(["providerLinks"])).toBeDefined()
    )

    expect(
      screen.queryByRole("button", { name: /Add Provider/i })
    ).not.toBeInTheDocument()
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument()
  })
})
