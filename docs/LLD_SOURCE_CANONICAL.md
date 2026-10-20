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

Crate-owned logical readers can retain validator-issued admission receipts.
The opaque capability identifies the exact reader and its owner generation;
arbitrary providers keep their existing validation path. Structural admission,
static schema identity and the authoritative canonical byte count are separate
facts. Every actual mutation invalidates them before writing, including partial
failures and graph adoption. Unchanged exports still check the current metadata
length and combined byte budget. Standalone binary and FFI encoders retain their
existing validation stages. Generation exhaustion disables reuse.

Fresh source construction feeds scalar fields directly into the authoritative
structural validator. An optional observer uses the shared canonical-width
counter and records schema IDs in encounter order during that same pass. Only
successful structural validation can issue this sealed initial evidence; no
property batches or separate initial schema/count walks are required. Schema
compilation and ID membership checks retain their existing snapshot boundary.
Byte-count failures remain deferred until export, after metadata serialization;
each export still checks its current metadata length and combined budget.
Mutation and adoption discard the initial evidence before writing and retain
the generic validators, including after partial failures. Arbitrary providers
cannot supply this crate-private initial traversal. Source layout encoding keeps
its existing declarative recipe; this layer changes neither its ABI nor its
field declarations.

Explicit APIs returning `ModelGraph` or `Cow<ModelGraph>` still materialize that
representation at their original export stage. Additive `snapshot_read`,
`invoke_read` and render read outcomes let binary/JSON adapters avoid that copy.
Old owned outcomes keep their original public types and presentation ordering.

## Browser publication and recovery

A side-effect-free capability probe precedes source publication. Legacy
providers return false without exporting twice. A capable document emits an owned
`PLCSRC02` recipe: the original string catalog, immutable scalar columns,
declarative record layouts and complete current string/record replacements.
Later `PLCSRU01` outcomes reuse that immutable base, identified by a checked
process-local ID, and carry complete current replacements and graph extents.
Reset and truncation reconstruct strings from the original catalog, including
after canonical fallback. Numeric payloads retain raw IEEE words. The generic JS
reader evaluates layouts; grammar, helper policy and rendering remain in Rust.
The previous self-contained `PLCSRC01` decoder remains supported. Exhausted owner
IDs permanently select that format for new owners without adding an error.

The existing `ModelBridge` retains registry identities, busy/disposed state and
callback dispatch. Its storage binding selects logical traversal once. Capture
stores complete current replacements against the original source. A sealed,
creator-checked capability can accept these replacements directly; generic
providers retain existing full graph apply/ModelDelta ingress. Every
final outcome restores the complete accepted graph, including historical
objects. A failed observation or publication never confirms a new baseline.
An accepted revision is not rolled back because transport or a result factory
throws. The original factory value travels with the accepted graph and is thrown
after publication.

Logical publication, matching and capture use an operation-local node cursor.
It binds an immutable layout or sparse replacement once per node, supplies
key-only reads during deletion checks, and decodes collection atoms once.
Semantic numeric expressions stay numbers; raw IEEE words remain available for
exact NaN comparisons and canonical capture. Each invocation and capture owns
its cursor, and values are copied before descriptor or Proxy actions can reenter
another document. Generic mutation/restoration policy, fresh assignment,
canonical IDs and non-layout storage behavior remain shared and unchanged.

Fresh source publication can use a generated initializer compiled offline from
Rust's actual RecordSchema and source recipe. It creates the same public objects
with fixed field sites and registers every object in both identity ledgers.
Runtime eligibility checks the full schema, recipe structure, scalar tags, earlier
references, dense keys and ordinary inherited field descriptors before creating
any object. Unknown layouts, edits, non-finite values and negative zero retain
generic publication. The accepted layout remains the original bridge baseline;
helpers, capture and recovery keep the shared path. No grammar, helper policy,
layout algorithm or native pointer lifetime moves into this initializer. The
checked-in output must reproduce exactly from the built native schema/recipe.

Edited source ingress uses a separate bounded PLCSRI01 envelope: exact immutable
layout identity, expected revision, current root/extents and complete original-base
string/record replacements. It does not send the delta from the last JS observation,
which may have been rejected natively. Missing overrides restore original data.
Before dispatch, JS preserves the old full-frame count/missing-slot pass and scalar
format error order. Sparse physical overflow selects full ingress before mutation;
there is no failure retry or operation replay.

Native physical scanning borrows byte slices without table-sized allocations. A
shared wire decoder serves both scanning and legacy materialization. Two streaming
original-base merges first check full canonical width, then version, table, UTF16
and aggregate work limits in existing order. Only admitted replacements allocate.
Existing structural/schema validation, revision checks and eager imported Result
retain their stages. Commit invalidates receipts, replaces sparse state, clears
helper patches and advances revision atomically. Custom modules cannot forward a
builtin ingress token to bypass their captured apply hook; creator TypeId must
match the captured module. Existing binary and C/Swift entry points stay intact.

Prepared source admission binds its already completed canonical size and decoder
allocation gates to the exact owned overlay and exclusive state borrow. A
consuming PreparedSourceIngress performs the first structural traversal before
revision checks. A scoped StructuredSourceInput reuses only that candidate's
structural proof at registration and module boundaries; schema compilation and
the shared ordered schema-ID walker still run at the original stage. Only an
AdmittedSourceIngress can commit. Dropping a failed candidate leaves accepted
state, revision and receipts untouched. No global receipt or provider trust is
extended.

The same eager importer now supports a crate-private source-text policy paired
with its exact immutable candidate. Original catalog keys compare directly as
UTF8; original labels/title copy once from valid source text. Incoming UTF16
string replacements resolve first and use the existing strict chunk, length and
surrogate decoder. Public CapturedRead implementations keep that decoder. Array
membership, first-property matching, index interpretation, latent import errors
and text-budget charge order remain in one importer body.

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
