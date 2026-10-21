# Owned parsed models for public construction

The public engine can transfer a captured parser's owned model into its public
codec. Sankey uses this additive interface to avoid rebuilding a second typed
render model from the graph it has just constructed. Other codecs retain their
existing construction hooks by default.

This changes public-document construction, not immutable model projection or
source rendering. The public identity graph remains authoritative for edits,
methods, snapshots and host publication. Parsing, graph semantics and rendering
remain in Rust.

## Cost and evidence

Many public codecs build a canonical graph from a typed parser result, then
immediately import that graph into another typed model or render cache. During
construction, the original typed model, graph and new render representation can
coexist. Reconstruction repeats string decoding, property lookup, collection
traversal and allocation before the original model is dropped.

Medium public-parse WASM profiles from the accepted source-result-packet package
show these weighted inclusive scopes:

| Family | Import/projection | `create_public` |
| --- | ---: | ---: |
| Sankey | 8.68% | 24.20% |
| Sequence | 5.26% | 42.30% |
| Railroad | 13.86% | 25.01% |

These are saved 250µs CDP profiles targeting 1,500ms per workload, with WASM
SHA256 `0cc191fa6f7e12d3a81fbe956ee3a60a5ed7db758ea1340fe11e47d8181e3ba2`
(`source-result-packet-first/profile/*-medium-parse-wasm.cpuprofile`).
Each sample is counted once within a scope. Import/projection is contained in
construction; the columns overlap and must not be added. Sampling and inlining
limit attribution. These baseline shares motivate the architecture but predict
neither latency savings nor allocation reductions for a candidate.

## Additive ownership interface

`PublicDiagramModule::create_public_parsed` accepts `ParsedPublicModel<Self::Model>`.
Only the paired erased `Codec<D>` constructs that carrier. It contains:

- the owned model;
- private concrete creator `TypeId`;
- the prepared-input flag;
- the exact optional `EdgeLimit`.

The carrier exposes read-only inspection, `is_from_module::<D>()`, a consuming
model accessor and `create_legacy`. It has no public constructor, cloning,
deserialization, mutable-model accessor or operation that replaces its model
while retaining provenance.

The default hook calls `create_legacy`, which preserves the existing dispatch:
prepared input calls `create_public_prepared`; otherwise an explicit edge
policy calls `create_public_with_edge_limit`; otherwise it calls `create_public`.
The policy's presence and numeric bits are preserved. Existing providers need
not implement the new hook.

`create_document_registered` validates the same captured schema view before
consuming the parsed document. `ParsedDocument::into_public_state` transfers the
boxed model to the paired codec and keeps the captured core registration and
configuration alive throughout construction. On success, configuration moves
into `PublicDocument`; on error, unpublished ownership is dropped. The public
document retains its registration and revision-zero lifecycle. There is no
second preparation, detection or parse, and no new registration allocation.
An `Arc` strong-count or internal storage-address guarantee is not introduced.

`PublicDocument::from_module` continues to borrow an arbitrary typed model.
Caller-graph construction, raw entry behavior and immutable `ParsedDocument`
rendering keep their existing interfaces.

## Provenance is concrete, not nominal

A matching model type, diagram name or schema does not prove which parser
produced the data. An extension may return `SankeyModel` and forward its carrier
to Sankey's implementation. The private creator identity remains the extension's
concrete type, so Sankey rejects that fast path and dispatches the carrier through
its legacy prepared/edge/default policy.

Registering the actual `Sankey` type executes the audited parser and qualifies.
A wrapper with the same model, descriptor and schema does not. Prepared Sankey
input currently takes the legacy branch even with genuine Sankey provenance.
No assumption about arbitrary caller-authored model contents follows from this
interface.

## Sankey normalization

Both construction paths use the same `initial_graph` helper. Its graph-building
statements and `GraphWriter::finish` admission are unchanged: node/string IDs,
property order and flags, lookup entries, endpoint aliases and number payloads
remain identical.

The ordinary Sankey parser guarantees unique labels in encounter order,
`nodes[i].index == i`, dense collections, and endpoint records cloned from those
nodes. Graph construction restores endpoint aliases using `(index, id)`.
Successful graph admission bounds these source indices to an exactly
representable `f64` range. Strings originating in Rust are well formed. Therefore
fresh graph import would reproduce the same node labels, indices, collection
order, title and accessibility values.

The owned path performs the remaining normalization eagerly:

1. Build and admit the complete graph first.
2. Preserve the importer's ordered, saturating charge of twice each source node's
   UTF-8 label length. Crossing 128 MiB stores the same limit error,
   `Sankey imported text exceeds native allocation budget`.
3. On success, move the model into the public state. Replace both endpoint ID
   strings with empty strings using `mem::take`, dropping their allocations.
   Rendering reads endpoint indices rather than those endpoint labels.

Link values retain exact `f64` bits. Invalid/nonfinite value filtering remains in
layout. Empty optional strings remain present. No new `maxEdges` enforcement is
added to Sankey, whose existing parser does not enforce that option.

The importer returns a `Result` stored inside `SankeyPublicState`. The owned path
preserves that distinction: graph-admission failure fails construction; a
projection-admission error remains latent until the existing render boundary.
It is neither promoted to a parse error nor postponed to a first-use conversion.
The source-only 128 MiB branch is preserved by equivalent accounting; no newly
reachable source fixture for that branch is claimed.

For N distinct source nodes, reconstruction previously created two typed vectors,
two temporary reference vectors, two array-index maps, a label map, an endpoint
index map and two owned label strings per node. Ownership transfer avoids those
reconstruction allocations. The parser's endpoint-string copies have already
been allocated; the new path releases them after graph construction rather than
claiming they never existed. Measured allocation and peak-memory reports remain
the evidence for actual savings.

## Edits and extension boundaries

`apply_public`, `invoke_public` and `create_public_from_graph` continue to import
arbitrary graphs. Their behavior includes detached endpoints, repeated node
references, stale indices, array holes, nonstring values, lone surrogates and
repeated-label expansion limits. The initial parsed model cannot replace that
validation after a caller edits the graph.

First unchanged rendering starts with ready render state. Edited rendering and
helpers use the existing graph-refresh paths. Provider replacement or engine
drop does not retarget an already-created document. Configuration, public object
identity, descriptor semantics, error categories, revisions and partial method
mutations remain governed by the existing public-document contracts.

Other families require independent proofs before opting in. Sequence's projection
normalizes actor boxes, shared identity, lifecycle maps and message fields.
Railroad reconstructs a reachable DAG in a different index order, removes
unreachable nodes and normalizes metadata. Neither may simply retain its original
typed model. Where provenance does not discharge importer checks, a shared
validation traversal must preserve those checks without duplicating their policy.

## Validation

The public integration slice is
`crates/pele/tests/public_sankey_owned_parse.rs`: full 11,616-source corpus
graph/error comparison against a legacy wrapper; initial rendering, edits,
helpers and registration replacement; same-model delegated-provider isolation;
default policy/configuration/error dispatch; and source-limit/prepared controls.
Existing `public_sankey` coverage supplies arbitrary graph, detached alias,
partial method failure and repeated-label limit/recovery cases.

Corpus snapshot equality is not a full-corpus rendering claim. Compilation,
native integration, browser publication, Swift consumers and performance are
separate validation stages. Candidate acceptance requires matched public-parse
allocation/peak and first-use measurements, including unchanged/edited rendering
and helpers, with source/immutable and non-opted-family controls. Preserve
adverse results. This interface alone establishes no browser speedup or release
parity claim.
