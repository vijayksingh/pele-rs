# Current functional parity — incomplete (2026-10-21)

Native factories now cover all 22 class-root families plus nested ClassMember.
The latest integrated batch passes 13 public feature tests, including ordered
constructor getters, virtual member parser dispatch, escaped partial receivers,
live options and raw rendering. Fresh Swift/web verification of the ordered-body
layer remains pending; the cross-platform checkpoint below stays separate.

Fifteen class-root factories are implemented: Sankey, Venn, Ishikawa,
Requirement, Railroad, C4, XYChart, ER, Architecture, Kanban, Mindmap, Timeline,
Journey, QuadrantChart and State. The latest seven-family extension passes nine
native integrations, ten Swift features against a fresh ARM64 SDK and a strict
C client exercising all fifteen factories. Six browser executions across
Chromium, Firefox and WebKit cover 21 construction scenarios and seven retained
registry lifetimes. State preserves its initially empty drawing cache until
explicit extraction. The seven remaining root families are Class, Flowchart,
Swimlane, Agentflow, Gantt, Sequence and Block; nested ClassMember is additional.
Constructor reflection/shared identities and callback mutation behavior also
remain work. See [constructor scope](LLD_PUBLIC_CONSTRUCTORS.md).

The earlier seven-family extension passed four native features, ten Swift
features, a strict C client and six browser executions. A broader browser run
passed 62 checks but failed one constructor reflection-order observer; that
known difference remains explicit, with the original failed evidence preserved.

The preceding functional slice adds captured Sankey construction through Rust, C v16,
Swift and WebAssembly. Fresh factories own exact native defaults, helpers and
raw rendering without parsing; captured registration and subclass receivers
survive the relevant lifecycle operations. Two native Rust features, two strict
C clients, two Swift constructor features and six real browser executions pass.
The prior 31 Swift regression features passed against the new SDK. Constructor
runtime names/diagnostics, shared prototype identity and the other 21 class-root
families remain incomplete; see [constructor scope](LLD_PUBLIC_CONSTRUCTORS.md).

Native normal/prepared Sankey callback reentry is implemented and separately
[verified](parity/2026-10-21-native-sankey-callbacks/README.md). Presentation/image
routes and arbitrary native graph-ID lifetime remain outside that proof.

All 33 diagram parsers/renderers and mutable family adapters exist. This is not
full public API parity. The current functional audit against pinned Pele 1.6.0
records these concrete gaps at runtime `d7caf8c`:

- **31 of 33 prepared per-type entries lose callback mutations:** the first
  width callback changes `model.title`, then throws an original object. Pele
  retains the edit; native restores the previous title. All 33 preserve throw
  identity and call width once. Sankey and ER retain the edit in this case.
- **The audit found 22 constructor stubs.** Subsequent slices implement 15;
  all 22 now have native factories, with the latest platform verification pending.
  Plain-object families are separate.
- Native codecs still explicitly reject active lone UTF-16, sparse collection,
  dynamic coercion and overridden-dispatch cases. Some integration tests
  deliberately assert these gaps and recovery; their passing totals must not be
  reported as complete behavioral parity.
- Subsequent C/Swift normal/prepared Sankey callback reentry is verified above;
  presentation/image callbacks and arbitrary native graph-ID lifetime remain open.

See the [33-family checklist and reproducible evidence](parity/2026-10-21-runtime-audit/README.md).
Runtime behavior, fresh platform artifacts and release publication are separate
acceptance gates. The dated checkpoints below are preserved as historical
evidence, not current full-parity certification.

---

# 0.3 source checkpoint — 2026-10-19

The v1-only renderer, wrappers and playground are removed. The package root now
exports the full library. Source checkouts require a matching local Swift SDK;
they no longer silently select an older downloaded binary.

Current checks: workspace/all-target Rust compilation, packaged Rust consumer
(all 33 mutable families and 15 typed families), 28 strict C executions on ARM
and actual Intel/Rosetta, all-33-family Swift consumers on both architectures,
ARM AppKit pixel smoke, 60 focused browser checks across all three engines, including shared initialization
and failed-load recovery, TypeScript,
and 109 exact-WASM consumer checks. Production diagram logic remains unchanged.

Performance evidence is separate: 21 workloads, 126 rows per independent-VM web
condition and 168 actual Swift rows. The no-slower-than-JS target is not met;
[results](benchmarks/2026-10-19-vm/README.md) are published without a speedup claim.
0.3 remains a source candidate; historical release evidence follows.

---

# 0.2 verification checkpoint — 2026-10-19

