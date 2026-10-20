# Captured public class constructors

Native factories support fifteen authentic builtin root classes: Sankey, Venn,
Ishikawa, Requirement, Railroad, C4, XYChart, ER, Architecture, Kanban, Mindmap,
Timeline, Journey, QuadrantChart and State (`PrototypeId(1)`). They
complement [caller-authored graph construction](DECLARED_MODEL_CONSTRUCTION.md):
construction supplies real class defaults; graph adoption never invents defaults.
Other root classes and nested `ClassMember` remain incomplete.

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
cache. FlowDb sharing and the other constructor factories remain separate work.
