# Typed03 rebase onto e518

Private seven-file candidate against `e518012745a2eaf12c8bb048b91074566bf2000a`. Apply only after parent authorization. `candidate.patch`, `manifest.json`, `originals/`, and `files/` contain the complete source handoff. The original `source-typed-columns/` artifacts remain unchanged.

The old patch applied without conflict to exact committed originals. Six originals are unchanged. The sole changed original, `source_store.rs`, adds e518 private `LogicalCapturedText` support. The original-to-e518 delta equals the old-candidate-to-new-candidate delta, excluding diff line numbers. Its current-extent checks, replacement-first UTF16 behavior, and original-only borrowed UTF8 remain intact. The other six candidate files are byte-identical to the previously reviewed candidate. Prepared admission changes in the other e518 files remain untouched.

Rustfmt check passed for three Rust files; Node syntax checks passed for four JavaScript files; whitespace-strict apply-check passed. One generator execution against the historical frozen ef9 PLCSRC02 WASM reproduced the candidate initializer byte-for-byte. This is historical schema/recipe generation evidence, **not typed03 runtime validation**. No build, browser, Swift, VM, or timing run occurred.

The previously reviewed typed-column semantics remain unchanged: one owned compact column copy, raw IEEE words, packet-relative padding independent of incoming byteOffset, explicit DataView little-endian reads on big-endian hosts, and old01/02/update compatibility. New built03 regeneration equality, native/public lifecycle integrations, and browser feature validation remain adoption gates. Big-endian, identity-exhaustion01 and new padding-limit boundary are not execution-proven.

The LLD remains in the original private layer; parent owns any separate LLD integration.
