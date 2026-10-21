# Captured public class constructors

Native factories support all twenty-two authentic builtin root classes, plus
ClassMember (`PrototypeId(2)` on Class). The latest native layer adds Sequence,
Block and Agentflow and completes the root-factory inventory. They
complement [caller-authored graph construction](DECLARED_MODEL_CONSTRUCTION.md):
construction supplies real class defaults; graph adoption never invents defaults.
Fresh Rust, Swift/C and three-browser verification is recorded in the
[ordered constructor checkpoint](parity/2026-10-21-ordered-constructors/README.md).
Factory coverage does not establish full public API parity.

## Current source status

Source `0eeba91` implements the ordinary constructor and owned-operation slices
described below. Fresh-artifact acceptance passes 99 Swift features, seven C
clients and 387 browser checks; [exact evidence](parity/2026-10-21-api-completion/README.md)
records platform identities and limitations. The earlier sections
retain checkpoint-specific evidence and limitations; this status supersedes
their claims that constructor reflection, cross-renderer class sharing,
constructed Agentflow logging or original clock exceptions are unimplemented.

The authenticated native factory catalog supplies artifact-specific names, arity
and class identity. Thin web shells preserve the allocated receiver, subclass
`newTarget` and the pinned artifact realm. Reflection registrations are sealed
and bound to the captured native schema; copied schema/name metadata cannot
grant builtin authority. Default ER publication receives the capability from its
selected retained registration before creating public objects.

Constructed Agentflow uses the shared staged graph builder and commits a cache
only after successful completion. Diagnostic callbacks retain partial edits; a
completed nested cache survives an outer failure. C20 lets the native owned
renderer read and call Swift opaque constructor values without enumerating them
or holding a document borrow. Gantt parse/today and scheduler callbacks use the
same owned Resource operation and allocate returned Dates in the latest graph.

Firefox can expose different intrinsic constructor property orders for identical
pinned Pele copies. Only that constructor ordering is excluded there; other
reflection checks and Chromium/WebKit ordering remain strict. Raw upstream
controls are retained in the checkpoint.

These additions preserve the existing constructed-owner/raw-render distinction: a
constructed model does not become a parsed core document. General Proxy/accessor
metaprogramming, modified String prototype behavior, advanced mutable Class enum
semantics, and raw rendering inside a provisional constructor body are not
blanket compatibility claims. Native graph coordinates cannot reconstruct
JavaScript deletion/reinsertion history absent from an applied snapshot.

See [owned runtime completion](LLD_OWNED_RUNTIME_COMPLETION.md),
[detached rendering](LLD_DETACHED_RENDER.md), and
[native opaque host access](LLD_NATIVE_OPAQUE_HOST_ACCESS.md). Final acceptance
counts and artifact hashes must come from the new platform logs, not earlier
constructor checkpoints.

## Factory contract

`PublicEngine::capture_constructor(&DiagramId, PrototypeId)` returns an optional
`CapturedPublicConstructor`. `PublicDocument::capture_constructor(PrototypeId)`
captures from that document's retained registration. Both use concrete creator
identity, not matching schema strings. Custom providers default to no factory;
even a provider forwarding Sankey's opaque factory cannot acquire its authority.

The captured handle exposes `prototype()`, `schema_view()`, `ignores_arguments()`
and `construct()`. Its registration survives engine destruction/replacement and
host registry close. Each construction owns independent native state at revision
zero. The private factory explicitly selects the ignored-argument policy;
future option-taking constructors must supply their own policy rather than
inheriting an adapter assumption.

Sankey's implicit constructor ignores all arguments without enumerating them or
reading getters. Its Rust builder uses `SankeyModel::new(None)` and the existing
ordered initial graph builder, without a parser call or seed source. It creates
fresh nodes/links arrays and lookup Map, type `sankey`, and undefined title,
accessibility title and description. Existing eager projection, helper partial
mutations, raw rendering and snapshot admission remain authoritative. Schema
and graph validation complete before native construction returns; transport
byte admission remains at the existing export boundary.

