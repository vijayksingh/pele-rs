# Compact publication for graph-preserving renders

Status: proposed, unimplemented transport layer. Preserve existing owned Rust
outcomes, public PELGBIN1 exchanges, source rendering, and public model behavior.
This design does not depend on the proposed borrowed-outcome API.

## Objective and measured boundary

A retained render currently returns the entire canonical graph even when the
provider only reads that graph. WASM encodes strings/nodes/properties; JavaScript
decodes them into a second wire graph and restores the caller objects. Remove
that encoding/decoding when native graph identity is proven unchanged. Keep
complete restoration, including resource-callback edits to detached objects.

The `fa9f4e8` profiles under `dist/evidence/direct-current-profiles` attribute the
following inclusive costs. These precede subsequent changes and must be refreshed
before making an adoption claim:

| Retained workload | Native full binary encoder | JS graph decoder | Sum |
| --- | ---: | ---: | ---: |
| Flowchart large | 7.53% | 10.03% | 17.56% |
| ER large | 3.49% | 7.97% | 11.46% |
| Sequence small | 5.35% | 12.31% | 17.66% |

These are opportunity ceilings, not expected speedups: structural validation,
metadata decoding, compact framing and budget checks remain. Reconciliation
(approximately 15.15%, 6.68%, 10.05% respectively) also remains. The first layer
still clones the existing owned outcome graph; its separately estimated cost
was only approximately 0.75%, 0.37%, 0.62%. This proposal does not improve cold
parse or source render, and cannot close the entire JavaScript performance gap.

## One capability, narrowly audited

Add a default-false `render_preserves_public_graph` capability to the public
module trait and erased adapter, exposed read-only by `PublicDocument`. The
promise covers normal, raw and bounded render, success and every operation-error
path: provider rendering itself does not change any canonical graph atom, string,
node, descriptor, collection ordering or object ID. Internal projection caches
may change. This is distinct from `unchanged_public_graph_is_noop`.

Initially opt in only Flowchart, ER and Sequence after auditing all three render
entry variants. Their render paths read the graph or its cached typed projection.
Do not infer capability from diagram names, schema IDs, inherited model types or
registration replacement. Swimlane and other providers need their own audit.
Custom modules default to full publication; Gantt remains full publication.

The promise excludes native changes performed through an active host-model
callback, which the adapter tracks conservatively. A provider that mutates the
canonical graph outside that channel must not opt in. No capability is inferred
from equality of typed projections or successful SVG generation.

## Separate native encoding proof from host acknowledgement

There are two independent facts:

1. The WASM document has successfully encoded a full graph at a particular
   native revision and knows its exact encoded size.
2. The facade has fully published that graph into caller objects and still has
   the corresponding immutable wire graph available for restoration.

A successful native encoding is not a host acknowledgement. Failed JS decode or
reconcile may leave fact 1 true while fact 2 is false. Never combine them in a
single flag, and never use a callback's observed `bridge.graph` as proof of fact 2.

Store a private WASM-only `EncodedGraphBaseline { revision, frame_bytes }` per
`MutableDocument`. `frame_bytes` is successful PELGBIN1 output length minus the
metadata length; it includes every fixed header/count field and all graph bytes.
It retains no graph, second baseline, hash, callback or output buffer. A `Cell`
containing this small Copy record permits seeding from `snapshot_binary(&self)`
without changing its public signature.

Every constructor starts without a baseline. Successful full binary snapshot or
operation encoding may seed it using the actual encoded byte count. JSON output
never seeds it. A failed encoding never seeds it.

Audit every mutating WASM entry point, including legacy JSON/binary apply,
invoke, render and prepared/raw variants. Clear the baseline before any mutation
attempt, even if admission or application subsequently fails. A compact-aware
render may take the old record into a local variable before clearing the stored
record. It can reuse that local proof only for this operation and only under all
conditions below. Legacy mutation methods may conservatively leave the cache
empty; successful full binary methods can reseed it through one shared helper.

This invalidation is deliberately broader than revision advancement: an error
or partially rejected callback must not accidentally preserve a stale proof.
No extra snapshots are required merely to reconstruct an invalidated cache.
The next successful full output restores eligibility.

## Conservative active-callback tracking

For a compact-aware operation create an operation-local dirty marker, initially
false. Set it before active-model synchronization or invocation can happen, not
only after a successful `apply_delta` or `invoke_builtin`. Dirty never returns
to false during the operation, even if a callback throws or produces no delta.

The current WASM adapter resolves every active host function through
`host_functions::Resources::host_function`. Marking dirty at resolution is an
adequate conservative first implementation: it occurs before `Function::call`,
`begin_sync`, initial delta extraction and all callback work. It also rejects
compact output when a function was resolved but never called. This avoids
wrapping the complete ActiveModel trait or allocating an extra Rc solely for
mutation observation. Keep this coverage explicit if that adapter changes.

Ordinary text-measurer, icon and clock callbacks may alter caller JS objects but
cannot mutate this busy native document directly. They do not dirty the native
marker. Their edits must still be restored from the captured graph. Existing
`nativeBusy`/`busy` guards, callback schedules and original exception handling
remain unchanged. A callback touching another document does not acknowledge or
invalidate the current document's native graph.

## Additive private transport

Keep all existing entry points and PELGBIN1 decoding. Add compact-aware numeric
retained-render and raw-render WASM entry points, selected only when both sides
support them. The first implementation calls the existing owned core render
methods. Do not duplicate render orchestration, introduce public lifetime-bearing
outcomes, or change C/Swift/public owned APIs.

