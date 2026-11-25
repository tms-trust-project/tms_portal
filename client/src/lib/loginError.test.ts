import { loginErrorMessage, withoutLoginError } from "./loginError"

describe("loginErrorMessage", () => {
  test.each([
    ["institution_required", "Choose your institution to log in."],
    [
      "unapproved_institution",
      "That login isn't from an approved institution. Choose your institution and log in with your identity from it.",
    ],
    [
      "stale_authentication",
      "Your login could not be confirmed as new. Please log in again.",
    ],
    [
      "login_cancelled",
      "The login was cancelled. Choose your institution to try again.",
    ],
    [
      "login_expired",
      "Your login took too long or was started again in another tab. Please log in again.",
    ],
    ["login_failed", "The login could not be completed. Please try again."],
    [
      "login_unavailable",
      "Logging in is unavailable right now. Please try again later.",
    ],
  ])("explains %s", (code, message) => {
    expect(loginErrorMessage(`?error=${code}`)).toBe(message)
  })

  test("gives an unknown code the general message", () => {
    expect(loginErrorMessage("?error=something_new")).toBe(
      "The login could not be completed. Please try again."
    )
  })

  test("is undefined without an error", () => {
    expect(loginErrorMessage("")).toBeUndefined()
    expect(loginErrorMessage("?error=")).toBeUndefined()
    expect(loginErrorMessage("?client_id=other")).toBeUndefined()
  })
})

describe("withoutLoginError", () => {
  test("drops the error and keeps the rest", () => {
    expect(
      withoutLoginError("?error=login_failed&client_id=other&state=s")
    ).toBe("?client_id=other&state=s")
  })

  test("leaves nothing when the error was all there was", () => {
    expect(withoutLoginError("?error=login_failed")).toBe("")
  })
})