`PublicDocument::is_constructor_created()` records this provenance. A native
`PublicDocument` is an explicit low-level owned document and its render/helper
operations remain usable. Construction does not mint a parsed `ParsedDocument`.
The web high-level core facade must separately reject constructed roots, exactly
as pinned Pele core rejects `new SankeyDb()`, while raw Sankey entry rendering
accepts them. Raw rendering must not upgrade the root to parsed provenance.

The web shell owns JavaScript `new`/`newTarget`, the allocated receiver, ordinary
prototype mechanics and exception identity. It fills that receiver with the
native graph and gives it a separate constructed owner. It must not interpret
Sankey defaults, pre-observe ignored arguments, or replace the receiver with a
canonical-prototype object. C/Swift constructor handles expose the same native
capability explicitly, without simulating JavaScript class syntax.

C v16 capture takes a live registry, diagram ID and public prototype ID, returns
`{selected:bool}`, and publishes an owned factory only on success. A foreign
provider overlay returns unselected even if an authentic builtin is still linked.
Constructed document metadata includes `constructorCreated:true`; older creation
envelopes and existing v4 document ownership remain compatible. Free the factory
and each document independently, after their synchronous calls return.

Swift exposes `Registry.captureConstructor(for:prototype:)` and the owned
`Pele.PublicConstructor`. `construct()` returns a fresh `MutableDocument`;
`constructPrepared()` returns a fresh `PreparedDocument` using the named raw
render contract. Both expose `isConstructorCreated`. Closing the registry or
factory does not close already constructed documents. A closed factory rejects
future construction. Owners remain confined to one actor/thread.

Public integration `public_sankey_constructor` compares default graph shape,
own descriptor flags, aliases, raw SVG/measurement traces, first helpers,
readonly partial failure and repair against an actual pinned Pele fixture. It
also exercises independent constructions, retained factories after engine
replacement/drop, captured document factories and rejection of forwarded
provider capabilities. The fixture generator is
`scripts/parity/capture-public-sankey-constructor.mjs`; the input is the pinned
Pele ESM Sankey entry, not another Rust construction path.

`public_direct_constructors` covers the seven additional classes against
`public-direct-constructors.json`: ordered defaults, 19 helper calls, seven
repeated aliases and 14 raw outcomes. XYChart's empty constructor correctly
throws the upstream no-plot error. ER's host policy retains its original native
registration and exposes the same captured factory protocol as ordinary owned
documents. It performs no schema-based certification or registry lookup after
closure. Native feature tests cover source, typed fallback, prepared and raw
policy lifetimes; web and Swift consumers exercise the actual adapters.

The next six factories reuse empty native builders shared with parser population;
construction performs no parsing. Their feature fixture covers 17 helper calls,
six persistent aliases and 12 raw empty/populated outcomes. State shares its
ordered field initializer while preserving a distinct lifecycle: construction
does not extract the root document or populate the drawing cache. Its fixture
covers seven helpers, two aliases, explicit root-document replacement and three
raw drawings. Web checks also preserve the shared Mindmap/Kanban getType helper.

Fresh consolidated verification for this extension: nine native integrations,
ten Swift features against a freshly built ARM64 SDK, one strict C client
exercising all fifteen factories, and six browser feature executions across
Chromium, Firefox and WebKit. The browser fixture covers 21 construction scenarios
and seven retained-registry lifetimes. These results do not certify the remaining
seven class-root families (Class, Flowchart, Swimlane, Agentflow, Gantt, Sequence
and Block), nested ClassMember, or the metadata gaps below.

Cross-family and cross-renderer constructor/prototype unification is not claimed
by this slice. In pinned Pele Flowchart and Swimlane share FlowDb; option-taking
classes also have ordered getter, reference-retention and callback requirements.
Those are subsequent functional slices, not arguments for parse-empty factories.

## Web metadata scope and remaining parity gaps

