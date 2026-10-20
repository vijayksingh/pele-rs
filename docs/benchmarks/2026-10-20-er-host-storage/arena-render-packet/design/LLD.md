# Borrowed arena render publication and direct successful result

Private candidate; no compilation, execution or performance claim. This layer
always exports the complete post-render graph. It adds no graph-unchanged effect,
acknowledgement shortcut, lazy materialization, mutable storage cache or replay.

## Ownership and ordering

`PublicDocument` shares normal/raw operation dispatch through a private
`GraphExport` policy. Existing owned methods select `Owned`; additive view methods
select `Borrowed` and return `Cow<ModelGraph>`. Both preserve revision checks,
pre-dispatch snapshot validation, resource/error handling, revision advance and
post-dispatch schema validation. For normal rendering, `Owned` materializes the
validated graph **before** deferred `presentation.publish()`, exactly where the
old `snapshot()` did. Wrappers move the already-owned graph after publication;
they do not defer its clone. Borrowed views keep the document borrowed until the
consumer finishes transport. Extensions without `borrow_public_graph` retain
owned export fallback. No callback is executed with a new mutable graph borrow.

The C binary consumers v5/v6/v9, prepared raw, presentation and native-resource
paths select the corresponding view method. Existing document/operation guards
remain alive through serialization. C ABI, wire bytes, metadata serializer,
callback ordering and admission stay unchanged. v4 JSON stays on the owned API.
No Swift source changes or new C entrypoints.

## Optional WASM transport

Legacy exports remain byte-for-byte unchanged. New normal/raw numeric exports
share the core operation and existing resource adapters. The facade selects a
capability before dispatch and never retries a failed operation.

Error outcomes retain the old full `PELGBIN1` frame with serialized Result
metadata. Successful outcomes use `[1, metadataBytes, graphBytes, status, payload]`.
`graphBytes` is a full graph frame with empty metadata; `payload` at status 0 is
the existing shared success packet `[1, metadataBytes, result]`. The result has
identical key order, raw dimension bits and links. Both pieces are validated
before full ModelBridge reconciliation. Graph/revision publication and original
resource exception priority remain unchanged.

A result-factory exception occurs only after the committed graph is encoded and
admitted. Status 1 carries that original JS value alongside the graph; the facade
publishes the graph/revision before throwing it. This is an explicit new factory
failure protocol, not a claim of universal parity under monkey-patched intrinsics.
Arbitrary resource throws still take precedence in the existing outer catch.

## Exact byte gate

`render_packet::prepare` counts the exact legacy retained/raw Result serializer
using existing per-write Counted semantics before link encoding or result factory
calls. It preserves serializer errors and the 64 MiB metadata ceiling. Successful
encoding calls the existing binary encoder with empty metadata and budget
`64 MiB - metadataBytes`. Binary metadata length is always a fixed-width u32;
therefore the graph frame is exactly the legacy frame minus its metadata bytes.
The same graph validation runs first, then all normal counts/writes. A full
legacy frame fits exactly iff the reduced-budget empty-metadata frame fits.
An earlier private buffer-write rejection cannot trigger host callbacks or
publication. Error outcomes retain the legacy full-budget metadata path.

No large SVG JSON buffer or JSON.parse is needed on success, but exact metadata
counting still scans/escapes SVG and the full graph is encoded, decoded and
reconciled. New typed result imports and a JS-owned graph byte copy have costs.
This is an allocation/ownership hypothesis; fresh matched profiles and first-use
measurements must determine latency. Encoder validation-token optimization and
unchanged graph publication are separate proposals.

## Public validation gates (not yet executed)

* Native extension integration compares owned/viewed normal/raw results,
  exact graph frames and numeric bits, borrowed/default-owned fallback,
  JavaScript/native callback errors, output limits, revision rejection/repair,
  post-render invalid graph rejection and deferred presentation order.
* Browser differential forces the legacy capability on the identical package;
  covers 32 arena families plus nonempty Flowchart links, normal/raw output,
  exact full graph/revision/metadata counts and resource traces.
* Browser feature cases cover arbitrary original throws, full callback
  restoration, aliases/cycles/detached references, readonly/accessor edits,
  computed NaN dimensions, factory failure publication and subsequent recovery.
* A calibrated accessibility string makes metadata itself fit below 64 MiB
  while its canonical graph plus metadata exceeds 64 MiB, checking identical
  rejection categories, subsequent stale-revision failure, and fresh-document recovery in both modes. The existing transport-failure revision gap is not repaired by this layer. Calibration and size asserts
  must succeed; equality of two early fixture failures is insufficient.

Existing source/host packet envelope-overflow tests remain required consolidated
coverage; no public result/schema/model contract changes are intended. Fresh
native/Swift artifacts, C consumers and all-engine browser checks are still
required before adoption. No release-gate or performance claim here.