The facade passes an `allow_unchanged` request only when its prepare phase proved
an exactly confirmed graph unchanged and did not call apply. This request is
permission to use the optimization, not proof of native state. The WASM side
also requires capability true, saved baseline revision equal to input revision,
and no active callback resolution during the operation. Otherwise it sends the
ordinary full exchange after this same operation. Never rerun rendering to obtain
a fallback graph.

Use a distinct private compact frame, for example:

| Field | Encoding |
| --- | --- |
| Magic | Eight bytes `PELGREF1` |
| Input revision | u64 little-endian |
| Output revision | u64 little-endian |
| Metadata length | u32 little-endian |
| Metadata | Existing result JSON bytes |

No graph or new result representation is present. The compact-aware decoder
accepts either this frame or an ordinary PELGBIN1 exchange. The public graph
binary decoder continues accepting only its existing format. Reject truncated,
trailing, unknown-magic, invalid-UTF8 or invalid-metadata frames, and revision
mismatches. Keep revisions as exact integers/decimal text, never JS Numbers.

A compact frame must bind the captured input revision and the expected single
render advancement. A full fallback uses existing revision handling. No
untrusted compact frame can supply or replace the restoration graph.

## Preserve every budget and operation stage

Do not compare the compact packet size alone with the binary exchange budget.
For an unchanged graph the exact hypothetical full size is:

```text
saved_baseline.frame_bytes + encoded_current_metadata.len()
```

Use checked arithmetic, the same metadata-count representability condition and
the same DEFAULT_BINARY_BUDGET/error as full encoding. This preserves admission
for graphs near the transport ceiling without re-encoding their contents. The
cached record comes only from a real successful full frame, never an estimate,
caller-supplied length or JS snapshot size.

Maintain the current order:

1. Decode options/revision and perform core admission/preflight as today.
2. Execute the same render/resources/output policy.
3. Advance revision exactly as today, including dispatched typed errors.
4. Export and schema-validate the final owned graph as today.
5. Publish deferred presentation after final validation, as today.
6. Serialize result metadata. Preserve any metadata error before binary checks.
7. Preserve the encoder's structural graph validation, then enforce the exact
   hypothetical full-frame budget. Emit compact or full bytes.

No change to callback budgets, graph work limits, UTF-16 limits, output limits,
native SVG byte bounds, raw/core error distinctions or metadata serialization.
A failed final export suppresses presentation as before. Metadata/transport
failure occurs after presentation as before and emits no successful outcome.
Do not restore a saved native baseline after such a failure. Successful compact
output advances the saved baseline revision and retains its frame_bytes value.

## Facade preparation and complete publication

Extend `prepareMutableInput` to return a local publication context containing
its wire graph, input revision and compact eligibility. Clear `confirmedGraph`
before scanning or any other fallible work, as today. Only a previously confirmed,
exact unchanged comparison with no apply can set compact eligibility. Host edits,
rejected apply recovery, unknown state and extra invocation arguments retain the
full path. A successful apply is not enough for the first compact layer: request
one full output to establish the canonical native baseline again.

Retain the returned graph in the operation's local variable through callbacks.
Existing snapshots/deltas replace wire arrays rather than mutating this captured
graph in place; preserve that invariant. Do not capture only `bridge.graph` and
look it up again after native return. Callback observations may have changed it.

After validating a compact response, publish its revision and call the ordinary
full `bridge.reconcile(capturedGraph)`. A full response uses its decoded graph.
Restore all retained nodes, including detached arrays, deleted properties,
descriptor flags, Map/Set contents/order, Date values, aliases and raw numbers.
Keep existing collection clear/reinsert behavior, including caller-held live
iterators. Empty native change is never permission to skip host restoration.

Set `confirmedGraph = true` only after complete reconciliation. Preserve original
host throws and typed operation errors after publication, exactly as today.
Decode/reconcile failure leaves acknowledgement false; never retry the native
operation silently. The next attempt falls back to normal snapshot/apply/full
publication. Disposal drops the captured transport storage under the existing
busy guards; it cannot occur while this operation is active.

## Integration and adoption evidence

Use public integration/E2E slices, not implementation unit tests:

- Compare old and compact entry points on equivalent FC/ER/Sequence documents:
  exact SVG/dimensions, graph bytes, callbacks and revisions; include raw, normal,
  bounded-error and recovery paths. Prove actual compact frames on repeat render.
- Cold snapshot seeds eligibility; a method/apply forces one full output before
  later compact renders. Failed apply, failed protocol publication and stale
  revisions must not falsely acknowledge host state.
- Resource callbacks edit ordinary properties, readonly/configurable flags,
  detached arrays, collections, Dates and aliases, then either return or throw
  original sentinel values. Compare final caller state and exception identity
  with the full lane. Include a partial reconciliation failure and recovery.
- Active Gantt/Class callbacks, native invokes within callbacks, custom providers
  and unaudited built-ins remain full publication. Exercise callback errors and
  rejected delta paths even if no successful mutation was reported.
- Check compact malformed/truncated/trailing frames and exact revision bounds;
  retain raw UTF-16, negative zero and IEEE NaN payload transport coverage.
- Use a bounded public graph/metadata fixture near the real full-frame ceiling:
  identical success/failure across full and compact output despite compact bytes
  being small. Include dispatched output-limit errors and presentation order.
- Verify old WASM with new facade falls back, and new WASM with old facade keeps
  existing full behavior. Keep all package hashes and benchmark provenance.

After correctness, profile actual release WASM and compare matched retained
workloads. Confirm encoder/decoder samples and transferred bytes decrease; count
full/compact selection separately from timing. Keep parse/source as negative
controls, inspect A/A variance, then run the existing complete release matrix.
A copying reduction is not itself evidence of lower elapsed time.
