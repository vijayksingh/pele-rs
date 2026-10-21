# Mutable API transport performance

Design against `7a0b79c`, 2026-10-20. Proposed layers, not a performance or
compatibility completion claim. Existing public-model contracts in
[LLD_PUBLIC_MODELS.md](LLD_PUBLIC_MODELS.md) remain authoritative. No v0.3
release until the matched public API performance gate passes.

## Problem and measured boundary

The [current profile report](benchmarks/2026-10-19-profile/README.md) attributes
20.88% of Flowchart and 28.48% of ER mutable-parse self samples to JS
`ModelBridge.reconcile`. Those samples describe the report's baseline, not a
promise about the current candidate. Source rendering has a separate native
cost floor; eliminating mutable transport cannot fix source-render performance.

The current mutable paths repeat representations and traversals:

- Parse: typed Rust model, native graph, often a second typed projection,
  exported graph clone, binary bytes, JS graph records, JS public objects.
- Retained render: scan every host descriptor, build JS graph records, size and
  encode the whole graph, decode and validate it in Rust, replace native graph
  and rebuild typed projection, clone/validate/discard a pre-render snapshot,
  render, clone/validate/encode the final graph, decode it to JS records,
  restore every host object.
- Method call adds an owned pre-call graph solely to validate receiver/arguments.

Relevant locations:

- `bindings/web/graph.js`: `ModelBridge.reconcile`, `snapshot`, `reconcileDelta`.
- `bindings/web/graph-binary.js`: `encodeGraphBinary`, `decodeGraphBinary`.
- `bindings/web/full.js`: `adoptMutable`, `invoke`, `renderRaw`, retained `render`.
- `crates/pele/src/compat/public/engine.rs`: `snapshot`, `invoke`,
  `render_raw_operation`, `render_operation_policy`, erased `Codec`.
- `crates/pele-wasm/src/{lib,host_functions}.rs`: binary snapshot/apply/outcomes.
- Flowchart `create_state` and ER `create_public`: graph construction immediately
  followed by `imported(&graph)`. Their apply and method paths also rebuild that
  projection unconditionally.

## Invariants

Public models remain ordinary caller-owned mutable JS values. No Proxy or frozen
model, no promise that callers use provided setters, no JS diagram logic.
Preserve descriptors, enumeration order, schema prototypes, aliases, cycles,
retained detached objects, methods, UTF-16 units, numeric bits, revision checks,
original host exceptions and state published before operation errors.

An uninstrumented mutable JS object requires inspection before the next native
operation. Optimizing this path cannot assume zero work for an unchanged model.
The target is one necessary observation pass, compact changed-record transport,
and publication proportional to actual changes when callbacks permit it.

Host callbacks include text width, icons, clock and host model functions. A
resource callback can mutate any retained host object even when Rust changes no
graph node. Therefore native graph equality never proves host object equality.
Keep existing callback publication/error timing; do not invent new callback-time
imports while doing a transport optimization.

All byte, code-unit, object and work ceilings remain independent checks. A small
delta does not authorize an oversized resulting graph. Plain-data descriptor
access still rejects unsupported accessors/prototype edits as before.

## Layer 1: fresh publication

Record the object count at entry to `reconcile`, before allocating missing nodes.
Only objects allocated by this call may skip stale-key deletion, Map/Set clear,
and Date's second `setTime`. Fresh Errors still require stale-key deletion:
their constructors create own properties absent from the native graph. Existing
nodes always use the restoration path.
Allocate every shell before populating references, preserving forward references
and cycles.

Fresh ordinary objects and arrays may assign default data properties instead of
calling `defineProperty`, subject to precise guards:

- `__proto__` always uses `defineProperty`.
- Array `length` always uses `defineProperty`; assignment would accept descriptor
  flags that redefining the nonconfigurable length must reject.
- Instance, Error and nondefault property descriptors retain `defineProperty`.
- The graph validator currently allows duplicate property keys. A preceding
  configurable, readonly descriptor can legally be redefined but not assigned.
  Either require every property on the fast node to have flags 7, or retain a
  fallback for keys previously populated with nondefault flags.

