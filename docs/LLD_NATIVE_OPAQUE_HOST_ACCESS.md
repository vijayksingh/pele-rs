# Native opaque host access

Status: implemented; coordinated C/Swift runtime validation pending.

The C15 owned renderer can call native helpers and C12 bound functions. Retained constructor values use a different owner ledger, so reading an opaque `options.log` previously returned undefined. This prevented constructed Agentflow diagnostic callbacks in Swift even though native graph-backed and browser consumers passed.

C20 adds an optional owner-bound callback table, without changing C15 signatures. Native-selected Get, Call and scalar operations use borrowed typed values: UTF16, raw IEEE bits, opaque tokens, current-document graph coordinates and builtin method IDs. They do not serialize the graph or enumerate options. Opaque IDs use the same nonreused allocator as C12 functions; graph coordinates have a distinct wire tag. The host ledger must reject foreign IDs. C12 fallback occurs only after an explicit unhandled Call.

The table is reference counted and cloned before dispatch. No document lock, job borrow or binding borrow spans foreign code. Reply storage is released exactly once after validation and copying, including failed replies. Arguments are bounded to one million, aggregate request and reply text to sixteen million UTF16 units, and nesting to 64. Call policies distinguish copied Value, retained opaque identity, and an uninspected Discard result.

Swift installs a weak-document bridge inside its existing detached lifetime. It reuses the same `HostOperationOwner` logic as constructor/invocation steps and keeps original errors in nested scopes. Disposal remains blocked through callback completion and publication. The existing host ledger retains at most one million identities; retained results live until owner close rather than claiming a temporary lease. Native references remain graph coordinates, without JavaScript object-lifetime guarantees after arbitrary graph replacement.

Acceptance uses the unchanged 18-case constructed Agentflow oracle consumer, the public C20 lifecycle client, and the existing C15/C17/C18/C19 clients. Compilation alone does not establish callback parity or release readiness.