The web feature proves construction behavior and ownership, not complete Sankey
constructor metadata parity. The native shell exposes the friendly schema name
`SankeyDb`; the pinned ESM Sankey constructor is named `B`. The JavaScript class
shell also has its own inferred lexical name, so engines that include that name
in their call-without-`new` error produce different text. Node/V8 source inspection
observes `Class constructor constructor cannot be invoked without 'new'` for the
shell versus `Class constructor B cannot be invoked without 'new'` for the pinned
ESM class. Both reject with `TypeError`; exact engine message parity remains open.
Core-bundle and ESM minification may name the same source class differently.

`tests/web/sankey-constructor.spec.js` records native, core-oracle and ESM-oracle
names and complete non-new errors without rewriting them. It separately asserts
the matching intrinsic contract: arity zero; own key order against the same live
browser oracle; data-descriptor flags for each; prototype member order;
`prototype.constructor` writable/non-enumerable/configurable flags and exact
backlink identity; and ordinary Function/Object prototype parents. These metadata
assertions do not compare the known differing name values or error messages.
Earlier all-family model tests exclude the prototype's `constructor` from method
comparisons, so their success is not evidence for this missing metadata parity.

The seven-family test also retains complete ordered intrinsic observations.
Firefox exposes a further key-order difference among these classes: some pinned
constructors expose `prototype,length,name`, while the shell exposes
`prototype,name,length`. Those observations remain diagnostic rather than a
passing reflection-parity claim. The feature asserts the three intrinsic keys'
presence and descriptor flags, and compares model fields, aliases, helpers and
render outcomes exactly. Initial stricter observer failures are retained.

A future native-authenticated constructor identity catalog must distinguish the
pinned runtime name from the descriptive schema name and define the shared
constructor/prototype identities across core, raw entries and registries. This
slice neither infers authority from a name nor introduces a global schema-name
cache. The scoped FlowDb raw binding below does not unify the global constructor
identity catalog.

## Retained options and owned method phases — native layer

Flowchart, Swimlane and Gantt now have native first-value factories in addition
to the sixteen ignored-argument families (including Class). This native layer
is independently tested; its new web and C/Swift host adapters still require
platform verification. It does not establish full cross-platform API parity.

`ConstructorArgument`/`PublicHostValue` preserve undefined, null, Boolean,
number bits, owned UTF16 and an owner-local opaque identity. Undefined defaulting
belongs to the family initializer. Retained options are neither copied nor
inspected during construction. Authenticated `live_fields()` metadata identifies
fields the adapter must protect from eager observation, including native-created
default objects retained in detached history.

`prepare_detached_invocation`, `resume_detached_invocation` and
`finish_detached_invocation` expose an owned, creator-checked continuation.
Native code selects Get, Call, Truthy, StrictEqual, GreaterEqual and ToString at
the actual statement. Hosts perform only those ordinary value operations, with
no diagram decisions. Every accepted resume advances the current revision
before mutation; finish releases its LIFO reservation without another increment.
Dropping a suspended job therefore leaves partial state at its published revision.
Same-owner helpers may run between phases; completion uses the latest state.

Call result transfer is explicit: `PublicHostResult::Discard` never examines a
callback return, `Retain` forwards its original host value without scalar copies,
and `Value` imports an owned scalar. Native code also supplies the call-site
noncallable error. Gantt captures its second warning function and receiver before
calling the message producer, then forwards the untouched producer result.

Flowchart/Swimlane reuse the existing helper algorithms around live reads for
addVertex, addSingleLink, addLink and addSubGraph. Gantt adds the explicit warning
helper continuation; scheduler-driven warning callbacks remain separate work.
Four native features pass in the combined source: Class defaults/helpers/raw
render, two Flow features covering forty pinned cases and lifecycle/scalar
behavior, and nineteen Gantt warning cases plus defaults/raw rendering. Existing
Flow methods/SVG regression also passed in the private candidate. This is scoped
feature evidence, not an all-library or platform acceptance result.

## Ordered constructor bodies and provisional receivers

`begin_construct_with_arguments` creates an owned `PublicConstructionJob`.
Its snapshot, apply and invoke operations expose the provisional receiver between
native-requested host operations. No document borrow spans a getter or callback.
The authentic factory supplies immutable `argument_transfers()` metadata:
ignored arguments are not inspected, options retain their original host value,
and ClassMember's two scalar arguments use owned value transfer. JavaScript
function arity is separate from consumed argument count.