No changes to existing-node restoration, wire format or native state.

## Layer 2: borrow native graph for validation and transport

Add a defaulted opt-in trait method such as
`borrow_public_graph(&PublicState) -> Option<&ModelGraph>`. Existing external
modules return `None`; graph-owning built-ins return their `state.graph`.
Expose the same capability through the erased codec. A `Cow<ModelGraph>` helper
can select borrowed graph or existing owned `export_public` fallback.

Keep public owned `snapshot`, `InvocationOutcome`, and `RenderOutcome` behavior.
Introduce an internal/transport operation seam that returns revision/result
without requiring an owned graph. The transport wrapper borrows the current
post-operation graph, validates it and encodes it before releasing the borrow.
Legacy owned APIs clone through this seam when ownership is actually requested.

First remove only discarded pre-operation clones, including `invoke`'s graph
used to validate its call. Then migrate snapshot and final outcome encoding.
Preserve stale-revision rejection before dispatch, revision advancement after
dispatched failures, graph export/validation errors after mutation, and
`PresentationCapture.publish` ordering. An unavailable/failed export must not
be converted into a successful empty outcome.

Do not weaken external binary decoding or schema validation. Schema metadata
can be validated once at registration, but graph validity must follow each
untrusted import and native mutation boundary. Any trusted encoding helper must
require proof of that validation rather than accept an unchecked graph.

## Layer 3: acknowledged baseline and transactional host import

Separate three states currently conflated in `bridge.graph`:

1. `accepted`: the exact graph last acknowledged by native state, with revision.
2. `observed`: the latest descriptor/collection scan of real host objects.
3. Pending transfer: changes from accepted to observed, sent but not acknowledged.

Scanning must never advance accepted state. Existing `snapshot(..., true)`
updates `bridge.graph` before `apply`; an empty subsequent delta after a rejected
apply is therefore unsafe. Introduce explicit prepare/accept/reject transitions.
On rejection keep native revision and accepted graph unchanged. The next call
reobserves and retransmits every still-present edit, even if values did not
change since the rejected attempt.

Separate stable host object IDs/string interning from accepted native table
lengths. Rejected newly discovered objects remain associated with their host
objects; resend the entire unacknowledged suffix next time. IDs are not recycled.
On failure midway through scanning, interning work is still unacknowledged.
Budget checks cover retained intern tables as well as accepted state.

Initially this protocol can reuse full binary apply. It must prove rejected
apply recovery before adding a skip or delta optimization. An empty observation
can avoid upload only when it is equal to the accepted native state and no
pending transfer exists. Preserve externally observable revision semantics on
existing low-level APIs; the optimized private operation can explicitly carry
the required revision transition.

Add an additive bounded binary delta envelope with expected revision, accepted
string/node table lengths, optional root change, appended strings, changed
records and operation metadata. Keep PELGBIN1 full-graph APIs supported. Reuse
the existing `ModelDelta` semantics where applicable, rather than treating its
callback-only JSON envelope as a new public revision protocol.

Native apply validates the resulting graph/schema and commits replacements
transactionally. Rejected edits restore replaced records and appended table
lengths. Gantt/Class active arenas already demonstrate this pattern in
`apply_delta`, including native index invalidation; generic document deltas must
add document revision and root semantics. Family typed projection invalidation
occurs only after successful acceptance. Do not suppress a family's existing
operation errors or deferred projection errors.

## Layer 4: changed output and callback-safe restoration

Use operation-scoped native change tracking, starting with one graph-owning
family. Every graph mutation must mark its node, including partial mutation on
error. New strings/objects append; all changed nodes publish before returning
or rethrowing. Gantt/Class already have explicit journals for active callbacks,
but these cover their active arena, not every module or operation today.

Do not claim a mutation-free capability merely because an operation returned
success or its final SVG is unchanged. Audit each opted-in module. Other modules
retain full snapshot output until all their write paths participate.

Maintain a desired final host graph by applying native deltas to accepted state.
During every externally callable callback set `hostMayHaveChanged` before entry,
including callbacks that throw. Then:

