# Library architecture

The engine implements all 33 pinned Pele diagram families. The old standalone
flowchart renderer and its v1 JSON, C, Swift and web APIs have been removed.

The active contracts are [full-library architecture](FULL_PARITY_LLD.md),
[public models](LLD_PUBLIC_MODELS.md), [prepared entries](NATIVE_DIAGRAM_ENTRIES.md)
and [caller-authored model construction](DECLARED_MODEL_CONSTRUCTION.md).
Parsing, layout and SVG stay in Rust; host resources and presentation stay in
thin Swift and WebAssembly adapters. Registries and diagram module interfaces
support extension without changing the built-ins.

[Direct public parsing](LLD_DIRECT_PUBLIC_PARSE.md) shares source preparation and
lets modules construct canonical public state without a temporary typed model. The [ER target](LLD_DIRECT_ER_PARSE.md)
shares the same grammar and preserves exact arena bytes through conservative fallback.
[Direct normalized projection](LLD_NORMALIZED_PROJECTION.md) removes the first
model JSON buffer and reparse for the captured Gantt and ER builtins, while preserving
legacy projection for public providers and other diagram registrations.
[Owned public construction](LLD_OWNED_PUBLIC_CONSTRUCTION.md) transfers captured
parser storage through an additive codec hook; Sankey normalizes its owned model
without reconstructing a second render model from the fresh graph.
[Shared construction storage](LLD_SHARED_CONSTRUCTION.md) gives ER and Sankey
one ordered sink while preserving arena construction and existing host exports.
[Captured projection](LLD_CAPTURED_PROJECTION.md) separates Sankey's eager
render model from storage through bounded reads and borrowed arena values.

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
