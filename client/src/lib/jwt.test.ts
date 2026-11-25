import { decodeJwtClaims } from "./jwt"

// base64url, from UTF-8 bytes, without padding
const encode = (text: string) =>
  btoa(String.fromCharCode(...new TextEncoder().encode(text)))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "")

const token = (payload: string) => `eyJhbGciOiJSUzI1NiJ9.${payload}.signature`

describe("decodeJwtClaims", () => {
  test("decodes a JSON object payload", () => {
    expect(decodeJwtClaims(token(encode('{"sub":"jrosenberg"}')))).toEqual({
      sub: "jrosenberg",
    })
  })

  test("is undefined for a token without a payload", () => {
    expect(decodeJwtClaims("test-token")).toBeUndefined()
  })

  test("is undefined when the payload isn't base64url", () => {
    expect(decodeJwtClaims(token("!!!"))).toBeUndefined()
  })

  test("is undefined when the payload isn't JSON", () => {
    expect(decodeJwtClaims(token(encode("not json")))).toBeUndefined()
  })

  test("is undefined when the payload isn't a JSON object", () => {
    expect(decodeJwtClaims(token(encode("[1,2]")))).toBeUndefined()
    expect(decodeJwtClaims(token(encode("null")))).toBeUndefined()
  })
})
