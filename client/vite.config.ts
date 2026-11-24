/// <reference types="vitest/config" />
import path from "path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

// The tests expect this redirect uri.  Defaulting it under Vitest lets `vitest run` work
// without the REDIRECT_URI the `pnpm test` script sets.  Builds still take it from the
// environment (deploy/Dockerfile passes it as a build arg).
const TEST_REDIRECT_URI = "https://tms-portal.savanna.tacc.cloud/"
const redirectUri =
  process.env.REDIRECT_URI ||
  (process.env.VITEST ? TEST_REDIRECT_URI : undefined)

// A bundle without a redirect uri sends redirect_uri=undefined to /login, which rejects
// every login, so a build without a usable one stops here instead of shipping.
function checkRedirectUri(uri: string | undefined) {
  if (!uri) {
    throw new Error(
      "REDIRECT_URI must be set for a build, e.g. REDIRECT_URI=http://localhost:8080/ pnpm build"
    )
  }
  if (!URL.canParse(uri)) {
    throw new Error(`REDIRECT_URI must be an absolute URL, got "${uri}"`)
  }
}

// https://vite.dev/config/
export default defineConfig(({ command }) => {
  if (command === "build") checkRedirectUri(redirectUri)
  return {
    plugins: [react(), tailwindcss()],
    define: {
      __REDIRECT_URI__: JSON.stringify(redirectUri),
    },
    resolve: {
      alias: {
        "@": path.resolve(import.meta.dirname, "./src"),
      },
    },

    test: {
      globals: true,
      environment: "jsdom",
      setupFiles: ["./vitest.setup.ts"],
      coverage: {
        exclude: ["src/components/ui/**", "src/stubs/**", "src/lib/utils.ts"],
        thresholds: {
          statements: 80,
          branches: 85,
          functions: 80,
          lines: 80,
        },
      },
    },
  }
})
