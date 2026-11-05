import { screen } from "@testing-library/react"
import { ProviderCard } from "../ProviderCard"
import { renderWithQueryClient } from "./renderWithQueryClient"

const returnUri = "https://gateway.example/jobs/?mode=success"
const linkState = {
  result: "success",
  returnUri,
  clientName: "Link Test",
  providerName: "TACC",
  providerId: "tacc",
}

const providerLink = {
  id: 110,
  tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
  rp_account: "jarosenb@tacc",
  rp_id: "tacc",
  resource_provider_name: "TACC Resource Provider",
  last_login: "2026-09-18T16:41:52.290532+00:00",
  enabled: true,
}

describe("ProviderCard", () => {
  test("Renders manage button when state param is set", async () => {
    window.history.replaceState(
      {},
      "",
      `/?state=${encodeURIComponent(JSON.stringify(linkState))}`
    )

    renderWithQueryClient(<ProviderCard provider={providerLink} />)

    expect(
      await screen.findByRole("button", { name: /Manage/ })
    ).toBeInTheDocument()
  })
})
