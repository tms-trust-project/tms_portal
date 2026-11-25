declare const __REDIRECT_URI__: string

interface ImportMetaEnv {
  // "true" to require an institution for every login (see src/lib/loginOptions.ts)
  readonly VITE_REQUIRE_INSTITUTION?: string
}
