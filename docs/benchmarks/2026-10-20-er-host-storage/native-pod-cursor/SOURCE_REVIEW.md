# Independent source review

Approved private patch `38f7fd8a4151913ba4a2484d00d13bb5512465f4d4abb9529ab96f3feabd9a6a` (supersedes `807e58e9`). Candidate `33224aa2f52b2fc4a4ff9de2f36528f8d422ebe031e3d4831d1634650318cebe`; original/patch/candidate hashes verified against manifest and current production.

No concrete source blocker. The four row IDs plus 7/5/3/2 copied words occupy the fixed 21-word bank exactly. Every accessor checks lease-open; cached values belong to one immutable pinned epoch. Reentry can overwrite a row bank, but no scratch reference escapes, and each later scalar accessor rechecks its row ID. Growth cannot invalidate copied scalars. A miss refreshes the current buffer, validates directory/address/initialized row extent, then copies synchronously with no model callback. Bounds remain checked; partial record acceptance is not introduced.

UTF16 copies are bounded by the current page remainder and <=1024 units, use a plain scratch array, and finish the native reads before constructing strings. Spread materializes scalar arguments before a reentrant fromCharCode call; the next chunk refreshes anew. No per-record wrapper, canonical mirror, changed shared traversal, admission gate or public mutation is added. Argument views continue their existing refresh path.

Final delta only invalidates the destination row ID before copying and marks the new ID after complete success, avoiding an old-ID/partial-bank mismatch after an unexpected copy exception. This is conservative internal state hardening, not broader observable behavior.

Assumptions remain the supported ordinary-intrinsic, synchronous, immutable-epoch contract. No concurrent memory mutation/threading or monkeypatched DataView/native memory getter guarantee is claimed. Cached DataView objects are never trusted across a subsequent native read window; copied scalar rows may safely outlive callbacks. Existing memory-growth, Proxy publication, observation-throw/repair, same-owner busy and cross-owner callback E2E are the appropriate execution gates. No runtime execution or speed claim in this review.
