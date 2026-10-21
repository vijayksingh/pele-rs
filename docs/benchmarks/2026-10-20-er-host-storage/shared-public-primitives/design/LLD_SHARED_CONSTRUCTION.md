# Shared public model construction

`compat/public/initial_store.rs` owns the storage-only `InitialWriter` interface.
ER and Sankey use it to emit ordered construction events. `GraphWriter` remains
the arena implementation; no new browser backend is selected by this layer.

The sink accepts borrowed strings for immediate interning and owned `Node`
values with complete properties, collection entries and explicit prototype IDs.
Its associated output allows a future host sink to publish objects without
requiring a native `ModelGraph`. Parsing, diagram methods and rendering remain
outside this storage interface.

## Ownership and admission

The builder owns the sink and borrows its typed model. A successful builder
consumes the sink once through `finish(root)`. Each fallible event propagates its
first error immediately; it does not replace that error by calling `abort`.
Adapters needing external rollback must arrange that lifecycle themselves.
Existing explicit `abort` users keep their original contract.

The arena adapter wraps existing insertions and validates the completed graph
at `GraphWriter::finish`. Property descriptor defaults, record and dense-array
defaults, limits and canonical IDs are unchanged. The existing hidden
`compat::host_er::InitialWriter` export remains available to the separate WASM
crate and downstream consumers.

Sankey's `initial_graph` delegates to `initial_into<W: InitialWriter>`. It retains
node and link ID vectors, first-match `(index, id)` endpoint aliases, lazy
fallback-node creation, ordered Map entries and the original string-interner
sequence. It does not add a second atom vector or change record layouts.
The shared adapter already uses borrowed short-string lookup; any resulting
allocation reduction does not mean the native graph has been removed.

## Verification and next boundary

The public Sankey integration covers all 11,616 frozen source cases. Its graph
frames and serialized errors also match a SHA256 captured from `9ae8142` before
this refactor, independently of the two current routes sharing one builder.
Existing ER integration consumers exercise the preserved trait path and defaults.

The complete browser Sankey migration requires captured flat storage, eager
native projection, both helper methods, edits, callback restoration, prepared
and raw adoption, disposal and exact frame admission. Its writes must remain
private until publication: ER's live-write lifecycle cannot be substituted.
No host selection or first-use conversion is introduced by this foundation.
