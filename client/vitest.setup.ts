import "@testing-library/jest-dom/vitest"

import { beforeAll, afterEach, afterAll } from "vitest"
import { server } from "./src/stubs/server"

// Start server before all tests run
beforeAll(() => server.listen({ onUnhandledRequest: "error" }))

// Reset any runtime request handlers added during individual tests (crucial for test isolation)
afterEach(() => server.resetHandlers())

// Clean up and shut down the server after all tests finish
afterAll(() => server.close())
