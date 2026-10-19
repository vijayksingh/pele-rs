# Borrowed public operation outcomes

Status: proposed bounded implementation layer. No implementation or performance
claim accompanies this document. Existing owned APIs and PELGBIN1 remain intact.

## Cost and scope

`PublicDocument::render_operation_policy` currently validates a final graph with
`self.snapshot()`. For audited built-ins, `snapshot_view()` already borrows the
canonical graph, but `snapshot()` immediately clones its strings, nodes,
properties and collection entries. WASM then encodes that temporary graph and
drops it. Invoke and raw render have the same ownership seam.

Current `fa9f4e8` profiles in `dist/evidence/direct-current-profiles` attribute
inclusive native snapshot work to approximately 2.91% of Flowchart large retained
render, 1.20% of ER large, and 7.03% of Sequence small. These are upper bounds for
this layer: snapshot validation and dispatch remain. This layer removes a full
allocation/copy operation, but does not remove binary encoding, JS decoding,
string reconstruction, host restoration, or SVG metadata serialization. It
cannot by itself close the retained 4.41x median gap.

Implement one ownership primitive across invoke, core render and raw render.
Do not add a retained native baseline, delta protocol, mutation journal, cache,
new module capability, or JavaScript publication shortcut in this change.

## Additive interface

Add concrete lifetime-bearing outcome views alongside the current owned types:

```rust
pub struct InvocationOutcomeView<'doc> {
    pub revision: u64,
    pub graph: Cow<'doc, ModelGraph>,
    pub result: Result<Atom, PublicError>,
}
pub struct RenderOutcomeView<'doc> {
    pub revision: u64,
    pub graph: Cow<'doc, ModelGraph>,
    pub result: Result<RenderResult, PublicError>,
}
pub struct RawRenderOutcomeView<'doc> {
    pub revision: u64,
    pub graph: Cow<'doc, ModelGraph>,
    pub result: Result<Rendered, PublicError>,
}
```

Each view has a consuming `into_owned()` returning the corresponding existing
outcome. It moves revision/result and calls `graph.into_owned()` exactly once.
No existing owned outcome field, serialization implementation, or method
signature changes.

Add `invoke_view`, `render_raw_operation_view`, `render_operation_view`,
`render_operation_with_output_limit_view` and `render_operation_bounded_view`.
All render variants delegate to one private `render_operation_policy_view`.
Keep existing methods as thin wrappers around the matching view and
`into_owned()`. Do not duplicate the operation body between owned and borrowed
paths.

Explicitly tie the returned lifetime only to `&'doc mut self`; do not tie options,
config, settings or the host borrow to `'doc`. Those inputs need live only during
the operation. Result payloads remain owned, so metadata encoding does not need
to borrow the host. Holding a view prevents mutation/disposal of its document
through safe Rust until the view is consumed or dropped.

The existing `graph_view()` uses `borrow_graph` when available and owned export
otherwise. Use that existing seam through `snapshot_view()`. External modules
that retain the default `borrow_public_graph() -> None` get `Cow::Owned` and
preserve their export behavior, errors and number of export calls. No default
module opts into a stronger mutation guarantee.

## Exact operation ordering

Preserve the current order, not merely final output equality:

1. Reject stale revision and exhausted revision before dispatch.
2. Perform the existing preflight validation. Invoke validates its call against
   the pre-operation graph; render validates `snapshot_view()`.
3. Execute the same module operation with the same resources and callback order.
   Keep the existing distinction between outer protocol errors and a typed
   operation error inside `result`.
4. Once dispatched, advance revision exactly as today, including operations
   returning an error or output-limit failure.
5. Obtain and schema-validate the final `snapshot_view()`. Do not move this before
   the operation, skip it on operation errors, or validate only changed records.
6. Invoke validates successful return atoms/method IDs against that final view,
   preserving the existing conversion of an invalid return into operation error.
7. Core render calls `presentation.publish()` only after the validated final
   snapshot exists, exactly as today. Raw render retains its present behavior.
8. Return the view. The adapter encodes metadata and full binary graph after the
   presentation callback, as before.

A final export/schema failure remains an outer error after the native revision
and partial mutations advanced. It suppresses deferred presentation publication
as before. An encoder or metadata failure occurs after presentation publication
as before. Do not silently retry the operation after either failure.

