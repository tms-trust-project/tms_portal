import { vi } from "vitest"
import { screen, render, fireEvent } from "@testing-library/react"
import { DelegationCard, DelegationsListing } from "../DelegationsListing"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { delegations } from "@/stubs/collections"

afterEach(() => delegations.clear())

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

describe("DelegationsCard", () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })
  test("renders expired card", () => {
    const queryClient = new QueryClient()
    const futureDate = new Date("2027-10-24T22:25:40.895247+00:00")
    vi.setSystemTime(futureDate)
    render(
      <QueryClientProvider client={queryClient}>
        <DelegationCard delegation={testDelegation} />
      </QueryClientProvider>
    )
    expect(screen.getByText(/expired/i)).toBeInTheDocument()
    expect(screen.queryByText(/active/i)).not.toBeInTheDocument()
  })

  test("renders active card", () => {
    const queryClient = new QueryClient()
    const futureDate = new Date("2025-10-24T22:25:40.895247+00:00")
    vi.setSystemTime(futureDate)
    render(
      <QueryClientProvider client={queryClient}>
        <DelegationCard delegation={testDelegation} />
      </QueryClientProvider>
    )
    expect(screen.getByText(/active/i)).toBeInTheDocument()
    expect(screen.queryByText(/expired/i)).not.toBeInTheDocument()
  })

  test("renders on invalid expiry", () => {
    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <DelegationCard
          delegation={{ ...testDelegation, expires_at: "invalid" }}
        />
      </QueryClientProvider>
    )
    expect(screen.getByText(/unknown/i)).toBeInTheDocument()
  })
})

describe("delegations listing", () => {
  test("renders an empty list", async () => {
    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <DelegationsListing />
      </QueryClientProvider>
    )
    expect(
      await screen.findByText(/No current delegations/i)
    ).toBeInTheDocument()
  })

  test("renders a delegation card", async () => {
    const queryClient = new QueryClient()
    delegations.create(testDelegation)
    render(
      <QueryClientProvider client={queryClient}>
        <DelegationsListing />
      </QueryClientProvider>
    )
    expect(await screen.findByText(/Link Test/i)).toBeInTheDocument()
  })

  test("revoking a delegation", async () => {
    const queryClient = new QueryClient()
    delegations.create(testDelegation)
    render(
      <QueryClientProvider client={queryClient}>
        <DelegationsListing />
      </QueryClientProvider>
    )
    const button = await screen.findByRole("button", { name: /revoke/i })
    fireEvent.click(button)
    expect(
      await screen.findByText(/No current delegations/i)
    ).toBeInTheDocument()
  })
})
