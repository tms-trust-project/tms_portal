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
  process.env.REDIRECT_URI ??
  (process.env.VITEST ? TEST_REDIRECT_URI : undefined)

// https://vite.dev/config/
export default defineConfig({
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
})
