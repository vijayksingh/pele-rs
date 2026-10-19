# Baseline performance measurement — 2026-10-19

This is the archived baseline before coordinate-format optimization. See the
[current profiling and optimization report](../2026-10-19-profile/README.md).

**The no-slower-than-Pele-JS target is not met.** Default warm source rendering
on the 21 matched cases is 1.92–5.25× the JavaScript time
(median of case ratios 2.91×). This is not a speedup.

## Web: independent VM

Linux x86_64, AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4,
Node 22.23.2. The VM runs independently of the macOS development machine; unrelated
services remain untouched. Core source commit 21c57eb removes the v1-only engine;
web initialization fix 6097858 is included in the final default run. The
default WASM uses Rust 1.98.1, wasm-bindgen 0.2.128, opt-level=s, LTO and panic=abort.
Runtime, oracle, harness and source hashes are retained in the raw files.
The VM timezone was UTC.

| Large workload | Pele JS | Rust/WASM | WASM / JS |
| --- | ---: | ---: | ---: |
| Flowchart | 1.387 ms | 3.025 ms | 2.18× |
| Sequence | 0.512 ms | 1.344 ms | 2.62× |
| ER | 2.900 ms | 8.050 ms | 2.78× |
| Pie | 0.281 ms | 0.769 ms | 2.73× |
| Sankey | 0.869 ms | 1.987 ms | 2.29× |
| Gantt | 0.719 ms | 2.975 ms | 4.14× |
| Railroad | 0.756 ms | 2.325 ms | 3.07× |

Large means 200 nodes/messages/entities/sections/tasks, 199 Sankey links, or 60
Railroad rules. Small/medium/large workloads cover seven diagram families, not
all 33 performance characteristics. Two resources are measured: deterministic
UTF16 metrics and warmed browser Canvas widths. The table uses Canvas.

Each runtime/stage warms for at least 100ms and six operations, then calibrates
batches toward 12ms (max 128 operations). Nine alternating JS/WASM trials retain
all samples. Every complete SVG and exact dimension matches before and after
timing. Retained-model batches create fresh state and warm it outside timing.
Explicit Rust model disposal belongs to the parse timer. JS GC during measured
work remains included. p95 fields are batch-mean quantiles, not per-call tails.

The public source-render, mutable parse and retained mutable-render paths include
boundary crossings, model exchange and result handling. Across the 21 Canvas
cases, mutable parse ratios span
3.64–73.71×;
retained mutable-render ratios span
2.68–24.16×.
These are complete API costs; no renderer-internal speed is inferred by subtraction.

Startup, downloads, SVG/DOM painting and GPU work are excluded. This is one shared
VM and one browser version, not a universal performance guarantee. No GPU backend
exists.

## Swift: actual macOS API

Apple M1 Pro; current universal SDK, arm64 release consumer. The Linux VM has no
macOS/AppKit SDK, so Swift runs locally. The same 21 sources and fixed metric/clock
resources are verified against pinned Pele by whole SVG SHA256, length, type and
IEEE dimension bits. All eight API stages pass before and after measurement.
Seven trials follow at least 100ms/six warm operations and bounded calibration.
No compiler or owned test run overlapped timing; background host load is recorded.

| Medium workload | Source render | Retained render | Mutable render |
| --- | ---: | ---: | ---: |
| Flowchart | 0.597 ms | 0.493 ms | 0.621 ms |
| Sequence | 0.275 ms | 0.235 ms | 0.290 ms |
| ER | 1.624 ms | 1.411 ms | 1.580 ms |
| Pie | 0.306 ms | 0.290 ms | 0.309 ms |
| Sankey | 0.399 ms | 0.372 ms | 0.420 ms |
| Gantt | 0.450 ms | 0.383 ms | 1.130 ms |
| Railroad | 0.469 ms | 0.442 ms | 0.501 ms |

Medium means 48 units or 15 Railroad rules. The full raw report includes small and
large cases plus parse, parseMutable, snapshot, apply and full lifecycle timings.
Actual Swift costs include FFI, native validation, intrinsic Foundation decoding
and ownership. Model inspection/projection and mutable publication add work.
Prefer source render for SVG-only work; retain a document for repeated renders.
This table cannot be compared to the VM timings: hardware and host paths differ.
AppKit decode/paint, process startup and GPU work are excluded.

## Experiments and release status

A static schema-cache trial and a larger opt-level3 WASM trial were measured
against [their retained pre-initializer-fix baseline](pre-init-default.json),
using the same warmed timing loop. A final default-package run includes the
shared initialization fix and is the source of the main table. Only input-hash
collection changed in that final driver to include the new initializer module. Neither established a reliable, regression-free overall
win; they remain unshipped. The larger profile is 43% bigger before compression.
Do not cite an exploratory case win as a general improvement.

The source cleanup advances to 0.3.0 and removes the v1 engine/APIs. It remains a
source candidate; no new release is declared performance-ready. The performance
bar remains open. Next work needs profiles of graph exchange/model publication,
source processing and SVG assembly, without dropping alias, callback or error
semantics.

Raw evidence: [default](final-size.json), [schema-cache candidate](final-schema-size.json),
[schema-cache + O3](final-schema-o3.json), [Swift](swift-macos.json),
[hash/condition manifest](manifest.json), and
[final source/artifact correspondence](delivery-correspondence.json). Later
SDK-selection/notice metadata changed; all measured production sources, native
archives/headers remain exact; the corrected web package matches all eight
files recorded in the final VM run. Driver:
[web](../../../scripts/perf/web-vm.mjs),
[Swift](../../../scripts/perf/swift/README.md). Pinned oracle:
Pele 1.6.0 / 458322a5b8abb1ae57ddf127c32017ab9fb14a92.

## Reproduce

Build the current native SDK before running Swift. Build operations and samples
must run separately. Web can run on another Linux host with matching Node,
Playwright dependencies and the exact packed archive, using loopback HTTP only.
No credentials or private files are required.

```sh
npm ci --ignore-scripts
scripts/build-wasm.sh
npx playwright install chromium
TZ=UTC node scripts/perf/web-vm.mjs --package dist/package-web/package --output dist/evidence/web.json
# Optional larger compiler-profile condition; not the default:
PELE_WASM_OPT_LEVEL=3 scripts/build-wasm.sh
# macOS only, outside browser/build activity:
scripts/build-native.sh --universal
node scripts/perf/swift/build.mjs
node scripts/perf/swift/run.mjs --quiet-run --output dist/evidence/swift.json
```

The final runs replace exploratory six-operation warmups that exhibited JIT drift.
Original exploratory logs remain in the remote task workspace; no favorable
trial was selected or assertion relaxed. Final JSONs contain all 126 rows per web
condition and all 168 Swift rows.
