# ER dense-array publication

Unreleased construction-only storage primitive following [record publication](LLD_ER_RECORD_PUBLICATION.md). Parsing, field evaluation, renderer callbacks, mutable/caller-owned arrays and native/Swift ownership retain their existing contracts. Runtime validation and performance measurements remain separate gates.

## Interface and caller order

The doc-hidden `InitialWriter` gains a defaulted, object-safe method:

```rust
fn dense_array(&mut self, values: &[Atom]) -> Result<u32, PeleError>;
```

Its default enumerates the slice, calls `property(index.to_string(), value.clone())` in order, then calls `node(Node::Array)` with the property count as its length. Existing GraphWriter, counting admission and external writer implementations retain their property overrides, canonical table order, work charges and final error precedence. Atom clones copy scalar tags, IDs or number bits; no second value Vec is created. This method adds no required implementation and does not make the existing dynamic writer interface less object-safe.

Private `arr` accepts `AsRef<[Atom]>`. Every existing caller already supplies a Vec or empty array: text/key/style lists, entity attributes, relationships and subgraphs. All values and child nodes are materialized before decimal index keys, exactly as before. In particular, `texts` must not become a lazy string iterator that interleaves value and index interning.

The caller Vec now remains alive through the default `node()` call instead of releasing its buffer immediately beforehand. There is no additional Vec allocation, but the buffer lifetime changes; native peak-memory equivalence is not assumed. Native and Swift use the default path, without a second typed model or a specialized accounting implementation.

## Bounded host event

Only HostConstruction specializes the method. It first counts existing Atom wire sizes with checked addition, stopping as soon as the optional event no longer fits. Eligible arrays encode tag 12:

| Field | Encoding |
| --- | --- |
| Event tag | u8, value 12 |
| Node ID | u32 little endian |
| Dense length | u32 little endian |
| Values | length compact atoms, each 1, 5 or 9 bytes |

Event size is `9 + sum(atom wire sizes)`, limited to 65,534 bytes so the enclosing two-byte event count fits the existing 65,536-byte frame. The 128-event bound and owned Uint8Array transfer remain unchanged. Fit also proves the count fits u32. No extra scratch/value vector or persistent array layout is allocated.

Ineligible arrays use the exact generic property/node path, including existing bounded fragmented publication. This is optional specialization, not a public input limit; the generic length cast, table admission and failure order remain unchanged. The fit check reads only already-materialized Atom tags and cannot admit strings or nodes early.

For an eligible array, Rust still interns **every decimal index string in order**, after all values and before the existing node-count guard. Their IDs are omitted only from this event. Canonical strings, including entries no longer referenced by an explicit key ID on the wire, remain necessary for later IDs, UTF16 counts, table limits and snapshot/history accounting. This layer adds no maximum-index cache and skips no interner calls.

JS rejects pending node/string construction, an unexpected node ID or an impossible minimum payload before allocating. It calls the same eager `allocate(1)`, registering both ID and registry ownership immediately. Each ordinary value uses indexed assignment; each NaN uses the existing Reflect.defineProperty path with enumerable/writable/configurable true. The existing decoder preserves raw number bits and signed zero. Final Reflect.set of array length and its rejection result remain; the length-grew flag still requests the existing construction audit. Empty arrays follow the same path.

## Canonical accounting and lifecycle

An array still costs `9 + 5*N + sum(atom wire sizes)` canonical bytes and N references. The omitted five key/flag bytes per element count against the same graph budgets. All string table bytes and initial metadata still count. JS stores nodeBytes/nodeRefs and aggregate totals after successful publication. Final construction admission and fallback error precedence are unchanged.

The compact path avoids a Property Vec and Node wrapper in the host publisher, repeated key/flag wire encoding, the JS constructionNode object and per-index previous-descriptor reads. It retains the final public Array and all existing ownership registrations. Sparse arrays, arbitrary descriptors/keys, mutable arrays, MethodStore arrays and caller-authored roots continue through their existing paths. No JS diagram fields, grammar or layout logic is introduced.

No additional state survives a batch: the event is complete, and all existing constructor reset/abort/disposal paths remain sufficient. Large arrays retain existing incremental state. This layer does not adopt the rejected private Map-ID experiment; registry and object-ID weak maps remain as before.

## Ordinary-intrinsic boundary

The specialization uses the same supported pure-construction contract as fresh records. For a fresh ordinary array with unique consecutive indices, there is no prior own descriptor to reconcile. Changed Array/Object intrinsics are outside that contract. An inherited numeric Array.prototype setter could mutate a later index while an earlier assignment runs, invalidating the omitted previous-descriptor accounting; this change does not claim equivalence for that case. Replacing all assignments with defineProperty would also change setter behavior and is not an accepted workaround. No universal Proxy/getter guarantee follows.

## Validation and measurement gates

Public `er-record-layout.spec.js` already covers complete small/empty-array graphs through ordinary/prepared providers, direct grammar, typed fallback, retained helpers and rendering. The bounded `er-oversized-public-construction.spec.js` retains the 7,000-entity case and 7,000-relationship case, and adds 13,106 relationships without rendering a giant diagram. Relationship atoms are refs: the 7,000 array fits at 35,009 event bytes; the added array exceeds the event ceiling at 65,539 bytes. All compare pinned JS complete graph order/descriptors, aliases/cycles, first/last membership, errors/numbers and retained helper identities. The test writes its JSON artifact before assertions.

Existing retained NaN/-0 extras exercise general public behavior, not direct numeric construction: ordinary ER source produces string/ref arrays. Raw NaN publication remains source-reviewed; do not claim an executed constructor-NaN case. No private protocol or unit tests substitute for public feature validation.

Before adoption: native complete public-graph/error corpus and fallback/prepared integrations; fresh Swift SDK and consumer evidence for the default path; focused three-engine browser feature checks; and a frozen default-build package for independent VM parse/first-use timings with retained/source and non-ER negative controls. Keep changed source hashes, build logs and the complete package map. The exact initial raw zero-versus-13-metadata-byte threshold remains an acknowledged unexecuted boundary; this layer does not close it.

A preceding profile's generic property self time and source-derived array counts identify a target, not an expected gain. Empty arrays save transient construction state without per-index work. Report measured latency/allocation results separately, including adverse controls; no benefit is established by the architecture alone.
