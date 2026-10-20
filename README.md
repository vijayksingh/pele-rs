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

**All 126 measured WASM conditions still trail Pele JS. 0.3 remains unreleased.**
Latest complete matrix: [ER token ownership checkpoint](docs/benchmarks/2026-10-20-er-token-ownership/),
source `0b32797`.
Source render takes **1.58–5.12×** the Pele JS time.
Median ratios across all sizes/resources are **2.38× source**, **11.06× parse**
and **4.02× retained mutable render**. These describe this artifact; differences
from historical runs are not causal speedup percentages.

The [entry read frame](docs/benchmarks/2026-10-20-er-host-storage/entry-frame/)
improves retained ER rendering by 11.02–14.37% paired against the lexer artifact.
The subsequent [error-brand allocation removal](docs/benchmarks/2026-10-20-er-host-storage/error-brands/)
improves it another 7.44–10.01% against the frame artifact, while parse controls
are mixed. Retained ER still takes 3.35–4.30× JS time in that latest scoped run.
These separate measurement windows must not be compounded into a single gain.

The [shared ER lexer follow-up](docs/benchmarks/2026-10-20-er-host-storage/lexer/)
improves VM ER parse by 6.1–8.6% paired against the preceding packet artifact.
Local Swift ER mutable/immutable parse improves 20.8%/10.0% against frozen
`0b32797` across the combined native changes; Sequence controls are slightly
slower. These are scoped gains, with the JS performance gate still open.

The [result-transport follow-up](docs/benchmarks/2026-10-20-er-host-storage/result-packet/)
improves retained ER rendering by 9.4–14.9% in paired VM medians against the
preceding host artifact. It still takes 4.4–6.3× JS; no 0.3 release.

The newer [live ER storage checkpoint](docs/benchmarks/2026-10-20-er-host-storage/)
removes persistent model copies and fixes ordinary ER callback state. Its
30-condition VM comparison still shows mixed parse results and 75.7–91.9%
slower retained ER rendering by paired medians against the preceding backend.
The [prepared-entry follow-up](docs/benchmarks/2026-10-20-er-host-storage/prepared/)
fixes its callback regressions; fresh all-family Swift prepared/raw checks pass.
The [caller-root follow-up](docs/benchmarks/2026-10-20-er-host-storage/raw/)
passes focused raw callback checks, but repeated raw rendering regresses 71–88%
against the preceding arena path. Performance parity remains open; no 0.3 release.

The [direct-parse architecture checkpoint](docs/benchmarks/2026-10-20-direct-parse/)
shares source preparation and grammar, then constructs Gantt's canonical public
arena without temporary owned task records. Native public parse removes 438
allocations and has an 11.0% lower matched median. Independent VM parse medians
are 2.5–9.5% lower across six Gantt size/resource conditions. First edited and
callback renders remain slightly slower, and control rows also move; this is
not a general browser win. Full output checks pass, but the new complete
126-condition performance gate still fails every condition.

The earlier [eager Swift checkpoint](docs/benchmarks/2026-10-20-swift-eager/) removes the
second full model JSON buffer and recursive Foundation decode while keeping
`Document.model` eager. Same-Mac, matched immutable-parse medians:

| Workload | Previous bridge | Eager handoff checkpoint | Before / after |
| --- | ---: | ---: | ---: |
| Sequence small | 0.3084 ms | 0.0587 ms | 5.25× |
| Gantt medium | 5.3088 ms | 0.8451 ms | 6.28× |

Each wins all 15 pairs. Source and typed-retained controls remain near-flat;
these two workloads do not establish all-33 speedups or a JS ratio. Separate
guarded runs use 1000 ms measured warmup and identical batch counts. Raw reports,
source/SDK hashes, profiles and interrupted-build exclusions remain linked.
The ARM64 Swift corpus passes all 33 families, including 167,473 mutable drawings
and 270,358 error cases, with five existing explicit transport-budget differences.
Seven new projection integrations and 168 eight-API output conditions pass.
Native UI activation remains unverified: the earlier full suite failed 19 cases
and 53 assertions when the host could not become active/key; a bare AppKit probe
reproduced this without Pele. Universal Intel validation remains required before release.

The [next Gantt projection layer](docs/benchmarks/2026-10-20-normalized-projection/)
removes its first full JSON buffer/reparse: matched Swift parse falls from
**0.8582 to 0.6292 ms**, **26.69% lower paired median, 15/15 wins** against
that preceding eager binary. Sequence parse and source/retained controls stay
near flat. Gains from separate runs are not multiplied. VM results are mixed,
including a Gantt fixed-source slowdown; no browser gain is established.
Current Gantt corpus, eager integrations, 168 consumer API conditions and all
three browser engines pass. The full release gates remain open.

