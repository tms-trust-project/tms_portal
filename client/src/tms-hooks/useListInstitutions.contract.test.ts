/*
The client's Institution types against the server's OpenAPI spec (generateSpecs.sh writes
specs/tms_portal_openapi.yaml from the server's code).  A field renamed on one side and not
the other (as ipd_config became idp_config) fails here, rather than the sign in card
silently dropping every institution.
 */
import spec from "../../../specs/tms_portal_openapi.yaml?raw"
import type { GlobusConfiguration, Institution } from "./useListInstitutions"

type Schema = { required: string[]; properties: string[] }

// One schema from components/schemas.  utoipa writes it in a fixed layout (schema names
// at 4 spaces, its keys at 6, property names at 8, list items at 6), which is all this
// reads; it fails loudly rather than guess at anything else.
function schema(name: string): Schema {
  const lines = spec.split("\n")
  const start = lines.indexOf(`    ${name}:`)
  if (start < 0) throw new Error(`no ${name} schema in the spec`)
  const end = lines.findIndex((line, i) => i > start && /^ {0,4}\S/.test(line))
  const block = lines.slice(start + 1, end < 0 ? undefined : end)

  const section = (key: string) => {
    const from = block.indexOf(`      ${key}:`)
    if (from < 0) return []
    // the section ends at the schema's next key; its list items are at the same indent
    const to = block.findIndex((line, i) => i > from && /^ {6}\w/.test(line))
    return block.slice(from + 1, to < 0 ? undefined : to)
  }
  return {
    required: section("required").flatMap((line) => {
      const item = /^ {6}- (\S+)$/.exec(line)
      return item ? [item[1]] : []
    }),
    properties: section("properties").flatMap((line) => {
      const key = /^ {8}(\w+):$/.exec(line)
      return key ? [key[1]] : []
    }),
  }
}

// Typed samples: TypeScript requires every field of each type and allows no others, so
// their keys are exactly the fields the client reads.
const globus: GlobusConfiguration = {
  redirect_idp_domain: "utexas.edu",
  idp_logout_url: "https://enterprise.login.utexas.edu/idp/profile/Logout",
}
const idpConfig: Institution["idp_config"] = { globus }
const institution: Institution = {
  id: "utexas",
  name: "The University of Texas at Austin",
  enabled: true,
  idp_config: idpConfig,
}

const sorted = (keys: string[]) => [...keys].sort()

describe("Institution types match the server's OpenAPI spec", () => {
  test.each([
    ["Institution", institution],
    ["IdpConfig", idpConfig],
    ["GlobusConfiguration", globus],
  ])("%s has the spec's properties", (name, sample) => {
    expect(sorted(Object.keys(sample))).toEqual(sorted(schema(name).properties))
  })

  // the client relies on every field it reads always being sent
  test.each([
    ["Institution", institution],
    ["GlobusConfiguration", globus],
  ])("%s's fields are all required by the spec", (name, sample) => {
    expect(sorted(schema(name).required)).toEqual(sorted(Object.keys(sample)))
  })

  test("Institution's idp_config is the spec's IdpConfig", () => {
    expect(spec).toMatch(
      /^ {8}idp_config:\n {10}\$ref: '#\/components\/schemas\/IdpConfig'$/m
    )
  })

  test("fails loudly for a schema the spec doesn't have", () => {
    expect(() => schema("NoSuchSchema")).toThrow("no NoSuchSchema schema")
  })
})
