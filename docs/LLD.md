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
lets modules construct canonical public state without a temporary typed model.
[Direct normalized projection](LLD_NORMALIZED_PROJECTION.md) removes the first
model JSON buffer and reparse for the captured Gantt builtin, while preserving
legacy projection for public providers and other diagram registrations.

[Eager Swift projection](LLD_SWIFT_EAGER_PROJECTION.md) transfers normalized
immutable model events without a second full model JSON decode in Foundation.
