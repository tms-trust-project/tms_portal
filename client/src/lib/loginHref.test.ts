import { loginHref } from "./loginHref"

const params = (href: string) => new URL(href, "http://localhost").searchParams

describe("loginHref", () => {
  test("logs in through the institution and returns to the redirect uri", () => {
    const href = loginHref("utexas")
    expect(href.startsWith("/login?")).toBe(true)
    expect(Object.fromEntries(params(href))).toEqual({
      idp_id: "globus_idp",
      redirect_uri: __REDIRECT_URI__,
      institution: "utexas",
    })
  })

  test("encodes values so they can't add parameters", () => {
    const search = params(loginHref("a&idp_id=other"))
    expect(search.get("institution")).toBe("a&idp_id=other")
    expect(search.getAll("idp_id")).toEqual(["globus_idp"])
  })
})
