# Experimental captured web storage

This opt-in Sankey backend is an architecture experiment, not an accepted
performance optimization. Its first independent VM comparison rejects default
activation: compared with the arena in identical package/WASM bytes, parse takes
about 5.3–6.5 times as long and retained render about 3.7–4.0 times as long. The
direct source-render negative control changes little. Native graph removal alone
does not establish a useful host boundary. The selector stays disabled.

## Interfaces and ownership

`captured-graph.js` stores canonical graph IDs, raw floating-point words, UTF-16
strings, descriptor flags and historical slots in paged columns. Forks use
copy-on-write ownership. `captured-store.js` exposes bounded operation-scoped reads
and writes. The WASM `captured_store.rs` adapter implements the same native
`CapturedRead`, `CapturedWrite` and `AdmissionRead` interfaces used by the arena.
It retains bounded scratch windows, not a native generic graph.

Source and helper append events use one Rust `ConstructionEmitter` and one JS
wire decoder. Fresh construction owns an ephemeral native interner; appending to
an existing working epoch supplies assigned IDs and queries the existing first
string ID, without copying that interner. Bulk record/dense sink primitives keep
live ER construction local and avoid a per-record wrapper or per-field sink call.
These storage primitives contain no diagram grammar, helper policy or layout.

`CapturedOwner` reuses the shared observer, publisher and comparer. Accepted
storage, observed input and private helper working state are distinct epochs.
Failed observation preserves discovered identity/string history without accepting
its graph. Rejected apply leaves native revision unchanged. A dispatched helper or
render can advance native revision before a later transport failure; the owner
records that private state without acknowledging the facade revision or replaying
the operation. Callback publication restores the complete accepted graph. Caller
objects discovered by observation do not become registry-owned native shells.

The native document owns captured policy/configuration, revision and eager render
projection. Its WASM wrapper takes the document out of its `RefCell` before
imports and restores it on every exit; no interior borrow spans a callback.
Original host throws use the existing arbitrary-value lane and keep resource
callback priority. Initial raw observation/frame admission remains before config
effects, with independent native validation and the combined metadata/frame gate
before publication.

## Remaining cost and verification

Fresh source still constructs columns and then walks them through the shared
shell publisher. It has not eliminated that initial publication walk. Unchanged
apply checks, complete restoration, native admission and frame counting still
scan storage. Bounded per-node imports and typed-array scratch copies are a
profiling target; a canonical native store with bulk host access is a possible
replacement, not a measured result.

Build the experiment with
`PELE_WASM_CAPTURED_SANKEY=1 scripts/build-wasm.sh`. Tests opt in through the
internal owner module before constructing their candidate renderer; a renderer
created earlier stays on the control path. `PELE_CAPTURED_SANKEY=1` enables the
feature tests. The warm and first-use VM comparators accept
`--candidate-captured-sankey` and verify selected backends outside timed work.
No public loader option or default capability enables the experiment.

Initial source/prepared/raw, helper, callback, disposal and ER-routing checks pass
across Chromium, Firefox and WebKit. Complete combined-boundary, provider,
first-use and final release verification remain separate gates. No v0.3 release
can proceed while the Pele JavaScript performance gate remains unmet.
