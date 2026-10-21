# Owned public runtime completion

Status: implemented in source `0eeba91`; coordinated native and packed browser
acceptance passes. Shared Rust rendering is pinned from `73b0988`. This document
describes the selected ordinary API scope, not universal JavaScript semantics or
release approval.

## Semantic ownership

Rust owns constructors, helper dispatch, graph building, rendering phases and
publication. Swift/C and web execute the host operations requested by owned jobs.
Prepare and resume borrow document state briefly; run releases those borrows
before resource or foreign callbacks. Finish uses current revisions after nested
operations and retains partial mutations and the original host exception.

The core interfaces remain [public models](LLD_PUBLIC_MODELS.md),
[captured constructors](LLD_PUBLIC_CONSTRUCTORS.md), and
[detached rendering](LLD_DETACHED_RENDER.md). Family adapters share existing Rust
layout and SVG algorithms rather than implementing diagram policy in hosts.

## Host identity and collection lifetime

Bounded flat Record arguments create requested own data properties without deep
host-object traversal. Owner-local retained slots preserve original style-array
identities across graph construction and later draws. Tokens are owner checked,
quota bounded and invalidated on disposal.

Calls and iterators use generic live-read operations. Iterator acquisition
captures next once; body failures close the iterator while preserving the
original exception. A failing next does not trigger close. Browser Map/Set
publication preserves unchanged ordered slots. Native coordinates retain their
explicit limitation: a final snapshot cannot reveal every deletion/reinsertion
history.

[C20 opaque access](LLD_NATIVE_OPAQUE_HOST_ACCESS.md) extends the same owned
lifecycle to Swift constructor values. Borrowed typed packets preserve UTF16,
number bits, graph references and opaque tokens as distinct forms. The callback
table is retained outside the document lock; replies are released once. Unknown
opaque IDs cannot fall through to unrelated C12 callbacks. Swift retained values
remain bounded by the owner ledger and live until owner close.

## Implemented functional corrections

- Constructed Agentflow diagnostics suspend the shared graph builder at its
  actual callback phases. Cache assignment occurs only on success; nested cache
  completion survives an outer failure. Original style identities remain retained.
- GitGraph uses retained Set/string tag iterators. State evaluates its declared
  direction getter after own-property lookup; the web admission exception is
  sealed to the authenticated native owner rather than schema text.
- Constructor reflection uses captured native capabilities and artifact realms.
  Original host failures retain identity. Browser-generated errors use the
  native-selected expression at its failure phase without replaying getters.
- Caught browser Date operations unwind native ownership before rethrowing the
  original JavaScript value. Error handling does not inspect a thrown object's
  message getter.
- Radar carries captured property-presence information into shared geometry,
  preserving sparse/empty failure phases. Mutable native presentation and image
  callbacks use the owned render lifecycle, preserving fallback and presentation
  results without holding the document guard.
- Gantt parse/today and scheduling share owned Calendar requests. Parse work
  limits precede resource execution; resumed Dates are allocated in the latest
  graph after nested writes. Calendar behavior remains Rust-owned.
- XYChart evaluates the original positive category-loop comparison. A NaN stride
  exits after the initial mark instead of repeatedly allocating labels. Typed and
  live rendering share this condition and retain output limits and recovery.

## Acceptance boundary

[The public checkpoint](parity/2026-10-21-api-completion/README.md) records 99 Swift
features, seven strict C clients, 387 browser checks, packaged TypeScript exports
and a 33/33 runtime audit. Actual AppKit UI retains its separate prior lane.

Source, SDK, WASM/package, fixture/oracle and test inputs are pinned before and
after execution; rejected and interrupted attempts remain archived. Assertions cover full SVG, dimension bits, graph state,
aliases,
callback traces, original errors, nested operations and next-call recovery.
Passing a scoped matrix does not certify arbitrary proxies, custom coercion,
modified builtins or every malformed-value combination.
