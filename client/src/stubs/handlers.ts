import { http, HttpResponse } from "msw"

const providerStubs = [
  {
    id: "tacc",
    name: "TACC Resource Provider",
    clientId: "tms_dev_client_id_2",
    oauth2TokenUrl: "https://tacc.tapis.io/v3/oauth2/tokens",
    userInfoUrl: "",
  },
]

const whoamiStub = {
  name: "Jake Rosenberg",
  username: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
  idpDisplayName: "University of Texas at Austin",
  organization: "University of Texas at Austin",
}

const resourceStubs: Record<string, string>[] = [
  {
    id: "cffc01ff-a85d-437c-aebb-a42e05b817a9",
    name: "Stampede",
    description: "Stampede at TACC",
    provider_id: "tacc",
    provider_name: "TACC Resource Provider",
  },
  {
    id: "131d94fb-984e-4ce5-8f4f-adb965e7bc47",
    name: "Frontera",
    description: "Frontera at TACC",
    provider_id: "tacc",
    provider_name: "TACC Resource Provider",
  },
  {
    id: "7982b11e-f823-491e-9863-1096588f98f1",
    name: "Vista",
    description: "Vista at TACC",
    provider_id: "tacc",
    provider_name: "TACC Resource Provider",
  },
]

const providerLinksStub = {
  status: "200 OK",
  result: [
    {
      id: 110,
      tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
      rp_account: "jarosenb@tacc",
      rp_id: "tacc",
      resource_provider_name: "TACC Resource Provider",
      last_login: "2026-09-18T16:41:52.290532+00:00",
      enabled: true,
    },
  ],
}

const delegationsStub = {
  status: "200 OK",
  result: [
    {
      id: 103,
      client_id: "cae18f23-f67c-49e2-926c-695f496c9c58",
      client_name: "Link Test",
      rp_account: "jarosenb@tacc",
      expires_at: "2026-10-24T22:25:40.895247+00:00",
      created: "2026-09-24T22:25:40.918287+00:00",
      updated: "2026-09-24T22:25:40.918287+00:00",
      tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus",
      rp_id: "tacc",
    },
  ],
}

export const handlers = [
  http.get("/login/whoami", () => {
    return HttpResponse.json({ result: whoamiStub })
  }),

  http.get("/resources/providers", () => {
    return HttpResponse.json({ result: providerStubs })
  }),
  http.get("/resources/providers/links", () => {
    return HttpResponse.json(providerLinksStub)
  }),
  http.get("/delegations", () => {
    return HttpResponse.json(delegationsStub)
  }),
  http.delete("/delegations/:clientName/:id", () => {
    return HttpResponse.json({})
  }),
  http.delete<{ id: string }>("/resources/providers/links/:id", () => {
    return HttpResponse.json(providerLinksStub)
  }),
  http.post("/delegations", () => {
    return HttpResponse.json({})
  }),

  http.get<{ provider: string; userId: string }>(
    "/resources/:provider_id/:provider_account_id",
    () => {
      return HttpResponse.json({
        result: resourceStubs,
      })
    }
  ),
]
