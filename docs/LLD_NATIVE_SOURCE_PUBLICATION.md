# Native source publication

**Proposed; not implemented or selected.** This layer follows native source
append construction. It changes only the first publication of an admitted,
authentic builtin Sankey source document. The ready native canonical epoch and
typed render projection remain present before `parse` returns.

## Evidence and limits

The `4033e41` read-session candidate uses WASM `2310934`. Its six medium/fixed
diagnostic profiles are recorded in
the [read-session evidence](benchmarks/2026-10-20-er-host-storage/native-read-sessions/).
Parse has 22.14% native flat-storage self time, 7.76% admission self time, 13.74%
JS storage-reader self time, 18.31% shared-publication self time, and 8.92% GC.
Initial native construction is 24.17% inclusive; first publication is 31.39%
inclusive. These inclusive groups overlap other categories and cannot be added.
Parser entry is 6.07% inclusive. Samples do not identify allocation counts or
cache-miss counts.

The matched VM comparisons in that evidence still place Sankey parse at
12.82–20.83 times Pele JS and 1.28–1.47 times the existing arena. Flowchart
negative controls also moved adversely. Neither this proposal nor the source
append layer promises JS parity. Even removing the entire sampled publication
phase would leave most current parse time. Actual object allocation, string
creation, alias construction, and eager ownership bookkeeping remain necessary.

Fresh `reconcileGraph` already avoids removal scans, clearing new Maps/Sets,
resetting new Dates, and comparing existing property descriptors. A second
function that merely removes those branches is not the architectural change.
The avoidable boundary is asking JS to interpret generic paged storage through
many guarded scalar/record accessors after Rust has already constructed it.

## Current complete source path

1. `full.js` performs foreign-provider routing and serializes parse options.
   `NativeRegistry::parse_with_native_storage` decodes options and uses the
   existing single captured `ParsePlan`. ER and external providers keep their
   current routes.
2. `PublicEngine::create_host_sankey` parses once, validates the captured schema,
   then calls Sankey `initial_into(NativeSourceWriter, &model)`. The writer
   produces the canonical POD epoch; it does not publish JS objects.
3. Core independently runs structural admission at the former
   `GraphWriter::finish` boundary. Ordinary source moves and normalizes the
   authentic typed model. It does not reconstruct a second typed model from
   the graph. Prepared input still uses the eager importer.
4. WASM `initial_admission` runs snapshot structural/schema admission, serializes
   the initial type metadata, and runs outbound structural admission plus exact
   canonical frame counting. These are three structural walks in total across
   core construction and the adapter, plus schema traversal and frame traversal.
   This proposal preserves every walk and their order.
5. WASM creates the owner handle and epoch lease/directory descriptor.
   `adoptCaptured` registers the schema, captures provider policy, creates and
   attaches `CapturedOwner`, then calls `owner.reconcile`.
6. First reconciliation decodes the string history, allocates all public objects
   and both identity indexes, then populates every property and collection.
   Each value is reconstructed from the native POD representation. Only then
   does the facade install document/raw-render/finalizer ownership.
7. The parse benchmark reads `model.type` and disposes the WASM document inside
   the measured operation. Pele JS has no corresponding native disposal.
   This is the established full-API benchmark contract, not parser-only timing.

There is no full JSON model roundtrip, binary graph decode, initial observation,
or graph-to-second-typed import left in the selected ordinary source lane.
Removing those again cannot explain the remaining gap.

## Chosen boundary

Add a bounded **initial publication emitter** over an admitted immutable native
epoch. It feeds the shared constructor decoder with copied batches, directly
creating the caller's final JS objects. It replaces first reconciliation only.
It does not replace the epoch, run a second parser, store an event tape, or defer
canonical construction until the first method/render.

The emitter runs **after** all step 4 gates and after the same JS registration
and owner attachment steps. Emitting JS effects inside `NativeSourceWriter`
would move factory/prototype/ownership failures ahead of admission. That is
excluded. Consequently this design retains a native sequential publication walk:
it removes the JS POD-reader traversal, not every traversal of the data.

`InitialWriter` remains the one construction interface. Its source output can
carry private fresh-source provenance in the WASM result, but that provenance
does not waive admission. No public caller can mark imported or edited storage
as fresh. The source writer and publication emitter use the same canonical IDs,
native atom encoding, and constructor stream definitions. No second canonical
table or schema-specific JS grammar is introduced.

## Batches and effect order

Reuse `constructor-storage.js` and its bounded atom decoder. Add an explicit
two-phase fresh-publication mode instead of duplicating the interpreter or
reusing ER's live-owner policy:

1. Emit every string, in table order, including exact UTF16 units and duplicate
   IDs. Last-content JS string-index semantics remain unchanged.