All 33 native Rust diagram renderers, mutable codecs, prepared/raw entries and
caller-graph constructors are implemented. Swift and packed WASM expose the
inventoried interfaces, including native resources, interaction/accessibility,
Gantt live callbacks and ClassMember nested original-method delegation.
Runtime implementation is frozen at `0c0c5a4` / `fa07860`.
[v0.2.0 is published](https://github.com/vijayksingh/pele-rs/releases/tag/v0.2.0)
at `7a8b958`, with completed distribution/remote-consumer checks; see
[0.2 acceptance](RELEASE_0_2.md).

## Completed acceptance

- **Rust:** all 88 integration targets pass, 380 tests, zero failures/ignored.
  Source hashes remained unchanged. Validation used release opt-level 3, LTO off,
  codegen16, two jobs and two test threads; distribution uses its normal release profile.
- **Native:** a matching universal arm64/x86_64 SDK passes 28 strict C executions
  (14 per architecture), 176 Swift features (88 per architecture), all-33-family
  painted-pixel smoke checks and independent SwiftPM consumers on both targets.
  The 502 recorded source/SDK inputs are unchanged. The original Intel SwiftPM
  discovery failure is preserved; all 25 non-GUI classes passed through actual
  Rosetta/x86_64 XCTest without product changes or repeated C checks.
- **AppKit:** final harness run `run-scu8m8og` after `95d33ae` passed 31/31 in the
  active signed QA app, PID 88918: Mount 10, Zoom 9, Presentation 7, NativeResource 5;
  zero failures. Earlier active run `run-9m9jrn6g` also passed 31. Inactive attempts
  executed no assertions and remain recorded as unsuccessful attempts.
- **Web:** all 285 unique checks have passing evidence, 95 per Chromium, Firefox
  and WebKit, with one worker and no relaxed assertions/timeouts. Initial run:
  284 passed, one WebKit Class check interrupted by OS low-power sleep. The host
  slept 17,654 seconds (4h54m14s); that unchanged Class check passed separately in
  21,758 ms with its original 120,000 ms timeout. This was not a single uninterrupted
  285-pass run. Original raw results and sleep evidence are retained. There are
  zero skipped/flaky/retried cases; the separate confirmation is explicit.
- **Package identity:** the browser run verifies exact archive entries/extracted
  tree, three identical WASM copies, unchanged source tests and 33 diagnostic
  ledger attachments. Independent TypeScript consumer checks pass. License
  commit `415a04a` adds dependency/glue/stdlib notices; final archive hashes change
  while accepted runtime bytes must remain unchanged.

Native gate record: `dist/evidence/native-02-final/result-final.json`.
GUI record: `dist/evidence/native-ui/run-scu8m8og/result.json`.
Browser gate record: `/tmp/pele-final-web-ae1b146/gate-proof.json`, preserving the
initial 284-plus-confirmation account, source/package identity and ledger hashes.
These local records are evidence locations, not published download claims.

Final web delivery: all 104 packed entries match the browser-tested archive
except expanded license notices. Independent TypeScript and 109 exact-final-WASM
checks pass, including all 33 pinned SVG baselines, Class/Gantt callbacks and
caller-built models. Both architectures pass consumers from the exact final
universal ZIP. The demo records 114 real browser frames (19 seconds) from the
final web archive.

Publication verification passes: all public downloads/checksums, fresh Rust Git-tag
consumer, actual npm tarball installation with 109 checks, and artifact-free
SwiftPM consumers on both architectures. The public SDK/header match accepted
bytes and the README GIF matches its recording. The release attaches
`published-consumers.json`, `verification.json`, checksums and curated evidence.
The separate full remote CI rerun remains in progress at this checkpoint; local
acceptance and actual CI Swift 6 GUI recovery are complete. No registry publication,
unrestricted VM parity, GPU acceleration or speedup is claimed. Existing
[VM](WEB_VM_DIAGNOSTICS.md), [native work](WEB_LEGACY_CORPUS.md),
[construction](DECLARED_MODEL_CONSTRUCTION.md) and limit boundaries remain.

## Historical development record

The snapshots below record earlier revisions and superseded failures/unfinished
slices. Their old counts and pending-work statements do not override the current
checkpoint above. The repaired C4 failure is retained instead of erased.

---

# Full-library port: development evidence

Acceptance remains every row of `PELE_PARITY.md`: all 33 families and public APIs.
The public v0.1.0 release remains the legacy flowchart subset. No full-library
release or new GIF has been published from this branch.

## Current checkpoint: 2026-10-19

All 33 parsers/renderers and mutable model codecs are implemented. This count is
not full API or release acceptance. Later sections retain historical evidence;
their older gap descriptions do not override the checkpoints below.

- The numeric fix is checked across all 33 families: 1,023 pinned cases and
  3,069 source/retained/mutable renders, including NaN, infinity and signed zero.
- The complete browser regression passes 234/234 (78 per Chromium, Firefox and
  WebKit, one worker). The documented extreme Class native-output ceiling and
  VM-specific intrinsic diagnostics remain explicit transport/runtime boundaries.
  See `WEB_LEGACY_CORPUS.md` and `WEB_VM_DIAGNOSTICS.md`.
- Retained web custom providers are committed in `01ca5c7`; the expanded API
  regression passes 81/81 across the three browsers. This used a fresh isolated
  WASM build; it is not evidence that a published package contains these APIs.
- Native mount, lazy mounting, zoom and retained Swift custom providers are
  committed. The actual signed AppKit QA application passes all 19 mount/zoom
  features. Synthetic magnification is not physical-trackpad proof.
- `608464e` completes logical action regions for every current link-producing
  family: Flowchart/Swimlane, AgentFlow, ER and Kanban. Existing pinned producer
  corpora remain green. These regions alone do not prove native link interaction.
- `1ff8e01` and `1cf9f40` expose session resources and native presentation through
  C/Swift. One matching ARM64 release SDK passes all 17 selected Swift features
  (four presentation, thirteen provider/resource/registry) and four strict C
  integrations (presentation, resources, retained providers and render numbers).
  Its archive SHA-256 is
  `05344a5b19c62028694ebf8a0e8d1355a2bff3f79f8d0605ec44555b4e759a8e`;
  matching header SHA-256 is
  `c4e7be6035352d19818319a16ed4f6bb940a5b132a97ed4cbe86f3a837c1c3b0`.

Active remaining work: native link/accessibility controls and resources,
per-diagram prepared/raw entries and caller-built models, declared host-callable
model overrides, shared Config/ill-formed UTF-16 boundaries, then final matching
universal Swift/WASM packages and external-consumer/release verification. The
performance experiment remains on hold; no Rust/WebGPU speedup is claimed.

## Implemented renderers

`Engine::with_builtins()` currently registers all thirty-three actual Rust diagram modules.
`Engine::empty()` and typed `DiagramModule<Model>` remain extension interfaces.
Registration never certifies a global parity flag; all public API gates remain open.

| Family | Public scenarios | SVG results | Errors |
| --- | ---: | ---: | ---: |
| pie | 3,678 | 1,179 | 2,499 |
| quadrantChart | 10,614 | 1,586 | 9,028 |
| packet | 12,774 | 2,924 | 9,850 |
| radar | 8,431 | 1,791 | 6,640 |
| timeline | 11,932 | 8,017 | 3,915 |
| journey | 10,725 | 3,244 | 7,481 |
| flowchart | 18,175 | 8,952 | 9,223 |
| kanban | 11,681 | 5,290 | 6,391 |
| mindmap | 11,335 | 4,204 | 7,131 |
| class | 24,758 | 8,746 | 16,012 |
| er | 17,437 | 7,535 | 9,902 |
| xychart | 9,886 | 3,705 | 6,181 |
| state | 10,222 | 7,209 | 3,013 |
| gantt | 18,994 | 8,048 | 10,946 |
| sequence | 19,166 | 6,702 | 12,464 |
| gitGraph | 17,125 | 5,050 | 12,075 |
| architecture | 18,780 | 4,593 | 14,187 |
| sankey | 11,616 | 4,946 | 6,670 |
| treemap | 15,370 | 9,946 | 5,424 |
| c4 | 18,603 | 6,278 | 12,325 |
| info | 8,053 | 1,491 | 6,562 |
| requirement | 10,358 | 3,714 | 6,644 |
| block | 12,294 | 4,584 | 7,710 |
| railroad | 12,288 | 4,424 | 7,864 |
| cynefin | 7,209 | 3,011 | 4,198 |
| treeView | 6,800 | 4,301 | 2,499 |
| agentflow | 14,990 | 5,372 | 9,618 |
| wardley | 7,730 | 3,205 | 4,525 |
| eventmodeling | 6,953 | 2,437 | 4,516 |
| usecase | 16,744 | 3,424 | 13,320 |
| venn | 22,121 | 4,581 | 17,540 |
| ishikawa | 20,586 | 12,418 | 8,168 |
| swimlane | 10,405 | 4,568 | 5,837 |
| Total | 437,833 | 167,475 | 270,358 |

There are 172,247 complete model comparisons. Parsed models are retained when a
subsequent render fails. Six class stress cases explicitly exceed the JSON
projection depth/size budget; their typed models and exact source renders/errors
remain covered. These transport exceptions do not earn full public API parity.

These fixtures run through Rust, Swift/C and real browser WASM. One Venn error is an explicit native work-ceiling extension whose upstream render was deliberately not executed; it is not an upstream error-parity result. Available model projections and all errors
compare completely. SVG compares byte-for-byte, or by SHA256 of every UTF-8 byte
for large outputs. Extractors pin the upstream revision and source-file hashes.

The actual parsers/models/layouts/renderers are translated, including narrow
adaptation, configuration, accessibility and host resources. There is no upstream
JavaScript renderer in distributed Rust/native/WASM code. JavaScript is a
build/test oracle and browser font/resource adapter only.

Full flowchart now uses the actual shared layered/compound, shape/edge geometry,
graph normalization and SVG renderer. Its 47,917-edge typed stress document also
passes foreign model projection. See `FLOWCHART_PORT.md` for declared-model and
private-cache scope. Kanban and mindmap share the translated outline/icon parser;
Rust owns all branch/column geometry and responsive behavior.

## Shared interfaces and behavior

- Astra LLD/source inventory: `67204fc`; 33 families and 287 source files accounted for.
- Compatibility core: `e86bfc5`; legacy Rust/C/Swift/WASM interfaces remain v1.
  Pie/owned host adapters: `82a2791`; development model/render results are v2.
- Modules retain typed models and resolved configuration. Documents retain exact
  registration/metadata even after replacement. Open IDs and `parse_as` support
  consumer languages without changing the pinned built-in detector.
- Engine-owned resource sessions cache decoded icons and capture one clock value
  per render. Text/icon callbacks stay per-call; wrappers do not own grammar/layout.
- Ordered JS-compatible values, preprocessing, YAML and entities remain shared.
  Source locations and ordinary source/output limits use UTF-16 units.
- Shared hidden-YAML token matching now preserves the greedy optional-body branch
  found by packet differential tests; pie and radar use the same implementation.
- Typed packet rows share label strings. JSON projection stops while serializing
  above 16 MiB, before allocating a pathological 400 MB model projection. Rust
  typed models remain accessible, and source/document rendering remains usable.
- Native export resolves CSS theme tokens, moves rounded rectangle radii into
  style declarations, and repeats first-tspan coordinates on parent text. AppKit
  otherwise drops such text despite a valid SVG accepted by browsers.

## Verified delivery lanes (2026-10-18)

Host: macOS 26.6 arm64 / Apple M1 Pro, Rust 1.98.1, Swift 6.4, Node 25.9.

- Public Rust integration coverage passes across legacy, preparation, extension,
  all eighteen families, including full flowchart rendering. The consolidated
  release-profile run passed 50 public integration tests at the fifteen-family checkpoint; seven more GitGraph/Architecture/Sankey integration tests pass in release mode. No unit tests added.
- Source preparation: all 5,345 scenarios, including 761 examples from 34 upstream
  corpora; extension links: 1,536 entity/link scenarios.
- All seven Swift integration tests pass. The additional-diagram test exercises
  every packet/radar/quadrant/timeline/journey/flowchart/kanban/mindmap/class/er/xychart/state/gantt/sequence/gitGraph/architecture/sankey fixture through owned documents,
  callbacks and source rendering; pie and preparation have their own full corpora.
- Fifty-one browser integration checks pass with one worker in Chrome, Firefox
  and WebKit, using the extracted npm tarball. Includes exact corpus comparisons,
  actual SVG pixels, browser Canvas/font behavior and callback error propagation.
  The consolidated run passed 48 checks; the final three XYChart checks passed
  after its targeted tick-portability fix. The State/Gantt/Sequence batch adds
  twelve passing three-browser checks (full source/owned-model corpora and native
  SVG decoder pixels). Fifteen more checks compare Gantt in five actual browser
  timezones, including gaps/folds, half-hour changes, leap/month arithmetic,
  excluded weekends, ancient dates and Samoa's skipped day. The subsequent
  GitGraph/Architecture/Sankey batch passes fifteen additional three-browser checks:
  exact full corpora, native decoder pixels, and non-finite Sankey dimensions.
- C executable passes NULL/UTF-8/limits/ownership checks: 400 legacy lifetimes plus
  200 owned pie document lifetimes.
- Rust crate packaging and independent downstream Cargo consumer pass.
- Formatting/whitespace checks pass for completed slices. Shared graph/layout,
  shapes and edge helpers are exercised through actual public flowchart rendering.

AppKit decodes and paints current native exports. White proof backgrounds belong
only to raster evidence, while exported diagrams retain SVG transparency.

| Diagram | Native pixels | Additional text proof |
| --- | --- | --- |
| venn | 319 x 252; 48,377 painted | Intersections and 342 title pixels |
| ishikawa | 356 x 272; 6,991 painted | Fishbone branches and 445 title pixels |
| swimlane | 467 x 340; 10,323 painted | Lane routing and 472 title pixels |
| wardley | 846 x 610; 7,521 painted | Map, dependencies and 507 title pixels |
| eventmodeling | 418 x 263; 12,858 painted | Workflow lanes and 530 title pixels |
| usecase | 344 x 164; 4,207 painted | Actor, boundary and 362 title pixels |
| cynefin | 656 x 492; 77,124 painted | Five domains and 496 title pixels |
| treeView | 199 x 276; 4,477 painted | Files, nesting guides and 301 title pixels |
| agentflow | 620 x 100; 11,164 painted | Semantic workflow and 488 title pixels |
| requirement | 228 x 414; 7,895 painted | Tables, markers and 604 title pixels |
| block | 170 x 64; 1,390 painted | Grid nodes and routed arrow |
| railroad | 154 x 116; 1,271 painted | Alternatives, tracks and labels |
| treemap | 656 x 416; 242,303 painted | Hierarchy and 520 title pixels |
| c4 | 548 x 168; 4,989 painted | Elements, relation and 437 title pixels |
| info | 127 x 72; 1,106 painted | Pinned version and 419 title pixels |
| legacy flowchart | 416 x 789; 6,846 painted | Legacy outlined font path |
| pie | 358 x 296; 44,733 colored | Native compatibility export |
| class | 140 x 240; 3,904 painted | Members, relations and labels |
| er | 158 x 279; 4,467 painted | Attribute tables and cardinalities |
| xychart | 700 x 500; 81,821 painted | Bars, line, axes and 260 title pixels |
| state | 316 x 424; 5,636 painted | Composite states, arrows and 372 title pixels |
| gantt | 801 x 137; 14,583 painted | Task states, axes and 382 title pixels |
| sequence | 187 x 240; 4,492 painted | Messages, activation and 470 title pixels |
| gitGraph | 204 x 191; 4,812 painted | Commit symbols, branches and 491 title pixels |
| architecture | 155 x 128; 2,522 painted | Services, ports and 502 title pixels |
| sankey | 616 x 416; 224,041 painted | Flow bands and labelled nodes |
| full flowchart | 115 x 336; 3,338 painted | Nested group and resolved title halo |
| kanban | 428 x 105; 3,069 painted | Columns and card labels |
| mindmap | 269 x 73; 1,241 painted | Root and both branches |
| packet | 1035 x 98; 6,053 painted | 456 title pixels |
| radar | 470 x 419; 35,281 painted | 159 title pixels |
| timeline | 408 x 266; 49,449 painted | 295 title pixels |
| journey | 340 x 327; 4,719 painted | 413 title pixels |
| quadrantChart | 500 x 500; 8,718 painted | 373 title pixels |

Packet has one additional stress scenario: 40,000-character label shared across
10,000 rows. Rust verifies every typed row and exact rendered SVG hash. Foreign
hosts verify a bounded projection error and successful exact source-render hash.
The 400 MB JSON projection is deliberately not built.

## Numeric portability

Host comparisons found two failures that the original Rust JSON oracle comparisons
had masked. `serde_json` now enables `float_roundtrip`, preserving decimal values
through `Value::Number` serialization instead of moving some extreme values by one
ulp. WASM's libc `pow(10, integer)` also differed from the pinned JS engine, changing
XYChart tick strings and label margins. Integer decimal powers now use exact
string-to-f64 conversion; all 633 finite exponent cases from -324 through 308 were
checked against the development JS oracle. The public XYChart corpus passes on
native Rust, Swift/C and all three browser engines after both corrections.

Pinned XYChart explicitly clamped equal bounds can still emit NaN path geometry;
that upstream behavior is preserved and does not constitute a universal finite-SVG
claim. Ordinary diagrams have XML and real raster evidence.

## Additional host transport evidence

Sankey can produce a NaN result height from subnormal positive link weights in
the pinned implementation. JSON alone erases that numeric value to null. Render
transport now includes exact IEEE bits only for dimensions JSON cannot preserve;
Swift and web adapters restore those numbers and hide transport metadata. Swift
Codable round-trips the same bits. Ordinary finite result envelopes are unchanged.
Full non-finite source options and model graphs remain separate required API work.

Architecture icon tests also exposed an empty-string resource distinction: an
explicit empty icon suppresses its builtin, while a missing icon selects fallback.
Swift now supplies a non-null zero-length buffer for an explicit empty icon;
all five previously failing cases pass, with nil/error ownership unchanged.

The new GitGraph/Architecture/Sankey native corpus run passed all cases except
those five icon cases; the complete Architecture corpus and all seven Swift test
methods pass after that correction. These are cumulative host checks, not a new
consolidated eighteen-family Swift corpus run. Independent Rust/Swift package
consumers were last exercised at the fifteen-family checkpoint; final all-family
package and CI gates remain mandatory.

## Treemap, C4 and Info checkpoint

Eight additional release Rust integration tests pass. The complete three-family
corpora pass through Swift/C source and owned-document rendering, and through
Chrome, Firefox and WebKit (nine corpus checks). All three browser SVG decoder
checks pass, with Info's small version label required to paint at least 500 pixels.
AppKit paints each current native export; all seven Swift integration test methods
pass in this checkpoint. This remains cumulative family coverage, with the final
unfiltered all-family run required before release.

Treemap retains one deep-model JSON projection limit; C4 retains eighteen cyclic
or deep projections. Typed models and exact source renders remain tested. C4's
five header forms are one family, with a dedicated layout and obstacle router.
Info reports the pinned upstream version rather than the Rust package version.
Treemap classes and C4 extra attributes are restored as Maps by the web adapter.

Swift fixture models now decode directly from oracle bytes with JSONDecoder.
Foundation's Any projection exposed false numeric comparison failures and stripped
a leading U+FEFF in a model string; direct decoding verifies the public binary64
values and exact Unicode strings. No fixture cases were removed.

## Requirement, Block and Railroad checkpoint

Eight additional release Rust integration tests pass. All 34,940 new public
scenarios pass Swift/C source and owned-document paths; all seven Swift test
methods pass. Twelve checks pass across Chrome, Firefox and WebKit, including
complete corpora and actual SVG decoder pixels. AppKit also paints all three.
The native smoke sample uses Requirement's supported frontmatter title syntax.

Block's three and Railroad's thirty-three explicit deep JSON projections remain
bounded; their concrete models and exact source renders stay covered. Railroad
implements IR, EBNF, ABNF and PEG as one family. Its cached unclosed-comment
scanner avoids repeatedly searching the same suffix. No projection exclusion
counts as completed mutable/lossless public API parity.

## Cynefin, TreeView and AgentFlow checkpoint

Ten new release Rust feature tests and four extension-contract checks pass.
The complete three-family corpora pass through Swift/C; the selected corpus
method passed again after adding per-case and per-family autorelease scopes.
Sampling the earlier run showed Foundation temporaries accumulating until test
exit. The fixture reader now releases that temporary work as each case completes;
no cases or assertions were dropped. Native AppKit exports paint all three.

The browser checkpoint passes twelve cumulative checks across Chrome, Firefox
and WebKit. A retained read-only query interface verifies every AgentFlow semantic
model through Rust erasure, C/Swift and WASM; custom modules keep their query
handler after registry replacement. Query names/arguments are bounded, and the
builtin semantic projection uses the existing model-output budget. This is an
additive development query interface, not completion of AgentFlow's mutable
`semanticModel()` prototype contract.

TreeView preserves per-row icon callback order. Its three deep JSON projections
and twenty-seven isolated UTF-16 input mutations remain explicit ledger entries.
AgentFlow records four metadata surrogate exceptions; Cynefin has none. These
exceptions are not counted as completed public string/API cases.

## Wardley, EventModeling and Usecase checkpoint

Nine new release Rust integration tests pass. The three complete host corpora
pass the native Swift/C source and retained-document paths, plus twelve
cumulative browser checks across Chrome, Firefox and WebKit. AppKit also paints
all three native exports. EventModeling preserves its required source comments;
Usecase Map fields and JSON property-order maps are restored in the web adapter.

Four Usecase projections exceed JSON byte/depth budgets. The foreign checks
require those projection limits, then separately verify successful source renders
or the exact expected render-output limit. Projection-budget labels are diagnostic
fixture metadata, not render-result fields. No case is skipped. Usecase keeps one
JSON isolated-surrogate exception; EventModeling keeps two isolated output
surrogate exceptions from otherwise valid source. Those remain full-API gates.
Usecase JSON error text follows the pinned Node/V8 oracle; browser-engine-specific
JSON.parse wording is not separately certified by these canonical fixture checks.

## Calendar and deep models

Gantt uses the translated V8 date parser for native fallback strings, attributed
under its BSD license. Browser rendering calls its actual host Date parser.
Firefox and WebKit differ from Node on 883 and 1,980 legacy-date corpus inputs;
those cases compare against the same pinned Pele library running in that browser,
with the same host resources. This is host parity, not fixture date injection.

Native civil arithmetic remains proleptic Gregorian Rust. Swift supplies timezone
offsets through the additive v3 C resource interface, preserving historical seconds;
Rust resolves gaps forward and repeated times to their first occurrence. Foundation
calendar extraction was rejected after ancient-date corpus mismatches. Forty public
scheduling scenarios pass in UTC, New York, Kolkata, Lord Howe and Apia. Host-error
identity, invalid calendar data and recovery also pass through the Swift bridge.

State retains deeply nested documents with iterative clone/drop. Eight State model
projections exceed the bounded JSON view; typed models and exact renders remain
covered. Sequence retains its deep menu JSON and shared actor boxes without the
expanded foreign projection. One deep-menu projection is explicitly bounded.
Two State inline-note cases and two Sequence menu-link cases produce isolated
UTF-16 output surrogates in upstream from otherwise well-formed input. Their exact
code units and upstream output hashes are recorded in exclusion ledgers. They do
not count as passing public string/API cases. The required mutable/lossless API
architecture is [LLD_PUBLIC_MODELS.md](LLD_PUBLIC_MODELS.md).

## Remaining acceptance work

- Three family integrations, remaining shared behavior and the complete
  registration/lazy/mount/zoom/public model interface remain required.
- JavaScript model projections currently do not apply mutations back into the
  retained Rust document. Prototype methods/private enumerable state also need
  explicit compatibility decisions; full host parity is not claimed.
- Lone UTF-16 surrogate transport remains unresolved. Radar records two and
  quadrant three and mindmap seven unrepresentable mutations explicitly; they are not passing cases.
  Init-JSON's explicit 64-level work cap, extreme radix rounding and non-finite/
  negative numeric host budgets also require broader conformance.
- The 16 MiB JSON projection cap is an explicit transport extension. Class also
  guards deep repeated namespace projections. Large typed
  models/source rendering work without that expanded projection. Mindmap also
  preflights depth 48 before recursive model encoding. Its depth 80, 310 and 3,000
  source renders match exact SVG hashes in Rust, Swift/C and all browser engines.
- No new CI run, Intel execution, GPU acceleration or full-parity release is asserted.
  Initial four-family Chrome timings show slower current WASM public calls than
  Pele JS; see `PERFORMANCE.md`. Rust/GPU choices are not speedup evidence.

Development Swift APIs require source-built native artifacts. Root SwiftPM using
old v0.1 downloaded binaries excludes newer symbols. WASM needs a source build.
No replacement binary is substituted under the old release tag.


## All 33 renderer checkpoint

Venn, Ishikawa and Swimlane complete the family renderer inventory. Eight new
release-profile Rust integration slices pass; their 53,112 cases also pass through
Swift/C owned documents and source rendering, and Chrome, Firefox and WebKit WASM.
The targeted Swift corpus run takes 57.469 seconds. Three additional browser
checks decode and paint real SVG, and AppKit decodes all 33 family examples.
These targeted batch results do not replace the final consolidated release run.

Swimlane retains the real shared Flowchart model (`model.type === 'flowchart'`)
and dedicated lane renderer (`result.type === 'swimlane'`). Map hydration follows
the retained module, preserving this distinction. Ishikawa's four deep models
explicitly exceed JSON projection depth; their source render remains tested.

Venn uses translated circle intersections, overlap fitting and text geometry.
Pinned numeric oracle is Node 25.9 on darwin arm64; explicit fused operations and
fdlibm/V8 math preserve the tested native/WASM results. Required Sun/fdlibm notices
ship with Rust, Swift and web packages. Three finite decimal inputs overflow Venn
geometry into NaN, preserved by result-number transport rather than rasterized.
A Rust-only ceiling rejects more than 3,000,000 inferred pair iterations. The
2,450-set case verifies pinned parsing/model plus the native limit; its upstream
render was deliberately not executed. This one error is counted in the corpus
total as a platform extension, not an upstream render/error parity claim.

All public API gates remain open: mutable model fields/identity, real methods and
prototypes, lossless source/options transport, live registry, lazy/mount/zoom,
final independent consumers and distribution checks. The v0.1.0 release remains
the earlier limited flowchart API. No new release or GIF is published here.


## Mutable native model layer

Nine families now opt into `PublicDiagramModule`: Pie, Info, Sankey, Packet,
Radar, Quadrant, Treemap, Journey and XYChart. This is an additive native layer;
the immutable typed model API and released v1 contract remain available. Flat
model arenas preserve object IDs, aliases/cycles, Map/Set/Date values, absence,
undefined, UTF-16 strings, IEEE number bits and data-property flags. Typed native
module implementations own mutations and geometry. Existing documents retain
their registered codec/configuration after registry changes.

The three method-bearing codecs use explicit schemas; field-only codecs retain
opaque descriptors pending full inventory. Schema tables declare field, prototype and method IDs without recovering
contracts from serde projections. Journey exposes six actual methods, Quadrant
thirteen and XYChart thirteen. Calls publish the post-call graph and revision
even when a JavaScript or Pele operation error follows partial edits; stale or
invalid calls reject before dispatch. UTF-16 Pele errors preserve message, type
and snippet without lossy conversion, while ordinary errors keep their old form.
Custom extension integration tests exercise the same retained native dispatcher.

This layer does not certify foreign mutable APIs or complete model parity.
Constructors, arbitrary getter/proxy/coercion and overridden callback behavior,
active lone-surrogate rendering, sparse active collections in some codecs and
render-time JavaScript error categories remain explicit capability gaps. Native
work/projection ceilings also differ from unbounded JavaScript behavior. The
Swift/C and WASM mutable adapters are a separately verified layer in progress.

The frozen nine-codec native batch passes 57 release-profile integration tests
across 14 targets, including retained extension contracts and Packet/Quadrant
source-corpus regressions. Rust formatting and staged whitespace checks pass.


## Mutable host adapters checkpoint

The same nine native codecs now have additive C v4 owned handles, Swift
`parseMutable`/graph/schema/invoke APIs and browser arena hydration with real
prototype methods. Swift's six mutable-document integration tests pass, including
actual native method calls, partial errors, lossless extra values, nine edited
family SVG decoders and callback recovery. Strict C linkage/ownership tests pass
with 200 repeated documents, revision/input rejection and callback reentry.

Across Chrome, Firefox and WebKit, the 51 selected browser checks are covered by
the initial batch plus an 18-check focused correction run. Full nine-family source
and model corpora, changed model fields, method prototypes/arity/function shape,
shared function identity, partial failures, retained aliases, native SVG pixels
and browser Canvas resources are exercised. The corrections preserve Pele's
RangeError-to-limit guard, retain falsy host throws, align Firefox function key
order and render Packet's formerly oversized model through the flat graph.
Treemap's previous projection exception also renders through the new flat model.

JSON serialization bounds public output while writing, rather than allocating an
intermediary tree first. Native transport and post-call export limits remain
explicit boundaries. The TypeScript consumer slice passes strict checking with
TypeScript 4.9.5; this is a recorded local consumer check, not a CI compiler claim.
No full mutable API/release parity claim is made; see `MUTABLE_MODELS.md`.


## Twelve mutable native codecs

Kanban, Mindmap and Timeline add native mutable codecs, bringing this layer to
twelve families. Kanban and Mindmap expose their real own `getType` callable and
builder methods; Timeline exposes eight prototype methods, including its exact
`getCommonDb()` receiver identity. Sankey now exposes its two real methods and
lookup behavior, including partial-failure cache updates. Ordered own fields,
private stacks, aliases, detached values and data-property descriptors remain
part of the exported graph rather than a normalized JSON projection.

An additive shared-function descriptor identifies Kanban and Mindmap's imported
outline helper across schemas. Existing extension schemas remain source
compatible; registration validates aliases and retained documents keep their
original handler. Foreign adapters still expose the previously verified nine
codecs until their separate host checkpoint passes.

The frozen native batch passes 46 release-profile integration tests across
eleven targets, including all four affected historical corpora and shared
registration contracts. Native mutation SVG vectors and bounded-work recovery
are included. Constructors, active lone-surrogate rendering, arbitrary callback/
coercion behavior and each codec's documented collection limits remain open.


## Sixteen mutable native codecs

Flowchart, Swimlane, ER and Architecture add four more native codecs. Flowchart's
nineteen actual methods and private caches are reused by Swimlane with the real
shared schema, prototype identity and lane renderer. ER exposes ten methods with
entity/attribute/relationship aliases and ordered private group claims; Architecture
exposes six with real Map values, copied records and retained layout-hint arguments.

The frozen native batch passes 41 release-profile integration tests across eight
targets, including the complete affected source/render corpora, Flowchart and
Swimlane's pinned full graph/method vectors and native raster checks. All sixteen
codecs remain subject to their explicit constructor, callback/coercion, UTF-16
rendering and work-budget limits. The separately verified foreign adapter
checkpoint remains nine families while the expanded host checks run.


## Sixteen mutable codecs through Swift and web

The expanded C registry and WASM typed-model conversion expose the same sixteen
mutable codecs. Swimlane selection precedes the shared Flowchart model downcast,
so models keep the common FlowDb prototype while rendering real swimlanes. Shared
callable metadata makes Kanban and Mindmap's imported helper one host function;
disposing a later document does not break earlier live borrowed calls. Host
registrations retain weak document owners without retaining native handles.

The current source-built arm64 C archive passes strict C ownership/transport
checks. Eight Swift feature integration tests pass, including all sixteen edited
SVG decoders, actual Timeline operations/partial errors, shared metadata and
Swimlane's retained renderer. Chrome, Firefox and WebKit pass fifteen model/method
checks, twenty-four affected full-corpus checks and three decoder/pixel checks.
All three former deep Mindmap projection exceptions now render through the flat
public graph in each browser. The strict TypeScript 4.9.5 consumer also passes.

These local results cover the frozen adapter layer. They do not assert final CI,
Intel execution, complete mutable/API parity, new package publication or release.


## GitGraph mutable native checkpoint

GitGraph is the seventeenth native mutable codec. Its actual returned value is a
plain object with no model prototype methods; no synthetic builder API is added.
Commit/branch Maps, head aliases, present undefined merge fields and own-key order
are retained. The concrete Git renderer consumes the edited graph with the same
23 pinned mutation SVG comparisons and all 17,125 historical outcomes (5,050 SVG
results and 12,075 errors), with zero corpus exclusions.

Eight public feature tests plus two baseline integration tests pass in release
profile. Default iterators, finite/infinite sequence ordering, inactive fields,
UTF-16 extras, native raster/resource recovery and bounded alias expansion are
covered. NaN/nonnumeric sorting, exotic callbacks/coercion, active sparse/isolated
UTF-16 text and some render-error classes remain explicit gaps. The verified
foreign mutable checkpoint remains sixteen families.


## Imported callable construction

Native function-shape descriptors distinguish ordinary constructible functions
from class methods. Kanban and Mindmap's shared outline helper keeps its real
prototype object, constructor backlink, arity and construction behavior; class
methods remain nonconstructible. This closes the imported-helper construction
gap without claiming model constructors. Shape/identity registration validation
is linear and bounded, and retained documents survive invalid replacement metadata.

Three native extension integration tests pass in release profile alongside the
GitGraph batch; all eight Swift mutable feature tests pass with shape metadata.
The consolidated eighteen browser model/method tests pass in Chrome, Firefox and
WebKit. An initial Firefox own-property ordering failure was corrected by retaining
the helper's intrinsic two-argument length rather than rewriting it.


## Public browser error constructor

The development core/full entries export the same `PeleError` constructor, with
Pele's message/code/info defaults, actual Error inheritance, ordered fields,
descriptors and raw UTF-16/IEEE values. `PeleCompatibilityError` remains an alias
for existing catch checks. Native parse/render/method errors and the RangeError
guard produce this same public prototype. Transport decoding is separate from
the public string constructor. Host-generated stack/location values are excluded
from cross-file equality; semantic Pele line/column fields remain checked.

The packed browser package passes all twenty-one error and mutable-model checks
in Chrome, Firefox and WebKit. Explicit class-field declarations preserve native
field ordering and WebKit's enumerable line/column descriptors. The strict
TypeScript 4.9.5 consumer passes. This does not close model constructors or the
remaining native Error-value arena seam.


## AgentFlow and Ishikawa mutable native checkpoints

AgentFlow and Ishikawa bring native mutable coverage to nineteen families. The
frozen batch passes nineteen release-profile integration tests across four
targets, including all 14,990 AgentFlow and 20,586 Ishikawa historical outcomes.
AgentFlow also verifies all 5,373 historical semantic-model results and 114 raw
graph cases/194 operations/226 complete SVG hashes. Its original parsed drawing
graph remains cached: root label/graph edits can stay visually stale, while
detached original style-array edits remain live. Four astral metadata projections
now work through the public identity graph without changing typed convenience
serialization. Retained registration, native pixels and bounded expansion recover.

AgentFlow exposes all thirty-four actual method descriptors. The later returned
Error checkpoint below completes its previously unavailable `shapeError()` native
implementation using a branded identity-arena node. Builder shape failures already
throw real Pele errors. Arbitrary overrides/coercion, options.log callbacks,
active lone-surrogate SVG, some numeric provenance and render TypeError transport
remain recorded runtime gaps.

Ishikawa exposes the real addNode method, stack/baseLevel bookkeeping and aliased
tree. Sixteen pinned wide/narrow mutation SVGs, deep flat round trips, partial
stack failures and native resource/pixel recovery pass. Caller-added accessibility
extras are retained but ignored, matching the actual upstream renderer. No
renderer changes are needed. Active missing/sparse/lone-surrogate node text and
cyclic/over-budget trees retain explicit boundaries. Foreign mutable coverage
remains the separately verified sixteen-family checkpoint.


## Class and EventModeling mutable native checkpoints

Class and EventModeling bring native mutable coverage to twenty-one families.
The consolidated native batch passes thirty-four release-profile integration
tests across nine targets, including source baselines, retained extension methods,
shared callables and Venn dimension-transport regressions. Foreign mutable
coverage remains the verified sixteen-family checkpoint.

Class exposes twenty-three root and four nested ClassMember methods, full
namespace/class/note aliases, arbitrary numeric relation markers and note IDs,
and ordered optional fields. Its 29 mutation vectors cover 114 native calls:
104 whole SVG checkpoints plus ten UTF-16 storage/return checks with explicit
active SVG gaps. Forty thousand actual member-builder inputs are checked against
pinned results. The original 24,758-case source corpus remains green. The mutable
lane matches 24,756 cases; two enormous namespace cases hit the 16M-unit graph
transport limit before upstream's 4M-character SVG limit. Exact case IDs, source
hashes and both errors are asserted, rather than counted as parity. The six
legacy projection exclusions remain explicit. Namespace-parent capture, exact
style compaction and a negative lexer-direction cache preserve valid stress
cases. Cross-document lineType/relationType identity remains open.

EventModeling preserves its real plain-object model and shared lane/box/relation
objects. All 6,953 historical outcomes and thirty pinned mutation SVGs pass.
Edited duplicate box IDs now resolve note/spec headings to the last matching
name, matching upstream. Two generated UTF-16 hard-wrap output gaps remain
explicitly outside corpus success. Both codecs retain their documented exotic
coercion/callback, active sparse/UTF-16, render-error and work-budget boundaries.


## Stateful render operation primitive

The additive native `PublicDocument::render_operation` publishes post-render
graph state and advances its revision after dispatched success or failure.
Modules can preserve native JavaScript error categories through an optional
trait override; existing modules retain their implementation by default.
Stale revisions reject before dispatch. Nested results retain exact non-finite
dimension bits. Three integration slices cover retained handlers, partial edits,
callback failure, output limits, recovery and invalid post-operation graphs;
all pass in the thirty-four-test release batch above. The old native render API
remains unchanged. Foreign adapter validation and pre-callback synchronization
remain separate pending work.


## TreeView mutable native checkpoint

TreeView brings native mutable coverage to twenty-two families; foreign
coverage remains sixteen. Its returned plain wrapper preserves the virtual root,
optional own fields and shared node identities without inventing private builder
methods. Seven public integration slices and four source baseline slices pass in
release mode: all 6,800 historical outcomes, twenty-eight pinned mutation/config
SVG comparisons, repeated icon callback traces, deep/DAG graphs, native pixels,
resource failures and recovery. Existing twenty-seven source UTF-16 exclusions
and three JSON projection ceilings remain explicit. Active sparse/lone-surrogate
text, exotic coercion/prototypes and cyclic or over-budget expanded trees remain
recorded gaps. Existing parser, model and renderer sources are unchanged.


## State mutable native checkpoint

State brings native mutable coverage to twenty-three families; foreign coverage
remains sixteen. All eighteen real StateDb getter/method entries operate on its
identity graph and retained caches. Forty-six complete graph vectors cover
eighty-three operations and eighty-eight complete SVGs, including explicit
rebuilds, detached style aliases, direction getters and post-error recovery.
Four public and two baseline release integration slices pass all 10,222 historical
outcomes, including repeated-class/description stress and eight deep source
cases through the flat public graph. Superseded builder scratch objects stay
private; existing 1M-item/8M-work/16M-text budgets are unchanged. The source model
only adds skipped divider-counter provenance. Constructor/override/coercion,
active sparse/nonstring renderer data, lone-surrogate rendering and legacy
render-error category boundaries remain explicit.


## Stateful render operations through C, Swift and WASM

The additive C/WASM render operation now transports post-render graph, revision
and typed success/error without losing non-finite dimension bits. Swift exposes
`renderOperation`; its convenience render also advances revisions. Both invoke
and render publish the returned revision before rethrowing original Swift host
errors. Browser renders reconcile the graph before restoring the original host
throw, including falsy values and the pinned RangeError guard. Legacy C/native
render entry behavior is unchanged. The frozen sixteen-family host checkpoint
passes strict C integration, nine Swift mutable-document integration slices and
twenty-four packed-package checks across Chrome, Firefox and WebKit. This proves
post-call publication; pre-callback synchronization/reentry remains open.


## Venn mutable native checkpoint

Venn brings native mutable coverage to twenty-four families; the separately
committed foreign checkpoint remains sixteen. Nine public and two source
baseline release integration slices pass all 22,121 historical outcomes,
twenty-six pinned edited/method SVG comparisons, undeclared-ID geometry, raw
UTF-16 semantic errors, alias/partial-error behavior and native raster/resource
recovery. All eight real methods operate on known/current state. Skipped ordered
known provenance preserves stale private bookkeeping for typed callers; public
graph edits preserve exact current-array identity. The existing inferred-pair
ceiling remains an explicit native extension, with upstream rendering deliberately
unexecuted. Active sparse/lone-surrogate rendering, nonnumeric singleton sizes
with triple inference, exact render-error categories, exotic overrides and work
ceilings remain open. A typed Vec cannot retain original array identity after
permuting indistinguishable duplicate subsets. The renderer is unchanged.


## Twenty-three mutable codecs through Swift and browser WASM

GitGraph, AgentFlow, Ishikawa, Class, EventModeling, TreeView and State join the
previous sixteen foreign mutable codecs. Strict C integration and ten Swift
feature slices pass, including actual macOS image decoding and native method
dispatch. The packed browser package passes twenty-seven feature/error/operation
checks across Chrome, Firefox and WebKit. Eighteen corpus checks pass for six
families across all three browsers. The separate three-browser Class rerun
verifies its corpus plus four exact transport boundaries: the two known native
16M-unit graph differences and two additional 64 MiB foreign JSON export ceilings
for valid large style models. Source hashes, exact errors and source SVGs are
checked; these differences do not count as public-model parity. The first corpus
run exposed the two foreign ceilings rather than silently accepting them.

Raw prototype descriptors, nested ClassMember calls, private Sets/cache aliases,
State direction/rebuild behavior, detached AgentFlow styles/semantic-model results
and edited SVGs match the pinned probes. The existing AgentFlow query convenience
now calls its actual native semanticModel method. TypeScript 4.9.5 consumer checks
pass with mutable fields and method declarations. Native Venn remains the next
foreign checkpoint. Binary graph exchange is still required by the public-model
LLD and will replace JSON expansion; source/options and runtime gaps remain open.


## Native binary graph exchange primitive

`encode_binary_exchange`/`decode_binary_exchange` provide bounded little-endian
flat identity tables, revision and opaque metadata, following the public-model
LLD. Counts are checked against remaining bytes and native work ceilings before
allocation; decode checks references/flags and rejects trailing/truncated input.
Raw UTF-16, IEEE payloads, Dates, sparse arrays, builtins, host tokens, aliases and
cycles survive without JSON expansion. Three native feature integration slices
plus two original Class baseline slices pass in release mode. Both valid Class
style models blocked by foreign JSON export now fit the native binary lane under
the unchanged 64 MiB/default graph limits and render exact pinned SVG hashes.
Foreign adapters still use their separately verified JSON checkpoint until the
next binary-binding slice passes. This does not close source/options transport
or general JavaScript object/runtime parity.


## Sequence mutable native checkpoint

Sequence brings native mutable coverage to twenty-five families; the committed
foreign checkpoint remains twenty-three. All seventeen actual methods operate on
private lifecycle/wrap state, actor/box/Map aliases and exact own fields. Five
public and four original release integration slices pass all 19,166 historical
outcomes, including forty-eight stress cases. The reproducible oracle covers
135 full-graph cases, 202 operations and 254 complete SVGs. Raw UTF-16 JSON menus
use a bounded native iterative parser; two old menu-link surrogate exclusions
and one deep source projection now pass through the public identity graph.
Actor.box aliases remain independent of root boxes edits. Native pixels, retained
ownership, deep/cyclic extras and host/output recovery pass. Malformed dividers
produce a typed TypeError through render_operation instead of a renderer panic;
legacy errors/preflight callback ordering remain explicit boundaries. Active
sparse/lone-surrogate rendering, exotic overrides/coercion and numeric renderer
kinds/lifecycle indexes outside typed ranges remain open. Renderer unchanged.


## Gantt mutable native checkpoint

Gantt brings native mutable coverage to twenty-six families. Seven public and
two source baseline release integration slices pass all 18,994 historical
outcomes with zero exclusions. Thirty-five pinned mutation vectors cover
113 calls, 102 complete SVG hashes, six exact render errors and five explicit
active UTF-16 output gaps. All thirty-one real methods use a shared native
scheduler; task/Date/returned-array identities, detached byId tasks and partial
failures remain live. Numeric compact order and repeated task references retain
upstream layout identity. Public calendar scheduling stops at the first host
error without changing immutable calendar behavior. Temporary lookup strings
stay outside the retained arena, preserving valid stress behavior and budgets.
Constructor/custom helper/callback overrides, active lone-surrogate scalar output
and pre-callback host synchronization/reentry remain separate open gates. Foreign
Gantt exposure is pending; the committed foreign checkpoint remains twenty-three.


## Binary model exchange through C, Swift and browser WASM

Additive v5 C operations and WASM binary operations carry graph identity tables
plus revision and operation metadata. The standard Swift/browser adapters now use
this lane for snapshot/apply/invoke/render; legacy v4 JSON operations remain.
Strict C integration, twelve consolidated Swift feature slices, thirty packed
browser feature/ownership/error checks and six three-browser Class/Sankey corpus
checks pass. Both valid large Class style models now round-trip and render exact
pinned SVGs in Swift and Chrome/Firefox/WebKit under unchanged byte/work limits.
The two foreign JSON-export differences from the prior checkpoint are closed.
The two separate native Class graph-budget differences remain explicit.

Raw UTF-16, cycles/aliases, descriptor flags, builtin/host references, IEEE
payloads, non-finite rendered dimensions, stale/malformed inputs, native callback
identity and recovery retain their native contracts. Counts are bounded before
allocation in every adapter. Source/options transport, generic canonical graph
validation and complete JavaScript runtime/constructor compatibility remain open.

## C4, Requirement and Usecase native mutable checkpoints

Native mutable codecs now cover twenty-nine families. C4 exposes all nineteen
actual prototype methods and preserves parser-private indexes, repeated shape
and boundary aliases, last-layout lookup identity and exact host measurement
order. Nine public and two original release tests pass, including all 18,603
historical outcomes, 32 mutation SVGs and two partial-error drawings. Existing
18 deep/cyclic JSON projection exclusions retain flat graph/render proof.

Requirement exposes seventeen methods and sixteen ordered own fields, including
six pending builder strings and private style-copy history. Seven public and
three original release tests pass, including all 10,358 historical outcomes,
25 pinned vectors, 102 calls, 93 SVG hashes and sixteen exact method exceptions.
A 100,000-style-copy feature remains bounded without quadratic descriptor scans.
Nine active rendering differences remain explicit (five UTF-16, four nonscalar).

Usecase returns the actual plain object without invented builder methods. Five
public, three original and nine Sequence regression tests pass after sharing the
bounded UTF-16 JSON arena parser. The source corpus remains 16,744 cases; mutable
export matches 16,741 with three explicitly hashed budget differences: two huge
independent style/class expansions and authored JSON-pointer text. Source render
outcomes remain tested for all three. Ninety-six graph cases, 98 edits and 188
SVG hashes cover JSON authored order, holes, aliases, raw UTF-16 and IEEE values.
An 8,000-by-8,000 shared style-array import retains identity without allocating
64 million duplicate strings. Existing native budgets are unchanged.

These checkpoints do not certify full runtime parity. Constructors, custom
callbacks/coercions, several active sparse/nonnumeric fields and lone-surrogate
SVG output remain explicit shared or family-specific boundaries. Native source
codecs and foreign Swift/web exposure are counted separately.


## Twenty-nine Swift mutable families and returned Errors

The C registry and Swift wrapper now expose the twenty-nine completed native
codecs. Strict C integration and seventeen Swift feature slices pass: twelve
existing slices plus five new slices covering the six added families, all eight
Error brands, returned AgentFlow PeleErrors, Gantt Date scheduling/identity and
post-scheduling Swift callback failure/recovery. Swift renders decode through
AppKit and match the source lane; extensions retain cycles, raw UTF-16 and NaN
payloads. All 34 native AgentFlow methods are implemented.

The additive binary arena Error tag keeps own fields/descriptors and stable
references separate from thrown operation errors. Generated host stack/location
diagnostics remain runtime-local. Native release proof includes four AgentFlow,
four original AgentFlow, three binary and two returned-Error tests. Existing
exhaustive integration canonicalizers now retain Error brand metadata.

Full mutable Swift corpus verification is a separate pending gate. Set
`PELE_VERIFY_MUTABLE=1` when running the pinned AdditionalDiagramTests corpus;
`PELE_FAMILIES` optionally selects families. This checks independent source versus
mutable drawings, graph edits/cycles/UTF-16, exact error fields and ownership.
Five pre-existing native graph-export budget differences are source-hash pinned.
Full constructor/callback/active UTF-16 parity and release readiness remain open.


## All thirty-three native mutable codecs

Block and Wardley complete the source-built native codec inventory. This is
module/API implementation coverage, not full JavaScript runtime certification.
`PublicEngine::with_builtins()` provides the same extensible typed registration
interface for all 33 codecs; the C/Swift parse entry uses this central registry.
`empty()` plus `register()` remain available for custom modules/replacements.

Block passes ten public and three original release integration tests, including
all 12,294 historical outcomes (4,584 SVGs, 7,710 errors), nineteen edited/method
SVG checks and five real own helper functions plus six prototype methods. Three
pre-existing deep JSON projection omissions retain actual parse/render proof.
Parser counters/private collections, original array aliases, iterative hierarchy
building, partial-limit state, accessor metadata and descriptor-aware lengths
remain covered without changing original immutable SVGs.

Railroad passes six public and two original release tests, all 12,288 historical
outcomes, 27 mutation vectors/71 calls/63 SVG hashes and eight exact measurement
traces. Its bounded identity-aware breadth-first budget pass matches shared IR
objects at different depths. Thirty-three original JSON-depth projection omissions
retain source render/error checks; active callback/UTF-16 boundaries remain explicit.

Cynefin passes four public and two original release tests, all 7,209 historical
outcomes and 78 graph vectors/54 edits/146 SVGs. Wardley passes four public and
two original release tests, all 7,730 historical outcomes and 99 graph vectors/
82 edits/182 SVGs, including non-finite output dimensions. Both expose the actual
plain object; discarded internal builder methods are not fabricated. Native
pixels, aliases, retained ownership, reflow, budgets and recovery are tested.

The six prior additions (Venn, Sequence, Gantt, C4, Requirement, Usecase) passed
105,986 scenarios through the immutable Swift corpus plus actual mutable C/binary
snapshot/edit/render/error crossings. All three declared Usecase export budget
differences remain exercised. Final unfiltered 33-family native/Swift consolidation
and separate packaged consumers are pending below this checkpoint.

A clean checkout without local artifacts also builds and runs the published
v0.1.0 binary consumer. The new PublicBinary.swift is excluded alongside other
source-only APIs on that legacy path. Current source-built Swift retains all new
APIs. Two actual-C XYChart feature slices verify array append/truncate/grow,
descriptors, stale edits and recovery; plain-family declared-title edits compare
against equivalent source output rather than only unused extension properties.

The development performance harness in `scripts/perf` verifies all 21 workloads
against pinned complete SVG hashes and reports native stages, JS references,
allocations and transport independently. The initial shared-host baseline and
warmup caveats are recorded in its results directory. No runtime optimization or
general Rust/GPU speedup claim follows from this first baseline.

## Consolidated native verification

The frozen all-33-codec core at `cbad8d4` passes one consolidated release run:
326 integration tests across 75 test binaries, including original family corpora,
all mutable codecs, shared methods, binary exchange, returned Errors and stateful
render operations. `cargo package --locked --allow-dirty -p pele-rs` verifies the
actual packaged library; the separate Rust consumer then links that unpacked
package and parses, edits, renders and snapshots every one of the 33 mutable
families. Raw UTF-16 extension fields survive the consumer's graph round trips.
This is native/package proof; final Swift and universal/Intel results remain
separate gates.

## Consolidated Swift and universal SDK verification

The actual source-built C/Swift SDK passes 26 release integration tests in
1,598.349 seconds. The unfiltered corpus crosses all 33 mutable families:
167,473 drawing checks, 270,358 error checks and five explicitly pinned native
export-budget differences. The original immutable/source oracle checks remain
independent. These counts include the declared differences; they do not erase
them or certify arbitrary JavaScript runtime behavior.

Three additional actual-C integration tests close the five stress cases outside
the main fixture arrays: Packet's shared 40,000-unit label over 10,000 rows,
Mindmap depths 80/310/3,000, and Flowchart's 47,917-edge parse. They pass in
1.717 seconds, including binary snapshot/apply, graph counts, source-equivalent
rendering where the fixture defines it, raw UTF-16 extras and cyclic identity.
The Packet check never expands the shared labels into a 400 MB JSON model.

Fresh Rust arm64 and x86_64 archives form one verified universal XCFramework.
Both slices match their target build inputs byte-for-byte; both linked consumer
executables declare macOS 14.0 minimum deployment. The host is macOS 26.6;
Rosetta execution does not substitute for a physical Intel or macOS 14 runtime.
Separate copied SDKs pass
nine feature tests on arm64 and nine under actual Intel Rosetta execution, and
independent consumers parse/edit/render/snapshot/close all 33 families on both
architectures. The consumer's State source comparison preserves title-looking
DSL statements: pinned Pele proves those are ordinary state grammar, so
rewriting them would change the diagram. No library fix or relaxed assertion
was needed.

The universal library SHA-256 is
`1a7f5b670b6d0ccdc57c605bf6014aecb6fd95984cbf8c59fe4268184210833f`;
its arm64 slice is
`f8c8f8bb4939bb815813add7990c2ab1d3bcc6bfa0af7cb91c81f8934783e621`,
and its Intel slice is
`77122bf7b644b49e1ff8e29bd98d835f4e2119a15f639f128452df7e9916596e`.
The verified universal SDK replaces the local generated arm64-only artifact;
generated binaries remain excluded from Git. CI now requests these consolidated
Rust and Swift checks in release configuration. A final green hosted CI run,
new browser mutable adapters and the documented full-runtime/API release gates
remain separate from this native/Swift checkpoint.

After the three mechanical module/import ordering fixes at `9259e35`, both
explicit-target release builds succeed and produce byte-identical universal
archives/slices. The current unchanged 33-family consumer passes again on arm64
and Intel Rosetta. Existing full corpus and per-architecture SDK checks therefore
retain the exact same library bytes. `cargo fmt --all --check` also passes.

The separate Swift performance consumer now passes 105 correctness checks
(21 shared workloads × five actual API stages), exact pinned output fingerprints
and raw UTF-16/cycle graph checks. A short coordinated timing window records
parse/snapshot/apply/render/lifetime costs for three medium workloads. It keeps
intrinsic C/binary/publication/disposal costs inside each API measurement and
records shared-host load and Gantt's retained Date history. Source, executable
and SDK hashes match before/after. A controlled OS process check verifies the
build/test guard rejects an active owned process; provisional samples from an
earlier process-name parser are excluded. See `scripts/perf/results/2026-10-18-swift-api.md`.

## Exact numeric source limits through native and Swift

The additive `SourceLimit` primitive now preserves undefined/default, null,
fractional and nonfinite thresholds in the Rust inspection/parse interfaces and
the C v6/Swift mutable-document parse path. Comparison uses the original UTF-16
source length before preprocessing or detection. Legacy integer APIs retain their
contract. Config, edge limits, extension dispatch, handle ownership and binary
graph operations retain their shared implementations. An independent 16 MiB
source-byte ceiling remains active even for NaN or positive infinity.

The pinned 207-case oracle contains 54 complete model/SVG successes and 153
complete error outcomes. Nineteen focused Rust release integration tests pass,
including extension registration, legacy behavior, exact error formatting,
UTF-16 ordering and native byte boundaries. Strict C source-limit, mutable and
legacy integrations pass. Sixteen Swift release integration tests pass, covering
the complete new matrix plus existing document and preparation behavior.

Fresh universal artifacts also pass separate arm64 and Intel Rosetta checks:
three source-limit integration tests (all 207 cases), the actual C v6 integration,
and independent immutable/mutable consumers across all 33 families on each
architecture. The new universal archive SHA-256 is
`b9bbb51696fb744ac017c848f71a1e25d2395efb89571f008f4dccf2ae6f5455`.
These runs use macOS 26.6 and declare minimum macOS 14.0; physical Intel/macOS 14
runtime verification remains separate. A fresh artifact-free Swift consumer
downloads the unchanged checksum-pinned v0.1 archive and builds/runs its legacy
flowchart API, with modern source-limit files excluded as intended.

This closes only exact number-valued source limits for well-formed input.
Numeric edge/output limits, malformed UTF-16 source/output, arbitrary coercions,
constructors, callback dispatch and other full-runtime/release gates remain open.
See [source-limit contract](SOURCE_LIMITS.md). The earlier full corpus and
performance results retain their recorded revisions and artifact fingerprints;
this additive slice uses the focused verification above.

## Exact numeric output limits through native and Swift

The additive `OutputLimit` lane preserves Pele's nullish 4,000,000-unit default,
direct IEEE threshold comparison and final SVG UTF-16 length. Rendering and host
callbacks precede the semantic output check; mutable operations publish their
advanced graph/revision even on output or callback errors. Existing integer
entry points keep their contract. Explicit provider hooks let extensions retain
their concrete models and override private legacy limit logic. Class shares its
geometry while separating old integer preflight from exact post-render policy.
Its optional native byte lower bound uses emitted record IDs, not Map keys, and
defers authored-ID decoding errors to normal rendering so callbacks keep their
order. Independent 64 MiB returned-SVG and bounded metadata/exchange checks stay
active for NaN and infinity; this is not a universal peak-allocation guarantee.

The pinned 192-case oracle has 96 whole SVG successes and 96 complete Pele error
outcomes, plus two host-failure ordering vectors. It records complete measurement
traces and mutable state, including Gantt scheduling and Date/task aliases.
Immutable JSON and expanded mutable graph projections are asserted separately:
the historical immutable Set projection remains `{}`, while the public arena
retains the actual Set entries. Twenty-seven focused Rust release integration
tests pass. Tests additionally cover retained extension registration, empty SVG,
byte-budget rejection and the two Class preflight regressions. Strict C output,
source-limit, mutable and legacy integrations pass, including 600 existing
legacy/owned-document lifetimes.

Nineteen Swift release integration tests pass through the actual universal SDK.
Separate arm64 and Intel Rosetta SDK runs each pass six limit-feature tests,
all 399 source/output-limit cases, both callback-order vectors, actual C output
integration and independent immutable/mutable consumers across all 33 families.
The universal archive SHA-256 is
`972ffb6de07b8bd3742ab7cd8c0a0dea3bfe4f51ff24b2fa5d4cb58f2bbd9961`;
its tested arm64 slice is
`97a0a021194a1bbdb8430c5305fb67975d71546e4f510827d9aaea240fe0c54e`.
The host remains macOS 26.6; Rosetta is not physical Intel/macOS 14 proof.
An artifact-free consumer also builds/runs the unchanged checksum-pinned v0.1
download with the new Swift file correctly excluded. Formatter and diff checks
pass. See [output-limit contract](OUTPUT_LIMITS.md).

This closes exact numeric output limits for well-formed strings. Numeric edge
limits, arbitrary options coercion, malformed UTF-16 source/output, constructors,
callback dispatch and the other full-runtime/browser/release gates remain open.

## Composable exact source and edge limits through native and Swift

`ParseLimits` independently selects exact source and edge overrides while
retaining omitted legacy integer fields. Flowchart, AgentFlow and Swimlane share
their existing parsing paths with an explicit private policy. Undefined selects
5,000 edges; null compares as zero; fractional thresholds compare the existing
count before each push. NaN and positive infinity disable that semantic check.
The exact lane retains an independent 16 MiB source-byte budget and incremental
1,000,000-edge generation ceiling. Semantic rejection still precedes the native
generation check. The native ceiling is an explicit platform extension, not an
upstream error-parity result.

Additive provider hooks preserve concrete models, registration retention and
public struct layouts. The parsed document retains its explicit policy privately;
mutable graph creation seeds exact `options.maxEdges` before deriving state.
Thereafter graph options remain authoritative, including deletion/undefined and
partial Cartesian method failures. New C v7 and Swift `ParseLimits` APIs carry
independent tags and IEEE bits without weakening unrelated option decoding,
handle ownership, revision checks or transport budgets.

The pinned edge oracle contains 313 cases: 139 complete SVG successes, 171 full
errors and three parse-only successes. It also records 24 method steps across
the three families, covering partial commits, collection aliases and recovery.
Thirty-one focused Rust release integration tests pass together; the additional
million-edge Cartesian integration passes once in isolation. The combined slice
is 32/32. Five strict C integration programs pass: composable parse limits,
output limits, source limits, mutable operations and legacy ownership. All 22
focused Swift release integration tests pass against the fresh universal SDK.

Separate arm64 and Intel Rosetta runs each pass nine limit-feature tests: all
712 source/output/edge cases, 24 method steps and two callback-order cases. Each
also passes the actual C v7 integration and an independent immutable/mutable
consumer across all 33 families. The universal archive SHA-256 is
`1293ce6887a50aa45713a91dd1e18e79e6241557c9386eb3f85e71664ee4190f`;
its arm64 slice matches the separately verified archive byte for byte. The host
is macOS 26.6; Rosetta is not physical Intel/macOS 14 runtime proof. A fresh
artifact-free Swift consumer also builds/runs the unchanged checksum-pinned
v0.1 archive with both new Swift files correctly excluded.

The earlier full-corpus and performance results retain their recorded revisions.
This additive slice closes numeric parse policies for well-formed strings;
arbitrary coercion, constructors, active malformed UTF-16, callback dispatch and
the remaining public-runtime/browser/release gates remain open. See
[composable parse-limit contract](PARSE_LIMITS.md).


## Owned selective native registry

The source-built Rust/C/Swift interface now owns selective registries through the
existing typed `PublicEngine`; there is no second module system. New
`register_builtin` dispatch covers all 33 canonical types, and `with_builtins`
uses the same path. Membership/capabilities/discovery query that owner. Detection
preprocesses without grammar or the default source threshold; supports converts
preprocessing failure to false. Byte-budget/UTF-8 transport failures remain errors.
Unknown foreign registration names reject without mutation. Empty-registry parse
preserves the upstream unsupported-type error.

C v8 owns a try-locked engine, publishes handles after encoding metadata and shares
v7's independent parse-limit decoder. The non-Sendable Swift `Pele.Registry`
provides empty/all-builtins factories, explicit closure and existing owned mutable
documents. A document retains its concrete provider, schema, config and methods
after replacement or registry disposal. Existing Engine convenience APIs and the
published legacy fallback retain their behavior.

The portable oracle regenerates byte for byte: five independently isolated core/
full registries, 240 source discovery checks, 85 parse outcomes and three retained-
model provenance vectors. Eighteen focused Rust release integration tests pass,
including custom-provider/schema/method/config/drop retention. The previously
verified million-edge case was deliberately excluded from this registry batch;
no edge-generation code changed. Strict C registry and v7 integrations pass on
the current native archive. The artifact-free, checksum-verified v0.1 Swift
fallback also builds and runs with Registry.swift excluded.

Twenty-five focused Swift release integration tests pass against the fresh
universal SDK. Independent arm64 and Intel Rosetta copies each pass 12 registry/
limit-feature tests, four strict C integrations and immutable/mutable consumers
across all 33 families. The universal archive SHA-256 is
`4fb2e8fb065c2054651846e974587a07196bfb2d101fffe04b8dd0baa3b3fd59`;
its arm64 slice is byte-identical to the independently tested archive
`9b7cfe5a9d5a8cd2fcc0f86692de048ff9de0a898586a18d9fda4a5d0d78260d`.
The packaged header also matches current source. The tested host is macOS 26.6;
Rosetta is not physical Intel/macOS 14 runtime proof. Formatter and diff checks
pass. The contract, ownership rules and remaining boundaries are recorded in
[owned registry architecture](LLD_NATIVE_REGISTRY.md). Arbitrary foreign callback
providers, live JavaScript Diagram changes, constructors, active malformed
UTF-16, callback/coercion execution and browser/release gates remain separate.
