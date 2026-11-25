// REQUIRE_INSTITUTION is read once, when the module loads, so each case re-imports it
async function requireInstitution(value?: string) {
  if (value !== undefined) vi.stubEnv("VITE_REQUIRE_INSTITUTION", value)
  vi.resetModules()
  return (await import("./loginOptions")).REQUIRE_INSTITUTION
}

afterEach(() => vi.unstubAllEnvs())

describe("REQUIRE_INSTITUTION", () => {
  test("is off when VITE_REQUIRE_INSTITUTION isn't set", async () => {
    expect(await requireInstitution()).toBe(false)
  })

  test('is on for "true"', async () => {
    expect(await requireInstitution("true")).toBe(true)
  })

  // the Dockerfile rejects these; a dev or CI build without that check gets "off"
  test.each(["false", "True", "1", ""])('is off for "%s"', async (value) => {
    expect(await requireInstitution(value)).toBe(false)
  })
})
