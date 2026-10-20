# Direct normalized model projection

The immutable parse pipeline needs an eager, normalized public model. Gantt's and ER's
captured builtin registrations constructs that model from the existing JSON
serializer's formatter events, removing the complete intermediate model JSON
buffer and its recursive parse. It retains the typed model, owned normalized
Value, exact envelope-size pass and eager Swift copy. Mutable public graph
construction uses its separate direct-parse path.

## Private strategy and extension contract

The concrete erased `Module<D, P>` type captures a private projection policy
through `PhantomData<P>`. Both policies are zero-sized; there is no stored tag
or runtime policy branch. Unit builtin handlers remain zero-sized when boxed.
Only the audited Gantt and ER instances installed by `Engine::with_builtins` opt in.
Public `register` always selects the original projection, even when a replacement
uses either builtin descriptor or model type. Retained documents keep their captured
registration after replacement or engine destruction. Public traits, signatures,
C ABI and Swift/Web APIs do not change.

Projection validation runs once before dispatch. Arbitrary provider Serialize
implementations retain their existing one-call behavior, including side effects,
raw fragments, custom errors and malformed map keys. Other builtin families keep
the original path until individually audited and measured.

## Serialization and normalization

A private Formatter observes serde_json Serializer operations while delegating
the exact original writes to CompactFormatter and a bounded counting writer.
No replacement serde Serializer or JSON parser is introduced. The counter has
the original all-or-error 16 MiB write rule and retains no output buffer.

A bounded frame stack constructs existing Value arrays and Objects. Original
UTF-8 fragments and decoded escape scalars construct owned strings, preserving
NUL, BOM and Unicode spelling. Numeric spelling is captured from CompactFormatter
and parsed with the same f64 operation as legacy normalization; direct numeric
casts could change f32 semantics. Numeric/bool map keys remain strings. Existing
Object insertion and enumeration retain duplicate-key and integer-key ordering.
Serde JSON still decides nonfinite, enum, map-key and date serialization behavior.

Over-depth input stops retaining the partial tree but continues serialization
and byte counting. Full serialization errors and byte admission keep their
original precedence over the deferred 128-container depth error. Unsupported
formatter patterns and raw fragments discard partial state and invoke original
projection. Only the privately selected pure builtin can reach this exceptional
retry; providers never do. This is a compatibility fallback, not a universal
formatter equivalence claim.

Gantt's audited serializer emits ordinary maps, sequences, scalars, empty JSON
objects and dates as strings/null. It has no raw fragments or injected custom
Serialize implementations. Public immutable documents expose shared model
references. Expanding opt-in requires auditing that concrete serializer and its
publicly reachable model states again.

ER's serializer tree contains derived records, string enums, optional strings,
vectors and ordered pairs. Its private parser HashMap/HashSet bookkeeping is
skipped; groups contain string references rather than recursive model values.
No raw fragments, custom framing, floats or callbacks are reachable through this
model. Its opt-in is one captured registration change, with the common formatter
unchanged. Sequence and Usecase remain legacy because their models forward
RawValue fragments; other families require their own audit and measurement.

## Verification and measurement

Public integration tests compare captured Gantt against ordinary public Gantt
registration: complete normalized values, exact envelope bytes, retained render,
Unicode/escapes, duplicate IDs, empty models and exact 16 MiB minus/equal/plus-one
admission. Replacement-provider controls cover single serialization, custom/raw
number handling, dates, malformed keys, depth and error precedence. These controls
prove provider isolation; they do not inject unreachable cases into the optimized
formatter. No public test-only opt-in exists.

ER additionally compares all 17,437 frozen source/options cases through both
actual model_view policies, including complete error and document envelopes.
Its integration slice checks ordered numeric-looking IDs, duplicate attributes
and classes, BOM/control/Unicode content, retained rendering, exact model-byte
admission and late syntax errors. Captured provider replacement must serialize
once; failing validation must precede serialization. These tests add no public
opt-in and do not inject unreachable float/raw/depth cases into builtin ER.

Frozen source-matched Swift executables establish end-to-end eager parse cost.
Source and typed retained rendering remain controls. Rebuild the WASM package and
measure on the independent VM: unchanged mutable model code does not justify
assuming unchanged compiled performance. Keep raw samples and artifact hashes.
Geometrically growing arrays and formatter bookkeeping can offset eliminated
buffering; source inspection alone cannot establish an optimization win.

The first rollout stored a runtime enum beside each unit handler. That caused
33 additional one-byte allocation requests when constructing builtin registries.
The static policy correction restores the prior boxing behavior. The public
`registry_allocations` example records construction-only requests: 69 to 36
allocations, zero remaining one-byte handler requests, and unchanged four
reallocations in this native diagnostic. Requested bytes are cumulative traffic,
not allocator-reserved memory, live memory or RSS. Timing needs separate proof.
