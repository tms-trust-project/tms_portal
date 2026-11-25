/*
Build with VITE_REQUIRE_INSTITUTION=true to make every login go through an institution
from /institutions.  The sign in card's Log In then stays disabled until the list has
loaded (and for good if it fails or is empty), and the plain Globus login is never
offered, there or in the user menu.  Off by default.
 */
export const REQUIRE_INSTITUTION =
  import.meta.env.VITE_REQUIRE_INSTITUTION === "true"
