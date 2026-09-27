import { fireEvent, render, screen } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { HttpResponse, http } from "msw"
import { delegations } from "@/stubs/collections"
import { server } from "@/stubs/server"
import { LinkSuccessAlert } from "../LinkSuccessAlert"

const returnUri = "https://gateway.example/jobs"
const linkState = {
  result: "success",
  returnUri,
  clientName: "Link Test",
  providerName: "TACC",
  providerId: "tacc",
}

const testDelegation = {
  id: 103,
  client_id: "cae18f23-f67c-49e2-926c-695f496c9c58",
  client_name: "Link Test",
  rp_account: "jarosenb@tacc",
  expires_at: "2026-10-24T22:25:40.895247+00:00",
  created: "2026-09-24T22:25:40.918287+00:00",
  updated: "2026-09-24T22:25:40.918287+00:00",
  tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
  rp_id: "tacc",
}

function renderAlert(state = linkState) {
  window.history.replaceState(
    {},
    "",
    `/?state=${encodeURIComponent(JSON.stringify(state))}`
  )
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  render(
    <QueryClientProvider client={queryClient}>
      <LinkSuccessAlert />
    </QueryClientProvider>
  )
}

afterEach(() => {
  delegations.clear()
  window.history.replaceState({}, "", "/")
})

describe("LinkSuccessAlert", () => {
  test("shows the linked account and return destination", async () => {
    renderAlert()

    expect(
      await screen.findByText("Authentication Successful")
    ).toBeInTheDocument()
    expect(screen.getByText(/jarosenb@tacc/)).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: /Confirm Delegation/i })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: "Return to Science Gateway" })
    ).toHaveAttribute("href", returnUri)
  })

  test("shows an existing delegation without offering to create another", async () => {
    delegations.create(testDelegation)
    renderAlert()

    expect(
      await screen.findByText("Authentication Successful")
    ).toBeInTheDocument()
    expect(await screen.findByText("Link Test")).toBeInTheDocument()
    expect(
      screen.queryByRole("button", { name: /Confirm Delegation/i })
    ).not.toBeInTheDocument()
  })

  test("submits the selected provider account and shows success", async () => {
    renderAlert()
    fireEvent.click(
      await screen.findByRole("button", { name: /Confirm Delegation/i })
    )

    expect(await screen.findByText("Delegation successful")).toBeInTheDocument()

    expect(
      screen.queryByRole("button", { name: /Confirm Delegation/i })
    ).not.toBeInTheDocument()
  })

  test("shows an error and retry action when delegation fails", async () => {
    server.use(
      http.post("/delegations", () => HttpResponse.json({}, { status: 500 }))
    )
    renderAlert()

    fireEvent.click(
      await screen.findByRole("button", { name: /Confirm Delegation/i })
    )

    expect(await screen.findByText("Delegation Error")).toBeInTheDocument()
    server.resetHandlers()
    const retryButton = screen.getByRole("button", { name: /Try again/i })

    fireEvent.click(retryButton)
    expect(await screen.findByText("Delegation successful")).toBeInTheDocument()
    expect(screen.queryByText("Delegation Error")).not.toBeInTheDocument()
  })

  test("uses the most recently linked account for the provider", async () => {
    server.use(
      http.get("/resources/providers/links", () =>
        HttpResponse.json({
          status: "200 OK",
          result: [
            {
              id: 110,
              tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
              rp_account: "jarosenb_new@tacc",
              rp_id: "tacc",
              resource_provider_name: "TACC Resource Provider",
              last_login: "2026-09-18T16:41:52.290532+00:00",
              enabled: true,
            },
            {
              id: 111,
              tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
              rp_account: "jarosenb_old@tacc",
              rp_id: "tacc",
              resource_provider_name: "TACC Resource Provider",
              last_login: "2025-09-18T16:41:52.290532+00:00",
              enabled: true,
            },
          ],
        })
      )
    )
    renderAlert()
    expect(
      await screen.findByText("Authentication Successful")
    ).toBeInTheDocument()
    expect(screen.getByText(/jarosenb_new@tacc/)).toBeInTheDocument()
    expect(screen.queryByText(/jarosenb_old@tacc/)).not.toBeInTheDocument()
  })
})
