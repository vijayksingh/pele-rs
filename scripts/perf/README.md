# Matched native performance evidence

This is an independent consumer of the Rust library and the pinned development-only Pele 1.6.0 package. Production code never loads JavaScript. `run.mjs` checks complete SVG SHA-256, UTF-8 byte length, diagram type and IEEE-754 width/height against upstream revision `458322a5b8abb1ae57ddf127c32017ab9fb14a92` before collecting samples, and checks them again afterward. A mismatch aborts the run.

Build and verify without timing:

```sh
node scripts/perf/build.mjs
node --expose-gc scripts/perf/run.mjs
```

The build manifest ties both binaries to all Rust source inputs, manifests and this consumer's lockfile. Source changes during a build reject the manifest. Run only after relevant agents freeze. The timing driver checks the manifest against current inputs and records source/binary/oracle hashes and Git worktree state.

Coordinate a quiet window with other Pele builds and corpus jobs before any timing or allocation collection. A bounded first pass is:

```sh
node --expose-gc scripts/perf/run.mjs --quiet-run --allocations \
  --cases flowchart/medium,er/medium,gantt/medium --trials 7 --batch-ms 12 \
  --output dist/evidence/perf-first-native-js.json
```

`--filter medium` selects all seven medium cases. `--cases` accepts exact comma-separated IDs; `--stages` accepts comma-separated stage names. No timing occurs without `--quiet-run`. The native protocol also requires `PELE_PERF_ALLOW_TIMING=1`; the driver supplies it only in that mode. CPU/build process inspection rejects an active Rust/Swift build. It does not stop unrelated processes. Before/after CPU and load snapshots remain part of the evidence, including external xctest activity. These are local measurements, not a claim of a globally idle host.

## Work and comparisons

The case matrix has flowchart, sequence, ER, pie, Sankey, Gantt and railroad, each at three scales. It exercises text measurement, graph layout, scheduling and string-heavy SVG output. It is representative coverage, not all 33 families or a general language comparison.

All cases use UTC, a fixed 2024-01-01 clock, font size 16, padding 8, maximum width 800, the same source limits and deterministic text width (`UTF-16 length × size × (bold ? .71 : .6)`). Both runtimes retain their options and host callbacks outside timed loops. Calendar fallback uses UTC; no OS font, filesystem or network calls occur in a loop. A separate production-font lane would need matched host resources and is not implied by these results.

| Stage | Rust work | JavaScript reference |
|---|---|---|
| `parse` | Retained immutable registry, parse and drop typed document | Parse and drop raw model |
| `parse-fresh-registry` | Registry construction, parse and drop | Same parse reference; JS package registry already loaded |
| `parse-public` | Retained public registry, parse and construct native graph/state | Same parse reference; extra native graph work is explicit |
| `parse-public-fresh-registry` | Fresh public registry plus native graph/state parse | Same parse reference |
| `render-source` | Parse plus render | Parse plus render |
| `render-retained` | Render retained typed document | Render retained raw model |
| `render-mutable-core` | Existing native mutable render API | Render retained raw model |
| `render-mutable-publish` | Render operation, revision and post-operation graph publication | Render retained raw model; native graph publication is additional work |

The four parse APIs have different return contracts. Their ratios quantify those actual API costs; they do not isolate parser algorithm speed. `render-mutable-core` is the legacy compatible API; callers requiring scheduled graph state must use the publication API.

Native-only components separate `registry`, `public-registry`, `model-json`, `graph-snapshot`, `graph-json-encode`, `graph-json-roundtrip`, `graph-binary-encode`, `graph-binary-decode`, `graph-apply` and `result-json`. Registry stages include construction and destruction. Graph roundtrip includes validation; binary encode/decode use the production binary exchange codec and its validation. Apply includes cloning the input graph and typed import. These costs are **not measured Swift/C interop overhead**. No FFI speed claim follows from them. Source parsing and immutable model JSON export are distinct phases.

Each batch starts from fresh public parse state and one warm render outside its timer. Native immutable documents remain retained; JS retained models are recreated and warmed per batch. Mutable scheduling can append Date nodes within a batch; native rows report before/after arena size, and mutable batches are capped to bound that growth. Each batch consumes a checksum of returned data. Exact output hashing is outside timed loops. Module loading, process startup, NDJSON control/IPC, DOM insertion, painting and GPU work are excluded.

Six one-operation warmups plus at least 100 ms of measured work per runtime/stage precede independent batch calibration. `--warmup-ms` controls that work duration (0 retains six one-operation warmups); recorded warmup counts make this explicit. Trials alternate JS/native order. Reports retain raw batch averages, counts, medians, median absolute deviations, min/max, sample p95 and a deterministic paired bootstrap ratio interval. **Sample p95 is the p95 of batch averages, not per-operation tail latency.** GC before each JS batch is outside its timer when Node starts with `--expose-gc`; GC during a batch is included. Native allocator behavior and JS garbage collection differ; neither is silently subtracted.

## Allocations and optimization decisions

`pele-perf` uses the normal system allocator. `pele-perf-alloc` separately counts requested allocation/reallocation bytes and freed bytes for one warmed operation; it refuses elapsed timing requests. Setup, graph reset, control JSON and result reporting are outside counters. Peak additional requested bytes are a logical live-allocation delta, not RSS, allocator resident bytes or JS heap comparison. Reallocation records its new requested size and old freed size. Freeing retained allocations created before a counted operation may produce a negative live delta.

A candidate optimization needs evidence from matched outputs and relevant feature/corpus regression tests, plus before/after samples on the same host and allocation counts. Registry caching needs thread-safety and state isolation proof. Removing preflight graph snapshots needs equivalent resource validation and failure ordering; simply skipping validation is not an optimization. This harness makes no Rust, WebAssembly, Swift or WebGPU speed claim in advance.

The [first local baseline](results/2026-10-18-first-native-js.md) records measured costs, output proof, shared-host load, initial warmup limitations, and one bounded optimization proposal.

The [isolated property-cache experiment](experiments/2026-10-18-property-cache/README.md)
records exact output/graph equivalence and before/after allocation evidence for
all 21 workloads. Runtime adoption is held: ER/Railroad allocations improve,
while small Pie workloads and unique-key memory regress. Its source-only replay
bundle preserves the experiment without changing the production library or
claiming a latency improvement.

The [actual Swift API consumer](swift/README.md) crosses the production C/binary
interface and records parse, snapshot, apply, render and lifetime costs. Its
[first verified local results](results/2026-10-18-swift-api.md) include all 21
correctness workloads, bounded three-workload timings, exact SDK provenance and
explicit ownership/scheduling history. These costs are not isolated FFI overhead.

## Independent browser VM

`web-vm.mjs` measures matched Pele JS and packed WASM public APIs in one
Chromium browser, including mutable publication and parse/disposal. The server
uses same-origin isolation headers; the harness records and requires an observed
clock tick no greater than 10 microseconds. This avoids 100-microsecond timer
quantization dominating tiny JS parse batches. Batch/memory caps remain intact.

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/web-vm.mjs \
  --package EXACT_PACKAGE --warmup-ms 500 --output web.json
node scripts/perf/check-web-parity.mjs web.json EXACT_PACKAGE
```

Focused `--cases`, `--profiles` and `--stages` runs diagnose changes. The release
checker requires all 126 conditions, exact runtime/harness hashes, sufficient
warmup/trials and matching-or-faster medians in every condition. A focused run
cannot pass. Older non-isolated reports remain historical observations; rerun
both control and candidate before attributing changes under the new clock.
