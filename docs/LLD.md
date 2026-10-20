# Library architecture

The engine implements all 33 pinned Pele diagram families. The old standalone
flowchart renderer and its v1 JSON, C, Swift and web APIs have been removed.

The active contracts are [full-library architecture](FULL_PARITY_LLD.md),
[public models](LLD_PUBLIC_MODELS.md), [prepared entries](NATIVE_DIAGRAM_ENTRIES.md)
and [caller-authored model construction](DECLARED_MODEL_CONSTRUCTION.md).
Parsing, layout and SVG stay in Rust; host resources and presentation stay in
thin Swift and WebAssembly adapters. Registries and diagram module interfaces
support extension without changing the built-ins.

SVG paint layers can use a borrowed append sink for coordinates, numbers and
escaped text. It shares the same formatting core as the existing owned-string
helpers; Sankey uses it without temporary element strings. The sink owns no
callbacks or budget policy. Measurement order, separate paint layers and final
document admission remain the renderer's responsibility.

[Direct public parsing](LLD_DIRECT_PUBLIC_PARSE.md) shares source preparation and
lets modules construct canonical public state without a temporary typed model. The [ER target](LLD_DIRECT_ER_PARSE.md)
shares the same grammar and preserves exact arena bytes through conservative fallback.
[Direct normalized projection](LLD_NORMALIZED_PROJECTION.md) removes the first
model JSON buffer and reparse for the captured Gantt and ER builtins, while preserving
legacy projection for public providers and other diagram registrations.
[Owned public construction](LLD_OWNED_PUBLIC_CONSTRUCTION.md) transfers captured
parser storage through an additive codec hook; Sankey normalizes its owned model
without reconstructing a second render model from the fresh graph.
[Source-backed canonical models](LLD_SOURCE_CANONICAL.md) derive fresh Sankey
records from indexed source, retain sparse edits and publish through a shared
logical read interface while preserving existing owned graph APIs.
[Shared construction storage](LLD_SHARED_CONSTRUCTION.md) gives ER and Sankey
one ordered sink while preserving arena construction and existing host exports.
[Captured projection](LLD_CAPTURED_PROJECTION.md) separates Sankey's eager
render model from storage through bounded reads and borrowed arena values.
[Captured methods](LLD_CAPTURED_METHODS.md) keeps Sankey helper policy in Rust
while separating scalar reads, positional writes and descriptor queries.
[Captured admission](LLD_CAPTURED_ADMISSION.md) shares structural and schema
validation over borrowed storage without changing public graph admission order.
[Shared browser graph traversal](LLD_SHARED_GRAPH_STORAGE.md) binds storage once
and shares host observation, publication and comparison without selecting a new
document backend.
[Captured Sankey ownership](LLD_CAPTURED_SANKEY_OWNER.md) retains native render
state without a generic graph and preserves shared admission, helper and render
policy behind an opt-in feature.
[Experimental captured web storage](LLD_CAPTURED_WEB_STORAGE.md) implements that
boundary with shared traversal and bounded column imports. Its initial VM result
rejects default activation; profiling and a coarser boundary remain necessary.
[Native canonical storage](LLD_NATIVE_CANONICAL_STORAGE.md) moves the generic
tables into Rust-owned POD pages, uses direct native admission and bounded host
observation, and defines pinned page leases for the shared browser publisher.

[Eager Swift projection](LLD_SWIFT_EAGER_PROJECTION.md) transfers normalized
immutable model events without a second full model JSON decode in Foundation.

[Browser host storage](LLD_HOST_ER_STORE.md) defines the unreleased ER construction,
method and live-render boundary, its bounded transfers and its verification gates.
[Ordered record publication](LLD_ER_RECORD_PUBLICATION.md) removes intermediate
field/property vectors from fresh ER records through a defaulted construction sink.
[Dense-array publication](LLD_ER_DENSE_ARRAY_PUBLICATION.md) adds a bounded generic
array event while retaining canonical index strings, budgets and native defaults.
[Source result transport](LLD_SOURCE_RESULT_PACKET.md) shares a borrowed success
packet across diagram families while preserving legacy envelopes and exports.
[Arena render outcomes](LLD_ARENA_RENDER_PACKET.md) borrow post-render graphs for
binary consumers and return browser success results directly while retaining
full graph publication and exact combined transport limits.
