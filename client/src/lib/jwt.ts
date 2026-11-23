/*
The claims in a JWT's payload, for display.  They're decoded, not verified: the portal
checks the token's signature on every request, so nothing here should be trusted.
Undefined when the token isn't a JWT with a JSON object payload.
 */
export function decodeJwtClaims(
  token: string
): Record<string, unknown> | undefined {
  const payload = token.split(".")[1]
  if (!payload) return undefined
  try {
    // base64url to base64, padded to a multiple of 4
    const base64 = payload.replace(/-/g, "+").replace(/_/g, "/")
    const padded = base64.padEnd(
      base64.length + ((4 - (base64.length % 4)) % 4),
      "="
    )
    const bytes = Uint8Array.from(atob(padded), (char) => char.charCodeAt(0))
    const claims: unknown = JSON.parse(new TextDecoder().decode(bytes))
    return claims && typeof claims === "object" && !Array.isArray(claims)
      ? (claims as Record<string, unknown>)
      : undefined
  } catch {
    return undefined
  }
}
