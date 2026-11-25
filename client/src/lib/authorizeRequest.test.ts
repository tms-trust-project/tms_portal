import { authorizeHref, getAuthorizeRequest } from "./authorizeRequest"

const search =
  "?response_type=code&client_id=other-portal&redirect_uri=https%3A%2F%2Fother.example.edu%2Fcallback%3Fx%3D1&state=s%261&scope=openid"

describe("getAuthorizeRequest", () => {
  test("reads the authorize request the server sent to the sign in page", () => {
    expect(Object.fromEntries(getAuthorizeRequest(search)!)).toEqual({
      response_type: "code",
      client_id: "other-portal",
      redirect_uri: "https://other.example.edu/callback?x=1",
      state: "s&1",
      scope: "openid",
    })
  })

  test("leaves out a missing state and scope", () => {
    const request = getAuthorizeRequest(
      "?response_type=code&client_id=c&redirect_uri=https%3A%2F%2Fr"
    )!
    expect([...request.keys()]).toEqual([
      "response_type",
      "client_id",
      "redirect_uri",
    ])
  })

  test("keeps only the authorize request's own parameters", () => {
    const request = getAuthorizeRequest(`${search}&error=login_failed&x=y`)!
    expect(request.has("error")).toBe(false)
    expect(request.has("x")).toBe(false)
  })

  test.each([
    ["the portal's own page", ""],
    ["a login error", "?error=login_failed"],
    ["no client", "?response_type=code&redirect_uri=https%3A%2F%2Fr"],
    ["no redirect uri", "?response_type=code&client_id=c"],
    [
      "another response type",
      "?response_type=token&client_id=c&redirect_uri=https%3A%2F%2Fr",
    ],
  ])("is undefined for %s", (_, query) => {
    expect(getAuthorizeRequest(query)).toBeUndefined()
  })
})

describe("authorizeHref", () => {
  test("sends the request back to /oauth2/authorize with the institution", () => {
    const href = authorizeHref(getAuthorizeRequest(search)!, "utexas")
    expect(href.startsWith("/oauth2/authorize?")).toBe(true)
    expect(
      Object.fromEntries(new URL(href, "http://localhost").searchParams)
    ).toEqual({
      response_type: "code",
      client_id: "other-portal",
      redirect_uri: "https://other.example.edu/callback?x=1",
      state: "s&1",
      scope: "openid",
      institution: "utexas",
    })
  })

  test("encodes the institution so it can't add parameters", () => {
    const href = authorizeHref(getAuthorizeRequest(search)!, "a&client_id=x")
    const params = new URL(href, "http://localhost").searchParams
    expect(params.get("institution")).toBe("a&client_id=x")
    expect(params.getAll("client_id")).toEqual(["other-portal"])
  })
})
