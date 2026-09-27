import { Collection } from "@msw/data"
import { z } from "zod"

export const delegations = new Collection({
  schema: z.object({
    id: z.number(),
    client_id: z.string(),
    client_name: z.string(),
    rp_account: z.string(),
    expires_at: z.string(),
    created: z.string(),
    updated: z.string(),
    tms_identity: z.string(),
    rp_id: z.string(),
  }),
})

export const providerLinks = new Collection({
  schema: z.object({
    id: z.number(),
    tms_identity: z.string(),
    rp_account: z.string(),
    rp_id: z.string(),
    resource_provider_name: z.string(),
    last_login: z.string(),
    enabled: z.boolean(),
  }),
})
