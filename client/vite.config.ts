/// <reference types="vitest/config" />
import path from "path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig, searchForWorkspaceRoot } from "vite"

// The tests expect this redirect uri.  Defaulting it under Vitest lets `vitest run` work
// without the REDIRECT_URI the `pnpm test` script sets.  Builds still take it from the
// environment (services/tms_portal/Dockerfile requires it as a build arg).
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
    // Under Vitest only, also allow the repo's specs/ folder, where the contract test
    // (useListInstitutions.contract.test.ts) reads the server's OpenAPI spec.  The dev
    // server keeps Vite's default and can't read files outside client/.
    ...(process.env.VITEST && {
      server: {
        fs: {
          allow: [
            searchForWorkspaceRoot(process.cwd()),
            path.resolve(import.meta.dirname, "../specs"),
          ],
        },
      },
    }),

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