Sequence reads `wrap` before `sequenceWrap`; Block reads `warn`; Agentflow
retains options and reads `lineOffset`. Native empty builders preserve exact
field order and fresh collection identities without parsing seed source.
Block's initial root children and blocks are separate arrays. Agentflow keeps
Pele's first-draw geometry cache, including edits made after that first draw.

ClassMember initializes its six fields, requests virtual parseMethod and
parseAttribute calls in upstream order, and uses existing native helper bodies
for their defaults. `finish_retaining_receiver` preserves a receiver exposed by
a virtual callback that subsequently throws; successful construction and retained
failed receivers remain distinct outcomes. Finalization resets unpublished
initialization to revision zero and never grants parsed-model provenance.

Agentflow's edge, subgraph and explicit diagnostic helpers use owned phases for
live options and logging. Nested helpers and original host exceptions retain
partial mutations. Graph-build diagnostic callbacks, arbitrary virtual helper
overrides and callback mutation of input arrays remain outside this slice.

The consolidated native batch passes twenty-four public feature tests: ordered
Sequence/Block/Agentflow construction, ClassMember dispatch and escaped receivers,
Agentflow live options/raw rendering, and Class/Flow/Swimlane/Gantt/shared-class
and Block warning/hierarchy regressions. Fixtures come from the pinned Pele entry modules. Modified string
prototype methods, shared readonly Class enum mutation and complete constructor
reflection identity remain explicit gaps. C/Swift and web adapters are verified
separately against fresh artifacts.

Swift exposes `beginConstruct(arguments:)` for provisional-owner access and
`construct(arguments:)` / `constructPrepared(arguments:)` for ordinary evaluation.
The existing `firstArgument:` overload uses the same generic job for every
family. `PublicHostObject` and `PublicHostCallable` express foreign getters,
coercion and calls explicitly, without reflection or family-specific adapter
logic. The actor-confined host store follows the completed owner; close and run
reentry are rejected while short snapshot/apply/helper calls remain available.

C18 provides matching begin/resume/snapshot/apply/invoke/finish/free operations.
Finish can transfer a partial owner alongside an error; every non-null returned
owner must be freed. Its bounded apply accepts and ignores snapshot metadata,
allowing C18 snapshot roundtrips; C5/WASM apply retain their stricter empty-metadata
contract. No metadata supplies graph or registration authority. Existing C16 and
C17 entry points remain available.

## Web retained options and compatible raw owners

The web adapter retains constructor options in an owner-local opaque ledger.
Authenticated live-field metadata masks the actual receiver's declared options
field during observation and publication, including replacement/accessor fields
and native default objects retained in history. Rust selects each Get, Call or
coercion phase. JavaScript performs that operation, publishes partial native
writes, then observes callback edits before resuming. An explicit per-job failure
flag preserves thrown `undefined`, symbols and original objects; nested caught
failures cannot replace the outer operation's result. Ignored returns are never
classified. Retained returns use temporary identity tokens and are released only
after completion when the admitted graph no longer references them.

Flowchart and Swimlane share the actual FlowDb constructor shell. A captured raw
renderer can operate on a compatible owner without replacing its registration or
copying its graph. Native authorization requires an authentic creator-filtered
factory, matching private class and PublicState TypeIds, the actual erased state
type, and the target root prototype. Swimlane explicitly shares Flowchart's class
identity inside Rust. Forwarded provider factories cannot confer this authority.
The target captured registration survives registry destruction; original helper
routing and revision ownership stay with the document. JavaScript retains target
capabilities per observed raw entry and selects them only inside the existing
native busy guard. Cross-renderer/global constructor catalogs remain separate.

Public feature evidence covers 66 Flow/Swimlane scenarios, constructor options
history/accessors, borrowed receivers, registry lifetime, 19 Gantt warning cases
plus six retained-result cases, and declared Class defaults/helpers/raw output.
The original three-engine 45-test attempt is preserved: 37 passed, eight failed.
Five feature failures were browser-generated TypeError wording only; three older
Sankey failures were a now-stale Flow-unsupported assertion or the already-known
Firefox intrinsic key order difference. The stale assertion now verifies real
Flow construction and its first helper.

