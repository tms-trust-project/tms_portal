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
