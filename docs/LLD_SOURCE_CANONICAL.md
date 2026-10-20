# Source-backed canonical models

Fresh mutable parsing previously built an indexed render model and a generic
`ModelGraph`, encoded that graph, decoded a second generic graph in JavaScript,
then created the caller's objects. The source-backed owner removes these
intermediate graph representations together. The public objects, authoritative
validation and editable-model behavior remain required work.

## Ownership and extension boundary

`PublicGraphRead` lends bounded property/collection batches and UTF-16 string
chunks. `GraphReadView` holds either this logical reader or the existing borrowed
or owned arena. Admission, canonical binary encoding and canonical JSON use the
same read boundary. Existing providers default to their arena export; they need
no new methods or representation changes.

Sankey is the first source-backed owner. Its immutable `IndexedSankey` owns labels
once and stores link endpoints as indexes. A catalog preserves the exact original
first-intern string IDs. Record IDs, fixed field IDs and source array/Map entries
derive from this data. Fixed field IDs use direct slots, not repeated string-map
queries. Actual edits own sparse properties, collection slots or replacement
records. Appended and detached historical records remain addressable.

Existing captured read/write interfaces run the same native helper policy over
this owner. Helpers eagerly refresh the render projection after success and
partial failure. Accepted caller graphs retain only differences from the source;
invalid render projections remain latent errors with the same repair behavior.
First helper and first render never build a complete `ModelGraph`.

Explicit APIs returning `ModelGraph` or `Cow<ModelGraph>` still materialize that
representation at their original export stage. Additive `snapshot_read`,
`invoke_read` and render read outcomes let binary/JSON adapters avoid that copy.
Old owned outcomes keep their original public types and presentation ordering.

## Browser publication and recovery

A side-effect-free capability probe precedes source publication. Legacy
providers return false without exporting twice. A capable document emits an owned
`PLCSRC01` recipe: complete current strings, immutable scalar columns, declarative
record layouts and complete accepted replacements. Numeric payloads retain raw
IEEE words. The generic JS reader evaluates layouts; grammar, helper policy and
rendering remain in Rust.

The existing `ModelBridge` retains registry identities, busy/disposed state and
callback dispatch. Its storage binding selects logical traversal once. Capture
stores changed records and uses existing native apply/ModelDelta ingress. Every
final outcome restores the complete accepted graph, including historical
objects. A failed observation or publication never confirms a new baseline.
An accepted revision is not rolled back because transport or a result factory
throws. The original factory value travels with the accepted graph and is thrown
after publication.

## Bounds and compatibility

Canonical structural/schema admission and the full logical `PELGBIN1` frame
budget run before recipe publication. Compact size cannot weaken those gates.
Recipes have an independent physical bound. If immutable source plus replacements
outgrows it, canonical bytes stream into sparse replacements over the same JS
baseline; there is no nested-graph upgrade or operation replay.

Ordinary concrete builtin source selects this representation. Prepared/raw
construction, arbitrary providers and typed-model adaptation retain their
existing ownership unless they already hold a genuine source-backed state.
Parsing completes before source construction, host callbacks or publication.
No global arena reset, native memory lease or scalar host import is required.

Compatibility verification includes complete source graph/binary comparisons,
first-use helpers, partial errors, edits and repair, exact descriptors/UTF-16/number
bits, callback restoration and unchanged provider dispatch. Matched VM parse,
retained/source and first-use measurements plus local Swift checks gate the
performance claim. This architecture alone establishes no speedup or release.
