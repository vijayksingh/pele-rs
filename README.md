# pele-rs

[![Conformance](https://github.com/vijayksingh/pele-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/vijayksingh/pele-rs/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/vijayksingh/pele-rs)](https://github.com/vijayksingh/pele-rs/releases/latest)
[![MIT](https://img.shields.io/badge/engine-MIT-blue)](LICENSE)

**Pele-compatible diagrams, one independent Rust engine for Swift and the web.**

**The engine implements all 33 diagram families** from
[Pele 1.6.0](https://github.com/obsidianmd/pele). Rust owns parsing, models,
layout and SVG generation. Thin C/Swift and WebAssembly adapters expose the same
engine, with host font, icon, clock, calendar and presentation resources.
The project has no Kamui dependency and runs no JavaScript diagram engine in its
native or WASM distribution.

Current development source targets **0.3.0**. Build from source below. The former
v1-only engine and wrappers are removed. A new release remains on hold while the
web performance target is unmet. Packages are distributed through GitHub, not
crates.io or the npm registry.

![Packed Rust WASM demo: five diagram families, mutable pie, error recovery and dark palette](docs/images/full-demo.gif)

Recorded from the actual packed-library playground with
[asserted interactions](scripts/record-full-demo.mjs). The
[playground](examples/web/full.html) uses the same WASM engine as the package.

## Diagram families

All 33 have Rust parsers/renderers and mutable model codecs:

| | Families |
| --- | --- |
| Graphs and structure | Flowchart, Swimlane, Class, ER, State, Architecture, Block, Mindmap, TreeView |
| Time and workflow | Sequence, Gantt, Timeline, Journey, Kanban, GitGraph, AgentFlow, EventModeling |
| Charts and data | Pie, QuadrantChart, Packet, Radar, XYChart, Sankey, Treemap, Venn |
| Systems and explanation | C4, Requirement, Railroad, Cynefin, Wardley, Usecase, Ishikawa, Info |

Compatibility is pinned to upstream revision
`458322a5b8abb1ae57ddf127c32017ab9fb14a92`. Implemented families do not by
themselves establish complete API or release acceptance. The
[parity inventory](docs/PELE_PARITY.md) tracks the complete library, and
[provenance](UPSTREAM.md) records its relationship to upstream. This is an
independent project, not an official Obsidian, Pele or Mermaid release.

## Install

### Rust

The Cargo package is named `pele-rs`; the dependency below aliases it as `pele`.
Use the source checkout for the current API:

```toml
[dependencies]
pele = { package = "pele-rs", path = "/absolute/path/to/pele-rs/crates/pele" }
```

```rust
use pele::compat::{Engine, RenderOptions, text::MetricsResources};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::with_builtins();
    let drawing = engine.render_source(
        "pie\n\"Swift\":2\n\"Web\":3",
        &RenderOptions::default(),
        &mut MetricsResources,
    )?;
    println!("{}", drawing.rendered.svg);
    Ok(())
}
```

`MetricsResources` supplies Rust's fallback measurement. Hosts can implement
`RenderResources` for their actual fonts and resources. Use `Engine::parse` and
`render_document` to retain a parsed document. The separate
`compat::public::PublicEngine` exposes owned mutable graphs, schema-described
methods and direct model construction.

### Swift

macOS 14+, Swift 6+. Run `scripts/build-native.sh --universal` before resolving
the local package. In the application's `Package.swift`:

```swift
// dependencies:
.package(path: "/absolute/path/to/pele-rs")
// Target dependencies:
.product(name: "Pele", package: "pele-rs")
```

```swift
import Pele

let engine = Pele.Engine()
let document = try engine.parse("pie\n\"Swift\":2\n\"Web\":3")
let drawing = try engine.render(document)
// drawing.svg, drawing.width, drawing.height, drawing.links
```

Use `parseMutable` for editable owned models, or `parsePrepared` /
`createPrepared` for low-level per-family models. The AppKit layer supplies
`DiagramView`, mount/update/destroy, zoom, link/accessibility controls and
explicitly approved native image resources. See
[native entries](docs/NATIVE_DIAGRAM_ENTRIES.md),
[mutable models](docs/MUTABLE_MODELS.md) and
[Swift build details](bindings/swift/README.md). No iOS or Windows binaries are
included.

### Web

Build the current package:

```sh
scripts/build-wasm.sh
npm install /absolute/path/to/pele-rs/dist/release/unlocalhosted-pele-rs-0.3.0.tgz
```

```js
import {loadRenderer} from '@unlocalhosted/pele-rs/full';

const renderer = await loadRenderer();
const model = renderer.parse('pie\n"Swift":2\n"Web":3');
model.sections.set('Rust', 5);
const drawing = renderer.render(model);
document.querySelector('#diagram').innerHTML = drawing.svg;
renderer.dispose(model);
```

Serve over HTTP. Initialization is asynchronous; parse/render are synchronous
afterward. `loadRenderer(wasmURL)` accepts a relocated WASM URL. The package also
exports `./core`, independent `./registry`, `./lazy`, and all 33 `./diagrams/*`
entries. Low-level entries accept parsed or caller-authored models after
`loadDiagramEntries()`. The built-ins share one WASM binary; lazy family selection
does not download separate native modules. See
[web interfaces](bindings/web/README.md),
[registry contracts](bindings/web/REGISTRY.md) and
[caller-authored models](docs/DECLARED_MODEL_CONSTRUCTION.md).

## Extending the engine

`DiagramModule<Model>` keeps new languages behind a typed parser/render interface;
`Engine::empty()`, open diagram IDs and explicit `parse_as` support independent
registries. A document retains the handler and config that created it, even after
registry replacement. `RenderResources` keeps fonts, icons, time and platform
services outside diagram logic.

The mutable interface adds `PublicDiagramModule` and owned model graphs. New
providers opt into their own graph codecs and constructors; unsupported
construction fails explicitly. Graph exchange preserves aliases, cycles, UTF-16
code units and IEEE number bits without making Swift or JavaScript implement the
diagram renderer. See the [architecture](docs/FULL_PARITY_LLD.md),
[public model design](docs/LLD_PUBLIC_MODELS.md) and
[construction contract](docs/DECLARED_MODEL_CONSTRUCTION.md).

## Compatibility and limits

The port compares source preparation, complete models, method mutations, SVG,
errors and ordered callbacks with the pinned Pele oracle through public
integration and browser tests. Test counts and artifact identities belong to
specific revisions; [the progress log](docs/FULL_PARITY_PROGRESS.md) separates
completed slices from final acceptance.

Native transport and work budgets remain enforced independently of Pele's
source, edge and output options. Source/config use bounded UTF-8/JSON domains;
mutable graph strings use lossless UTF-16 transport. Arbitrary JavaScript VM
behavior, custom getters/prototype interpreters and every undeclared helper
replacement are outside the current contract. A declared plain model can expose
an upstream missing-helper error; the constructor does not invent parser state.
See [source limits](docs/SOURCE_LIMITS.md), [parse limits](docs/PARSE_LIMITS.md),
[output limits](docs/OUTPUT_LIMITS.md),
[intrinsic diagnostics](docs/WEB_VM_DIAGNOSTICS.md) and
[extreme corpus boundaries](docs/WEB_LEGACY_CORPUS.md).

Host font metrics can affect drawing dimensions. Rust renders SVG using resources
provided by the host. Source, retained-model and mutable-model APIs use this same engine.

## Performance

**0.3 remains unreleased. JS performance parity is not met.** The most recent
complete release-gate run, historical source `0b32797`, fails all 126 conditions.
New scoped diagnostics do not replace that gate.

The latest measured candidate, [`11f2751`](docs/benchmarks/2026-10-21-source-symbols/), consumes one Sankey source-symbol catalog while preserving public DTOs and admission stages. Parse remains **6.459-14.268x JS** despite all six paired medians being lower than the original e148 Rust control. **Warm retained rendering remains 10.21-14.64% slower than e148** (2.064-3.162x JS, 0-4 lower pairs out of 21). All 24 rows and six adverse Flowchart controls are retained. Native 57, fresh Swift 16/C 3/API 168, Node 122 and browser 114 pass. Separate windows do not establish an isolated gain versus [a4](docs/benchmarks/2026-10-21-accepted-baseline/), and allocation changes were not measured. First-use and the full release gate were not rerun; complete API parity remains unproven. No 0.3 release.

The preceding candidate, [`bb0fc18`](docs/benchmarks/2026-10-20-er-host-storage/source-typed-columns/), remains **rejected for release**. Compact source columns avoid tagged-scalar expansion, but parse still takes 5.50–13.15× Pele JS. Five of six edited Sankey workflows remain slower than the original best Rust checkpoint (−2.01% to +22.20% paired). All 48 diagnostic rows and adverse controls are preserved. Fresh regression checks pass: 54 native integrations, 81 browser cases, Swift 16, strict C 3 and API 168. A direct oracle audit additionally found callback-mutation and helper-reentry mismatches; existing restoration tests did not establish that contract. Full API parity remains incomplete. First-use includes parse, edit setup, render and disposal. [Prepared admission](docs/benchmarks/2026-10-20-er-host-storage/source-prepared-ingress/), [edited profiles](docs/benchmarks/2026-10-20-er-host-storage/source-sparse-ingress/profile/) and [parse profiles](docs/benchmarks/2026-10-20-er-host-storage/source-fresh-initializer/profile/) remain preserved. No 0.3 release.

The preceding default checkpoint appends Sankey SVG directly into existing paint buffers,
sharing exact numeric/escaping rules with the other renderers. [Matched VM and
local evidence](docs/benchmarks/2026-10-20-er-host-storage/svg-append/), `e148605`:

| Default Sankey path | Paired change versus indexed-source default | Candidate / Pele JS |
| --- | --- | --- |
| Warm source render, six conditions | 18.74–26.04% lower | 0.993–1.578× |
| Warm retained render, six conditions | 14.43–18.14% lower | 1.78–2.79× |
| First unchanged render | 5.51–12.79% lower | 2.20–5.11× |
| First edited render | 5.69–8.98% lower | 3.77–7.49× |

The edit adds a cyclic extension property, not diagram content. All48 diagnostic
rows and adverse controls remain visible: five warm Sankey parse rows increase
1.81–4.69%, and Flowchart retained rows increase2.06–2.63% paired. This does not
establish overall parity or neutrality. Parse still measures6.44–15.06×JS in
this window; percentages are not compounded with earlier measurements.

For that SVG checkpoint:24native feature integrations;12focused browser checks
across Chromium, Firefox and WebKit; fresh arm64 SDK Swift16, strict C3 and
API168, covering12,481drawings and16,572expected errors. No new Swift timing
claim. Intel SDK and AppKit activation remain release checks.

The preceding [indexed source ownership](docs/benchmarks/2026-10-20-er-host-storage/indexed-sankey/)
reduced cloned endpoint records while preserving public models and helpers.
Source-backed storage still needs validation and publication corrections to address
the larger mutable-parse cost. No GPU or all33diagram speedup is claimed. VM and Swift
use different machines; their timings cannot be subtracted to estimate bridge cost.

<details>
<summary>Earlier measurements and rejected experimental backends</summary>

The native canonical Sankey backend puts its model tables in Rust-owned POD
pages, streams bounded host observations and shares browser publication and
matching. Its [first runtime](docs/benchmarks/2026-10-20-er-host-storage/native-canonical-rejected/)
still regressed against the arena. Profiling located repeated memory-view
refreshes and buffer getters, rather than the grammar, as a major cost.

The [bounded row reader](docs/benchmarks/2026-10-20-er-host-storage/native-pod-cursor/),
source `5db9f35`, lowered paired parse time 22.19–29.78% and retained render
32.98–37.70% against the first native reader. The subsequent
[operation-scoped read sessions](docs/benchmarks/2026-10-20-er-host-storage/native-read-sessions/),
source `4033e41`, lower parse another **4.05–11.08%** and retained render
**4.17–15.91%** versus the preceding cursor. Separate windows are not compounded.
A matched arena comparison still fails acceptance:

| Current opt-in native backend | Cost versus arena | Cost versus pinned JS |
| --- | ---: | ---: |
| Parse, six size/resource conditions | 1.28–1.47× | 12.82–20.83× |
| Retained render, six conditions | 1.20–1.41× | 2.70–4.64× |

The latest comparisons retain all **48 rows**, 21 paired samples per row,
source controls and adverse Flowchart controls. In the arena window, Flowchart
paired parse medians rise 3.15–6.71% and retained render 2.91–3.48%; no default-path
neutrality is claimed. Candidate/control packages differ in two JS modules and
use identical WASM. **The native backend stays disabled.** Six fresh profiles
show refresh down to 0.45% parse / 0.14% retained inclusive sampled time. Native
construction and generic publication remain substantial; profile groups overlap
and do not predict speedup.

[Browser correctness at `3166b3d`](docs/benchmarks/2026-10-20-er-host-storage/native-canonical-browser/)
passes **468/468 checks**, 156 per engine, with all 586 source inputs and 113
packed files pinned and unchanged. This includes the selected native backend,
memory growth, callback restoration and failure recovery. The later session
runtime passes **84 focused checks** across Chromium, Firefox and WebKit.
Test-only `34da266` adds three factory-failure checks, each with six normal/raw
cases, original thrown values, restored aliases and exact recovery; both the
initial fixture failure and corrected run are retained.
[Fresh local SDK correctness](docs/benchmarks/2026-10-20-er-host-storage/native-canonical-correctness/)
passes 16 Swift integrations, three C consumers and 168 API conditions;
its corpus covers 12,481 drawings and 16,572 expected errors with zero explicit
budget differences. All 362 native/SDK inputs remain unchanged through these
JS/test changes. These checks establish neither performance parity nor release
readiness.

The previous [JS-column backend](docs/benchmarks/2026-10-20-er-host-storage/captured-backend-rejected/)
also remains rejected: parse was 5.3–6.5× arena latency and retained render
3.7–4.0×. Its separate 450-test correctness result belongs to `00d2411` and is
not relabeled as the native backend's validation.

The first [shared traversal attempt](docs/benchmarks/2026-10-20-er-host-storage/shared-graph-traversal-rejected/)
was rejected after Flowchart/medium fixed retained render measured **9.16% slower**
(4/21 lower pairs). The revised [shared traversal](docs/benchmarks/2026-10-20-er-host-storage/shared-graph-traversal-v2/),
source `e48ea33`, passed full 417 and focused 54 browser tests but has **mixed timing,
not a neutrality or speedup claim**: its 42-row same-WASM diagnostic has median paired
changes of −0.19% for parse, +0.22% for retained render and +0.14% for source
render; 40 first-use rows also retain
adverse conditions. Both packages use identical `365ec6d` WASM. These results cover
only the browser traversal preparation, not the later native graphless Sankey owner
and not the 126-condition release gate.

The [shared constructor evidence](docs/benchmarks/2026-10-20-er-host-storage/shared-constructor/README.md) retains both the rejected per-field extraction and its bulk-storage redesign. V1 increased all six ER parse conditions (median paired +5.45%). V2 removes the per-record wrapper/per-field sink callback while preserving the shared wire decoder: parse shifts −0.84% to +1.86% (median −0.33%, four lower/two higher), with retained/source controls mixed. Each version passed 57 focused browser checks; both VM runs used the same frozen `365ec6d` WASM and accepted V2 traversal control. These are descriptive enabling-layer results, not neutrality, a general speedup, a new Sankey backend or a passing release gate.

An earlier [storage preparation](docs/benchmarks/2026-10-20-er-host-storage/shared-public-primitives/),
source `1906d68`, separates construction and projection from native graph storage.
It still retains the arena and initial snapshot/decode. Its 30-condition VM run
shows **all six Sankey parse rows slower by 0.51–6.22%**, and all six retained
rows slower by 0.04–5.45%. In a separate 24-condition first-use run, large fixed
Sankey's first unchanged render is **19.04% slower**; source controls are mixed,
including a 7.16% slowdown. These preparation layers establish no speedup.

Earlier matched checkpoints have scoped gains, while still trailing JS:

| Evidence checkpoint | Paired change versus its control | Measured cost versus JS |
| --- | --- | --- |
| [Arena result transport](docs/benchmarks/2026-10-20-er-host-storage/arena-render-packet/), `9ae8142` | All 36 affected retained rows lower; median −15.30% | 1.65–18.49× |
| [Source result transport](docs/benchmarks/2026-10-20-er-host-storage/source-result-packet/), `640deee` | All 42 source rows lower; median −13.89% | 1.35–4.63× |
| [Owned Sankey construction](docs/benchmarks/2026-10-20-er-host-storage/owned-construction/), `b19f38d` | All six parse rows 8.65–11.19% lower | 8.91–15.08× |

Each link retains all samples and adverse controls. These are different artifacts
and timer windows; percentages are not compounded. First-use measurements include
parse, first API use and disposal. Native allocation counts are separate from
latency and RSS. [Earlier checkpoints](docs/PERFORMANCE_CHECKPOINTS.md) retain the
full measurement history, native-core comparisons, compiler tradeoffs and rejected
experiments.

Latest [local Swift timing](docs/benchmarks/2026-10-20-er-host-storage/native-canonical-swift-timing/),
`25d87cb` → `3166b3d`, completed all four Sankey/medium conditions with 15 pairs
each: mutable parse **−0.299%**, immutable parse **−0.125%**, render **+1.328%**,
lifetime **−0.337%**. The strict compiler/test guard passed, but the shared host
was not idle; lifetime pairs range **−82.43% to +95.81%**. Every sample is retained.
This is neither a general speedup nor a neutrality claim, and it does not measure
the opt-in browser backend.

Correctness is pinned separately from performance:

- `9ae8142`: full browser suite **417/417**, 41 native integrations, Swift 16,
  strict C3 and 168 API conditions pass.
- `1906d68`: 20 targeted native integrations and **24 browser checks** pass;
  fresh SDK, Swift 16, strict C3 and API168 pass with all 353 source/SDK inputs
  unchanged. The preceding construction slice passed 18 overlapping integrations.
- Subsequent helper-storage source `243580c`: **22 native integrations** pass,
  including raw duplicate keys/descriptors, exact number bits, partial failures
  and repair. Fresh SDK at `365ec6d` passes Swift 16, strict C3 and API168 with all
  354 source/SDK inputs unchanged. The traversal experiment stopped at 147 browser
  passes, one interruption and 269 unrun checks; it is not a full-suite pass.
- Captured native core `25d87cb`: **61 feature-enabled and 29 default native
  integrations** pass. Its fresh arm64 SDK passes Swift 16, strict C3 and API168;
  the ER/Sankey corpus covers 12,481 drawings and 16,572 expected errors with zero
  explicit budget differences. All 357 source/SDK inputs stay unchanged through
  correctness. Later combined source dispatch `eb72419` passes **22 native
  integrations** separately; the SDK proof remains pinned to `25d87cb`.
  [Exact logs, source bindings and portable verifier](docs/benchmarks/2026-10-20-er-host-storage/captured-native-core/README.md).
  These overlapping checks do not establish captured-browser validation or latency.

Work now targets the remaining publication/read boundaries and native
construction costs, preserving eager projection and full callback restoration.
Intel SDK and AppKit activation
verification remain release checks. No GPU acceleration or all-33 JS speedup is
claimed. Timings exclude startup, downloads and SVG painting; Swift and VM figures
use different machines and cannot be subtracted to estimate bridge cost.

</details>

## Build and verification

Tooling: pinned Rust **1.98.1**, matching wasm-bindgen CLI **0.2.128**, Node 20+
and npm. Swift requires macOS/Xcode and Swift 6+.

```sh
rustup toolchain install 1.98.1 --profile minimal --component rustfmt
rustup target add --toolchain 1.98.1 wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
npm ci --ignore-scripts
npx playwright install chromium firefox webkit
scripts/build-native.sh
scripts/build-wasm.sh
npm run test:types
```

The previous eager checkpoint passed 42 native public integrations and all 324
browser checks. Current static-policy validation passes 20 native integrations,
three strict ARM64 C programs, the complete 33-family Swift corpus plus seven
eager integrations, all 168 consumer API conditions and 12 corpus/identity checks
for ER/Gantt/Sequence across Chromium/Firefox/WebKit. Full Swift UI activation checks remain failing
as disclosed above; the new ARM64 SDK is not universal Intel release proof.
Earlier cross-platform release evidence and the broader pinned corpus history
remain in [verification evidence](docs/FULL_PARITY_PROGRESS.md) and
[0.2 acceptance](docs/RELEASE_0_2.md).

`scripts/verify.sh` runs Rust integration corpora, packaged consumers, strict C
ownership checks, Swift integration features, AppKit checks and packed-WASM
browser tests. Browser tests use one worker across Chromium, Firefox and WebKit;
local Chromium uses installed Google Chrome. Native UI checks require an active
macOS desktop and count assertions only after the QA app becomes active.

`scripts/package-release.sh` creates universal macOS/WASM artifacts, verifies the
crate, computes SHA-256 checksums and updates the Swift distribution manifest.
It does not publish. Distribution checks and compatibility boundaries are
recorded in [release acceptance](docs/RELEASE_0_2.md).
Tests are integration/end-to-end, not unit tests.

## License and attribution

Engine: [MIT](LICENSE). Upstream Pele: [NOTICE](NOTICE) and
[upstream MIT license](third_party/PELE-LICENSE). Unmodified Noto Sans test font:
[SIL OFL 1.1](crates/pele/assets/fonts/OFL.txt), with pinned
[origin/hash](crates/pele/assets/fonts/README.md).
[Third-party notices](THIRD_PARTY_NOTICES.md) cover the remaining derived code
and fixtures. The distributed artifacts also retain
[Rust dependency notices](crates/pele/assets/RUST-DEPENDENCY-LICENSES.txt) and
[Rust standard-library/runtime notices](crates/pele/assets/RUST-STDLIB-LICENSES.txt).
System fonts are not redistributed.