The [static-policy correction](docs/benchmarks/2026-10-20-projection-policy/)
removes 33 unnecessary handler-box allocation requests per builtin registry
construction (69 to 36). Same-Mac parse paired medians improve 0.86% for Gantt
and 2.16% for Sequence, each 13/15 faster; control medians remain near flat with
outliers. VM source medians return near the earlier control, while parse remains
mixed. Full current 33-family Swift corpus and 168 consumer API conditions pass;
this remains a performance checkpoint, not release acceptance.

The [ER immutable projection layer](docs/benchmarks/2026-10-20-normalized-er/)
removes the same JSON buffer/reparse for captured ER. Matched Swift ER-medium
parse falls from 0.8484 to 0.6219 ms (25.67% paired, 15/15 faster). Local control
medians are slightly slower and VM results remain mixed; this change does not
remove mutable web graph construction. ER corpus and current API checks pass.

The [direct ER construction layer](docs/benchmarks/2026-10-20-direct-er/)
avoids a complete typed model and conversion during eligible mutable parsing.
ER-medium Swift mutable parse improves 17.9% paired; native public parse improves
18.5%, confirmed in a second window, but still trails JS. VM ER parse paired
medians improve 0.9–7.4% across six size/resource conditions. Controls and first
use remain mixed, including slower rows. Native, Swift and three-browser checks
pass, while a newly reproduced live resource-callback mismatch remains an open
API parity blocker. No full parity or release readiness claim follows.

The [ER token ownership layer](docs/benchmarks/2026-10-20-er-token-ownership/)
borrows lexer text and creates owned Strings only when grammar actions consume it.
ER-medium core parse improves 13.2% paired, native public parse 7.6%, and Swift
mutable parse 7.6%. Public parse requests 810 fewer allocations. VM results are
mixed, including a 45.2% paired slowdown for large ER Canvas parse; this is not a
universal web win. Controls and the complete failing gate remain disclosed.

[Earlier checkpoints](docs/PERFORMANCE.md) preserve their own matched controls,
raw samples and tradeoffs; gains are not combined. Recent
[rejected text/identity experiments](docs/benchmarks/2026-10-20-parse-candidates/)
remain outside production. No JS speedup or GPU acceleration is claimed.

Independent Linux VM: AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4.
Verified 5 µs clock ticks; nine alternating trials after at least 500 ms warmup.
The separate controlled Gantt comparison uses 15 trials and 1,000 ms warmup.
Large workloads use 200 units, except Sankey (199 links) and Railroad (60 rules).
Source-render measurements with cached Canvas metrics:

| Large workload | Pele JS | Rust/WASM | WASM / JS |
| --- | ---: | ---: | ---: |
| Flowchart | 1.386 ms | 2.485 ms | 1.79× |
| Sequence | 0.523 ms | 1.031 ms | 1.97× |
| Er | 2.826 ms | 6.177 ms | 2.19× |
| Pie | 0.279 ms | 0.617 ms | 2.21× |
| Sankey | 0.864 ms | 1.485 ms | 1.72× |
| Gantt | 0.713 ms | 2.554 ms | 3.58× |
| Railroad | 0.765 ms | 1.856 ms | 2.42× |

A separate [native-only comparison](docs/benchmarks/2026-10-20-native-core/)
before the latest root rewrite removes WASM/Swift entirely: typed retained native/JS medians are 0.80× Flowchart,
0.70× Sequence, 1.04× ER, 0.50× Pie, 0.41× Sankey, 1.15× Gantt and 1.38× Railroad.
Seven medium workloads on the same Mac; this is not all-33 certification.
Native compatibility work is still costly: the latest matched Gantt medium
comparison takes 0.149 ms for native public parse versus 0.082 ms for JS.
Typed native parse takes 0.055 ms in that run; public graph construction adds
work before any WASM crossing. These are stage medians, not subtractable phase
measurements or all-diagram guarantees. Current matched-history diagnostics show 22–25%
lower mutable publication medians against matched native control; the extra
graph/revision return contract and growing arena remain disclosed.

Swift and VM figures cannot be compared across machines. Timings include public
API/ownership costs and exclude startup, downloads and SVG painting. Earlier
reports remain historical; percentage gains are not combined across timer conditions.

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
