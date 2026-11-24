import { render } from "@testing-library/react"
import {
  QueryClient,
  QueryClientProvider,
  type QueryObserverOptions,
} from "@tanstack/react-query"
import type { ReactElement } from "react"

// queries: extra query defaults, e.g. retryDelay: 0 for a hook with its own retry rule
export function renderWithQueryClient(
  ui: ReactElement,
  { queries }: { queries?: Partial<QueryObserverOptions> } = {}
) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, ...queries },
      mutations: { retry: false },
    },
  })

  return {
    ...render(
      <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
    ),
    queryClient,
  }
}