Exact diagnostic comparisons remain executable and intentionally expose these
open gaps. Separately named semantic comparisons accept only the recorded
candidate/oracle TypeError message pairs; all state, descriptors, callback traces,
original error identities, results and SVG remain exact. Their raw reports keep
both messages. A separate semantic Sankey comparison checks intrinsic presence
and flags while the exact comparison retains key-order equality. This is scoped
functional parity, not complete diagnostic or constructor-reflection parity.
No host-engine failure witness or error-message normalization was added to the
runtime. Arbitrary Proxy producer arguments and advanced Class shared-enum
mutation/getter behavior remain outside this feature's acceptance scope.

## Web owned constructor bodies

Captured factories expose immutable argument-transfer metadata separately from
intrinsic arity. The shell passes its arguments object without enumeration. Only
native-declared positions are read: retained values become owner-local opaque
references; value positions transfer scalars and exact UTF-16 strings while
objects remain opaque. Ignored positions are never inspected. ClassMember uses
two value positions and arity two; options constructors retain one position while
keeping their native arity zero. No host diagram-name dispatch selects this.

A `PublicConstructorJob` owns provisional native state. The web bridge binds that
state to JavaScript's already allocated receiver, retaining `newTarget` and its
prototype. The same host-operation loop used by detached helpers drives the body.
Each suspension publishes partial state, executes the requested host operation
without a native borrow, observes edits, and resumes. Inherited native helpers
called by ClassMember's virtual parser dispatch operate through the provisional
job's ordinary invocation API. There is no second helper implementation.

Completion returns the owned document and its bounded canonical result frame.
The adapter swaps the handle inside the existing owner entry and finalizer; it
does not replace the public receiver or aliases. A throwing virtual parser can
therefore expose a partial receiver whose helpers remain usable. The per-job
original-error channel wins over subsequent observation or transport failures.
A transport failure that cannot supply the final document retires its provisional
bridge rather than leaving an escaped receiver bound to freed native memory.
Constructed owners never enter the parsed-document map.

The provisional API supports ordinary helper invocation. Nested detached helper
jobs or raw rendering during the constructor itself are not exposed by this
carrier. General arbitrary Proxy/accessor reflection, modified String prototype
methods and cross-registry constructor identity remain separate gaps. The public
feature tests compare native argument reads, declared models and raw output with
live pinned Pele, and exercise ClassMember's actual overridden parser calls,
original throws, subclass identity and escaped-receiver recovery.

Web acceptance uses the packed artifact (118 files, WASM
`39801cab676c1ed541b09d1bc0e27047e34f3613c2de3993f2d71fed6e4cffc8`).
The first focused run found Block's newly assigned warning function hidden by a
mask bound before constructor completion. Deferring receiver mask binding until
completion fixed that generic timing error; opaque ledger membership remains
active throughout the body. The original failing report is preserved. The final
51 checks pass in Chromium, Firefox and WebKit, including the new exact body
comparisons, existing constructor semantic checks, all 33 prepared entries and
source-initializer regressions. Runtime source (442 paths) and package hashes
match the after-tests proof; the mask fix changed only full.js and graph.js,
without rebuilding WASM. Existing diagnostic/reflection gaps remain explicit.

## Owned calendar resource operations

`DetachedInvocationStep::Resource` carries a sealed one-use operation. Hosts run
it outside document and job borrows, then resume with its native result. Gantt's
start-date and scheduling methods reuse the same scheduler and calendar parser
as synchronous invocation. Resource failures retain their original host error,
while finish still releases the invocation reservation.

C19 transfers the operation into a separate owner-authenticated carrier.
`take_resource`, `run`, `accept_resource` and resume state4 consume each phase
once. A fresh phase identity rejects foreign jobs and old replies. No native
calendar result passes through JSON, preserving Date/number bits. Swift and WASM
provide generic resource execution; date parsing and task scheduling stay in
Rust. Dropping a carrier does not synthesize a successful callback result.