2. Emit allocation headers in node-ID order. Allocate all final objects before
   any property/collection population, matching current reconciliation. Register
   each object in owner IDs and registry owners at its current allocation stage.
3. Emit node bodies in node-ID order: collection contents before properties,
   preserving field order, descriptor flags, raw numeric bits, and alias IDs.
4. Emit root and finish. Set the acknowledged native epoch and confirmed state;
   install the facade document/raw-render/finalizer associations at their
   existing positions.

The decoder keeps only allocation/population high-water marks, a pending node,
and bounded string chunk state. An allocation-only event and a populate-existing
event extend the shared stream; they do not maintain a second node/property
table. Sequential population IDs and allocated-reference bounds are checked.
Current record/dense bulk primitives can be reused only when their allocation
and population behavior obeys this phase order.

Maximum batch is 64 KiB and 128 events. Large strings and nodes span batches.
Atoms use the existing compact widths; no per-property WASM import or callback.
No unbounded encoded publication buffer is retained. Exact initial frame budget
continues to be determined by native canonical frame counting, not stream size:
the private stream is an implementation transport, not a relaxed public budget.

The JS sink shares the existing scalar resolver and fresh property-write rules:
default Object/Array assignment where already permitted; `defineProperty` for
other descriptors, Instance fields, `__proto__`, and explicit array length.
Number bits, including NaN and signed zero, keep existing conversion behavior.
Map/Set insertion uses the existing intrinsics and SameValueZero behavior.
No object-literal experiment or delayed ownership index is included.

## Eligibility, errors, and lifetime

The first pilot selects only the captured authentic ordinary-source Sankey
builtin. Its builder emits Object, Array, Map, and the known Instance root.
Prepared, raw, edited, arbitrary-provider, and other-family paths keep current
publication. They are separate coverage, not implicitly certified by source
success. Existing low-level exports remain unchanged; absent new capability
selects the old path before execution, never by replay after a failure.

The initial epoch stays pinned throughout emission. Each JS callback receives
an owned bounded byte copy; no DataView into WASM memory survives sink actions.
Memory growth or reentry cannot invalidate already copied batch operands.
No native mutable store or RefCell guard is held across JS imports. Decoder
state is private to this publication, not global reusable scratch.

Native schema, graph, and frame errors occur before registration/factory effects
as today. After those gates, a sink throw stops emission immediately and keeps
the original thrown value. Do not retry reconciliation or parsing, acknowledge
partial publication, or manufacture a different model error. Partial owner,
lease, and handle cleanup must follow the existing adoption lifecycle, including
receiver routing after failure; do not silently fix unrelated lifecycle behavior.
No model escapes through facade document registration before successful finish.

The maintained guarantee concerns ordinary intrinsics and the currently supported
prototype/descriptor contracts. It does not add universal parity for replaced
allocation intrinsics. Nevertheless all existing concrete memory-growth,
prototype, failed-adoption, borrowed-helper, and arbitrary-throw controls remain
required. Two-phase allocation avoids relying on an unproven assertion that
allocation/population interleaving is universally unobservable.

## Implementation and acceptance

Expected runtime scope is a native initial emitter and narrow optional WASM
entry, shared constructor decoder mode, a fresh CapturedOwner sink, and
`adoptCaptured` capability dispatch. The native page writer is an earlier,
independent layer. No changes to grammar, normalization, helper algorithms,
observation, full subsequent restoration, or admission are needed here.

Before selection:

- Native feature integrations compare emitted IDs/strings/atoms and reconstructed
  canonical bytes against the epoch and existing arena for all 11,616 source
  cases, including complete errors. Check chunk boundaries, UTF16, descriptor
  order, NaN/signed zero, aliases, and initial byte/work limits.
- Public browser differential tests compare complete initial model graphs and
  descriptors, first unchanged/edited renders, both helpers, callback mutation
  and throw recovery, borrowed helpers, disposal, provider replacement, and
  failure during allocation/population. Exercise batch-boundary memory growth.
- Raw/prepared/provider/ER controls must demonstrate unchanged route and error
  ordering. Native/Swift public interfaces remain unchanged.
- Compare complete parse/dispose and first-use operations against both the
  current native backend and arena, with all three Sankey sizes/resources plus
  Flowchart controls. Keep adverse rows and exact package/WASM hashes.
- Fresh profile must show the initial POD reader removed, quantify new native
  encoding and constructor-decoder cost, and retain full eager ownership.

This is a measurable boundary replacement, not a claim that 31.39% is removable.
A domain-owned virtual canonical representation could avoid more native storage,
but would need complete helper/edit/history semantics without first-use migration.
That larger representation redesign is not justified as this layer.