The borrowed graph is immutable during presentation publication. It contains
owned flat values, not host references that allow Rust mutation of its vectors.
Safe Rust borrowing prevents a presentation callback from mutating this same
`PublicDocument` while its final graph is borrowed. Do not use raw pointers,
interior-mutability bypasses or unsafe code to satisfy the lifetime checker.

For old owned wrappers, the graph clone moves from just before presentation
publication to immediately after the view returns. ModelGraph cloning has no
user callbacks or recoverable errors; validation/export still precedes
publication. If preserving allocation-before-presentation is deemed an explicit
contract, keep the old owned body and share only the operation-finalization
helper instead; do not accidentally add a second export to the owned path.

## Adapter changes

Switch WASM binary operation consumers to views, starting with the numeric
retained-render lane that the facade calls:

- `pele-wasm/src/host_functions.rs`: invoke-with-functions and numeric render.
- `pele-wasm/src/render_numbers.rs`: numeric render without host functions.
- `pele-wasm/src/prepared_entries.rs`: raw built-in render.
- `pele-wasm/src/lib.rs`: direct legacy binary invoke/render wrappers, if included
  in the same ownership-only slice.

Encode against `&outcome.graph` while the view is live. Read revision from the
outcome, not by reborrowing `self.inner`. Construct independent host resources
before calling the view API; do not retain an unrelated `self` borrow through
metadata encoding. The borrowed graph never crosses the WASM boundary: the
returned `Vec<u8>` remains independently owned. WASM memory growth and host
callbacks cannot leave JS holding a Rust arena reference.

Existing JSON/owned consumers and FFI signatures may remain unchanged. FFI binary
consumers can adopt the same core seam in a subsequent adapter-only change; this
is not required to prove the first WASM layer. No web package behavior changes.

## Validation is currently repeated

There are three graph checks on the render/binary path:

- pre-operation `snapshot_view().validate_schema`, required before dispatch;
- post-operation `snapshot_view().validate_schema`, required after mutation;
- `encode_binary_exchange().validate`, a structural recheck of the final graph.

The first two validate different states and must remain. `validate_schema`
includes structural validation, so the encoder rechecks the same final state.
Keep that encoder check in the initial ownership-only implementation. Public
`Cow<ModelGraph>` fields can be modified or replaced by a caller; such a view is
not a durable proof that its current graph has been validated.

A later bounded validation refinement can use a sealed `ValidatedGraphView`
with private Cow storage, read-only access and an internal encoder entry point.
Its constructor is only the successful schema-validation path; no mutable
accessor exists. The ordinary public binary encoder continues validating
arbitrary graphs. Never add a public unchecked encoder or a boolean argument
that callers can use to bypass admission checks.

## Integration proof

Extend public feature integration, not unit tests:

- For all 33 built-ins, compare owned and view operation results, revisions and
  exact full binary snapshots on separate equivalent documents. Include repeat
  render, dispatched output-limit failure and recovery. Borrowed built-in views
  must actually be `Cow::Borrowed`; an extension using default export remains
  `Cow::Owned`.
- Exercise one method with partial mutation then typed error, and a successful
  method returning an alias/new object. Preserve graph identity and exact binary
  IEEE bits/raw UTF-16, not ordinary float equality.
- Use a public custom provider with an event-recording host: verify dispatch,
  final export, presentation publication and result order. A final invalid graph
  or failed export must advance the dispatched revision and suppress deferred
  presentation exactly as the existing owned route does. Stale calls must not
  dispatch or publish.
- Retain the all-family numeric oracle suite and the existing host-callback,
  binary-budget, presentation and public graph recovery integrations appropriate
  to the adapter slice. Browser checks exercise the actual new numeric WASM
  entry points, including a callback that edits a detached object and throws.
- If an allocation diagnostic is used, separate it from elapsed-time claims.
  Check the exact release WASM profile and matched retained matrix once; cold
  parse/source render are negative controls.

## Subsequent larger copying seam

Borrowed outcomes are a prerequisite, not an excuse to accumulate a second
owned graph. A future output delta needs an independent acknowledged native
output baseline: `bridge.graph` is unsuitable because host snapshots, rejected
apply edits and active callbacks change it. Final deltas must materialize the
complete desired graph and still restore all retained host objects after any
caller callback, including empty deltas and partial-operation errors.

Do not use current `reconcileDelta` blindly for a final outcome; it is based on
the current active callback graph. Revision/token agreement, prefix identity,
full reconstructed graph/transport budgets and failure recovery need their own
reviewed protocol. Missing acknowledgement should choose a full output without
repeating the operation. The current borrowed layer adds none of that state.
