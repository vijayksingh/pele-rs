# Borrowed arena render outcomes and result transport

This layer removes an owned graph snapshot from binary render consumers and
adds direct successful-result transport for browser arena documents. Parsing,
model methods, layout, SVG generation and render orchestration remain in Rust.
It covers retained and raw rendering; source rendering uses the separate
[source result transport](LLD_SOURCE_RESULT_PACKET.md).

Every arena operation still exports, validates, transports and reconciles its
complete post-render graph. There is no unchanged-graph acknowledgment, omitted
restoration, lazy model conversion or operation replay.

## Core ownership boundary

`PublicDocument` exposes additive `RenderOutcomeView<'a>` and
`RawRenderOutcomeView<'a>` results. Each contains the advanced revision, an owned
render result or operation error, and `Cow<'a, ModelGraph>`.

The shared operation implementation selects a private `GraphExport` policy:

- Existing owned Rust methods select `Owned`. The graph becomes owned at the
  original post-render snapshot stage, **before** deferred presentation
  publication. Their wrappers subsequently move that graph into the existing
  outcome type.
- The additive view methods select `Borrowed`. A module implementing
  `borrow_public_graph` supplies a shared graph view tied to the document
  borrow. Extensions using only owned export retain that export, its work and
  its errors.

The view methods are `render_operation_view`,
`render_operation_with_output_limit_view`, `render_operation_bounded_view` and
`render_raw_operation_view`. They share dispatch and policy with their owned
counterparts.

Normal rendering retains this order:

1. Check expected revision and revision exhaustion.
2. Validate the pre-render graph against the captured registration.
3. Execute existing render orchestration and resource callbacks.
4. Advance revision, including dispatched operation errors.
5. Export and validate the post-render graph.
6. Materialize owned storage here when the caller selected `Owned`.
7. Publish deferred presentation.
8. Return the graph and render result for transport.

Raw rendering retains its low-level config/settings and error policy, without
adding normal render orchestration or presentation publication.

The document cannot mutate through the same borrow while a borrowed outcome is
being consumed. The public `Cow` field is not an opaque validation certificate:
the binary encoder continues validating the graph it receives.

## C and Swift binary consumers

The existing C binary render paths select view methods: v5/v6/v9, prepared raw
v13, presentation v11 and native resources v14. Their document operation guards
remain alive through metadata serialization and binary encoding.

C signatures, ownership rules, PELGBIN1 bytes, metadata serializers and resource
callback order remain unchanged. The v4 JSON operation stays on the owned Rust
method. No new C entrypoint or Swift source API is introduced.

These consumers avoid the additional owned graph snapshot when the module
supports borrowing. They still serialize their existing result metadata; the
direct JS packet described below is a browser transport.

## Optional browser endpoints

The additive WASM exports are
`render_numbers_operation_packet_with_functions` and
`render_raw_operation_packet_with_functions`. Existing binary exports remain
available with their established behavior.

`full.js` selects the optional capability before dispatch. A package lacking
the endpoint uses the existing binary method. Failure after dispatch never
causes a second call through the fallback endpoint.

Native operation errors retain the legacy complete PELGBIN1 exchange containing
serialized `Result` metadata. Successful operations return this private packet:

```text
[1, metadataBytes, graphBytes, status, payload]
```

`graphBytes` contains the complete graph and revision in a PELGBIN1 frame with
empty metadata. At status `0`, `payload` is the shared success result packet:

```text
[1, metadataBytes, renderedValue]
```

The result factory preserves property order, the normal result's diagram type,
raw result shape, links and exact width/height bits. It reconstructs dimensions
from separate u32 words, including signed zero and NaN payloads. SVG crosses as
a string rather than a large JSON-escaped metadata buffer. Links retain their
existing JSON transport.

Before publication, the facade validates packet shape, count, combined size,
empty graph metadata, next revision and result fields, and fully decodes the
graph. It then advances the host revision, reconciles all retained objects and
acknowledges the published baseline.

This full reconciliation restores callback edits to ordinary JS objects,
including detached history, aliases, cycles, descriptors and collection
contents. A successful render is not permission to skip that work.

## Exact combined admission

`render_packet::prepare` traverses the exact legacy retained/raw `Result`
serializer with the existing counted writer. Its per-write 64 MiB ceiling,
escaping, special-number metadata and failure category remain the same.
Counting completes before separate link encoding and result-factory calls.
It does not use the source-render v2 envelope as a substitute.

For metadata size `M`, successful graph encoding uses:

```text
encode_binary_exchange(revision, graph, empty_metadata, 64 MiB - M)
```

The metadata-length header is always four bytes. Replacing its value with zero
therefore leaves header width unchanged. If `B` is the empty-metadata graph
frame size, the old frame fits exactly when `B + M <= 64 MiB`; this is the new
reduced-budget condition.

The existing binary encoder still validates the graph before writing, performs
normal count conversions and enforces byte bounds. Its internal write rejection
can occur earlier under the reduced budget, with the same error and no
intervening callbacks or publication. Error outcomes retain their original
full-budget graph-plus-metadata encoding.

The metadata count still scans and escapes the SVG. The full graph still crosses
the boundary and is decoded. Removing these allocations does not by itself
establish a latency improvement.

## Failures and publication

Original resource exceptions keep the facade's existing priority. When a native
operation returns a graph-bearing error outcome, full graph publication occurs
before that exception or operation error is delivered.

A result-factory throw occurs after the native operation has committed and its
graph has been admitted. Status `1` carries that original JS value as `payload`
alongside the graph. The facade publishes graph/revision before throwing the
value, preserving object, symbol and `undefined` exception identity. This is an
explicit failure protocol for the new factory, not a guarantee of universal
equivalence under replaced JavaScript intrinsics.

There is an existing limitation when final transport encoding fails: native
revision has advanced, but no graph frame reaches the facade. The host revision
therefore remains stale. A subsequent operation on that same document can fail
with `Stale public model revision`, even after the caller repairs its fields.
This layer preserves that behavior. The public feature test checks the stale
retry and separately demonstrates recovery with a fresh document. It does not
claim to repair the affected document automatically.

## Validation boundaries

`crates/pele/tests/public_render_outcome_view.rs` exercises an extension through
public owned/view APIs: borrowed and default-owned exports, normal/raw results,
exact graph frames and dimension bits, partial errors, callback errors, output
limits, invalid post-state, revision rejection/repair and presentation order.

`tests/web/arena-render-packet.spec.js` compares optional packet and legacy
capabilities on the same package. It covers 32 arena families plus a nonempty
link case in both normal and raw lanes; callback restoration, retained detached
objects, arbitrary throws, malformed edits, nonfinite dimensions, result-factory
failure and recovery; and a calibrated combined graph/metadata overflow.

The overflow fixture keeps metadata alone below 64 MiB while the combined frame
exceeds it. It accounts for canonical string history and checks callback activity
before the failed transport, identical error categories, stale retry and fresh
document recovery.

These tests complement the pinned public parity suite, C consumers and fresh
native/Swift/WASM artifacts. Their execution results and matched performance
measurements belong to the corresponding validation evidence. This document
defines the contract and does not assert a speedup or release readiness.