- Without intervening caller execution, reconcile changed native nodes only.
- With caller execution, restore against the complete desired graph. Start with
  the existing full reconcile for correctness. A later observation pass may
  compare actual descriptors/collections and write only differing records.
- Active model callbacks exchange acknowledged host edits while Rust is
  borrowed. Update the same accepted baseline at each accepted nested exchange;
  rejected exchanges never advance it. Final resource callback restoration must
  still cover edits outside those accepted exchanges.

Host scanning/restoration remains linear when arbitrary callbacks run. It can
still avoid repeated wire encoding, BigInt/hex conversion, graph allocation and
property writes. Never compare only two native snapshots to skip this work.

## Layer 5: eliminate the JS wire-object mirror

The binary adapter currently expands numeric bits into hex strings, strings
into arrays of code units, and every atom/property into JS records, then converts
them again into public values. Replace only the private optimized lane with a
packed baseline: bounded owned bytes plus string/node offsets and identity tables.

On initial decode, validate tags/counts/references/budgets, decode strings once,
allocate shells, then walk records to populate properties directly. A two-pass
record reader permits forward references without allocating per-atom objects.
Keep default descriptor fast paths from layer 1. Retain byte storage independently
of WASM memory growth; never keep stale views into growable native memory.

Observation compares host descriptors to the packed accepted records and writes
changed records directly to a bounded growable output buffer. No full temporary
graph, no encode sizing traversal over that graph, no reconstruction of every
old UTF-16 array. Preserve numeric transport as raw 64-bit data/two u32 halves;
do not normalize NaN payloads through JSON or use float equality for bit changes.
Use exact code units for strings and original order for property/Map/Set records.

Legacy graph-object functions remain available for their existing callers and
diagnostic tests. This is a transport implementation change, not a schema or
public value change. Measure retained packed baseline memory and disposed-handle
collection as well as latency; do not replace allocation churn with unbounded
history retention.

## Family projection and native rendering follow-up

Flowchart and ER eagerly construct typed projections during public parse and
rebuild them after every apply/method. Consider a lazy cached projection,
invalidated only after accepted graph changes, then evaluated before rendering.
This removes work from parse-only callers and repeated builder sequences. Do not
seed it blindly with the parser's typed model: export/import normalizes public
semantics and may differ from original parser fields. Keep imported failure
categories and timing at the first operation that currently observes them.

Native parsing/rendering still needs separate profile-driven work for source
API parity: text/layout, SVG construction and result transport remain visible
after all mutable exchange costs disappear. No optimization here changes
geometry, text measurement ordering, number formatting or returned SVG bytes.

## Integration proof and acceptance

Use public entry operation scripts, not implementation unit tests:

- Parse all families; compare property order/descriptors/prototypes/aliases and
  SVG, including sparse arrays, readonly length, `__proto__`, cycles and Dates.
- Retain detached children/arrays, replace their parents, mutate the retained
  values, invoke a captured builtin with them and render repeatedly.
- Repeat rejected apply unchanged; then repair it. Include rejection after
  discovering new strings/objects, stale revision, schema error, budget failure
  and transport failure. Assert edits never disappear because baseline advanced.
- Callback edits title, Map/Set and array contents with no native mutation;
  callback also throws each of undefined/null/false/object/RangeError. Compare
  state restoration and original exception behavior, then recover successfully.
- Gantt/Class host override calls captured builtins, accepts and rejects nested
  edits, mutates after the final nested call, and throws after partial changes.
- Exact UTF-16 including isolated surrogates, negative zero, infinities and
  bit-preserving numeric payloads through accepted/rejected exchanges.
- Native custom module with owned-export fallback, invalid post-operation
  graph, partial failure, and revisions; existing C/Swift owned APIs remain valid.
- Repeated create/dispose and WASM memory growth preserve retained host values
  without retaining native documents through global tables.

Each layer gets one coherent commit and focused integration proof. Profile the
same public parse/render-model/source workloads before and after, retaining
package/WASM hashes and matched JS controls. Run the full performance gate after
a substantial measured improvement, not on every edit. A faster transport path
alone is not whole-library parity and never authorizes a v0.3 release.
