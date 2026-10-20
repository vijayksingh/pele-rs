# Native canonical captured storage

The experimental JS-column backend passed browser correctness but failed its
[matched VM comparison](benchmarks/2026-10-20-er-host-storage/captured-backend-rejected/).
Repeated scalar imports dominate its sampled admission and adapter work. The
replacement places canonical storage in Rust-owned memory. It remains opt-in;
native correctness alone does not justify browser selection or a speedup claim.

## Primitives and ownership

`FlatEpoch` contains the sole generic canonical graph: POD headers, properties,
collection atoms, UTF16 descriptors and units. `FlatWorking` forks its 1,024-record
pages through two-level copy-on-write directories. Neither stores a ModelGraph,
JsValue, retained node/property vectors, full encoded frame or parent-epoch chain.
The eager typed Sankey projection remains the rendering representation required by
the existing captured Document contract.

`NativeSourceWriter` implements the existing `InitialWriter`. Record and dense-array
construction write columns directly. Source grammar, configuration, provider
selection, source preparation and normalization are shared with existing paths.
Only the exact concrete Sankey builtin selects this representation. Combined
source dispatch keeps ER's existing direct-parser/factory fallback policy.

`CapturedRead`, `CapturedWrite` and `AdmissionRead` access local pages. Default
scalar admission visitors preserve arena borrowed-slice behavior; flat overrides
decode stack scalars directly without per-node temporary vectors or host imports.
Existing validation passes, predicates and error order remain authoritative.

Equal writes compare before making a page private. Observations reuse prior spans,
retain pending node counts locally and commit headers only after complete node
capture. Power-of-two span growth leaves bounded slack; maintenance compacts when
allocated slots exceed `4 * live + 2048`. IDs and string history never compact or
reorder. String payload exists once; keyed hashes and exact collision comparisons
index canonical IDs. Explicit duplicate string IDs remain distinguishable.

`seal()` commits already-written storage without fallible maintenance. After a
Document revision advances, the wrapper retains sealed W before compaction,
metadata serialization, transport admission or host result construction. Errors
at those later stages cannot roll back or replay native accepted state.

## Observation and publication

`NativeObservation` is the sole physical decoder for bounded host batches:
at most 64 KiB and 128 events per batch. Strings and nodes may span batches through
explicit begin/end events. Complete string additions survive aborted observation;
incomplete string payload and node W do not. Forward IDs remain subject to the
existing semantic admission stage. Sparse argument records preserve holes without
allocating a declared dense argument length in native observation.

The private frame rejects node IDs above 1,000,000 before gap allocation. Exactly
1,000,000 remains physically representable for the normal ordered limit error;
the shared browser observer already refuses newly discovered IDs at that bound.
The public binary decoder and its contract are unchanged.

The WASM adapter must separately retain accepted N, last observed/published O,
incremental string history H, operation-local W and the facade's acknowledged
revision. It must take Document out of its owner cell before resource callbacks;
no RefCell guard may span host code. Failed observation must preserve original
thrown identity while aborting W and synchronizing completed string history.

The browser owns actual public objects and identity/decoded-string ledgers. It
must not retain another column graph. The shared observer streams batches; the
shared publisher reads initialized POD pages through an immutable pinned epoch
lease. Directory allocations and pages stay alive through outer walks, nested
calls and disposal. Every live-memory read window refreshes views when
`memory.buffer` changes. No native-memory access may resume after a
user-code-capable action without a fresh window.

The first scalar reader repeated refresh and directory/range checks for every
word. VM profiles attributed 41.86% of parse and 53.73% of retained sampled self
time to refresh and buffer getters. The revised reader copies one bounded
header/property/item/string-descriptor row into a fixed 21-word register bank.
Each accessor checks the lease and requested row; cache hits read only immutable
copied values. A miss refreshes once and validates the complete initialized row
before copying. The prior row ID is invalidated until the copy completes. UTF16
uses page-bounded copied scratch, consumed as scalar arguments before string
construction can reenter. Shared publication/matching loops, full restoration,
native admission and the WASM binary remain unchanged. This is an access-boundary
change, not an assumption that resource callbacks are pure.

The next reader layer binds optional read-session hooks when creating a native
traversal. NestedStorage retains its original function references and allocates
neither sessions nor property registers. Each native session pins the immutable
view and copies at most 128 initialized rows per table into bounded scratch;
windows stay within a native page. Logical POD scratch across the four tables is
at most 8,704 bytes, excluding objects and bounded UTF16 scratch. It is released
in `finally`, never retained as another canonical model. A miss refreshes and
validates the source range before synchronous copying; cache hits read independent
copied words across callbacks. Publication/matching copy each property's five
operands to scalar locals before any descriptor or Proxy action. Slot and record
paths share one scalar resolver. Native admission, source construction, complete
restoration and the WASM artifact remain unchanged. Performance and default-path
controls must be measured separately before selecting this layer.

## Verification gates

Public native integrations compare exact canonical bytes and errors with the
arena, source/prepared/raw operations, first helper/render, partial failures and
repair, descriptors, duplicate strings, number bits, sparse arguments, aborted
history, page boundaries, malformed frames and snapshot isolation. The complete
11,616-case frozen Sankey source corpus also exercises NativeSourceWriter.

These checks do not prove browser publication, original host exception identity,
memory growth during callbacks or Proxy traps, live lease disposal, raw config
getter ordering, or performance. The complete browser owner must pass existing
captured-backend differential tests and added memory-growth/recovery cases before
measurement. VM comparisons must verify the selected native backend outside
timers, pin source/package bytes, compare exact SVG and dimensions, and retain
adverse rows. Fresh local Swift SDK correctness remains a separate gate after
shared native changes. No v0.3 release while Pele JS performance parity is unmet.
