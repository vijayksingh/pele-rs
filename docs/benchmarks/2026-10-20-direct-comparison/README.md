# Direct comparison of acknowledged mutable state

Native core: `6c63a71`; JavaScript: `fa9f4e8`; benchmark harness: `0cb9361`. No v0.3 release. Every one of the 126 measured conditions remains slower than Pele JS.

## Changes and contract

Native short-key lookups avoid temporary UTF-16 allocations while preserving canonical string IDs and exact construction-work charges. The direct host comparator subsequently checks every retained node against the acknowledged, validated native frame. An exact match can omit constructing a new wire snapshot on zero-argument operations from explicitly opted-in providers.

The comparator verifies own descriptor/order/flags, prototypes, canonical string/function/reference IDs, raw numeric bits, detached aliases, Map/Set ordering, Date values and array length. Host references and uncertain shapes use the original full snapshot lane. Confirmation clears before work and returns only after complete native publication; rejected scans/imports and failed transports force retry. Every outcome still restores host callback mutations and partial-error state.

The existing plain-data compatibility boundary remains: arbitrary proxy traps, changed built-in intrinsics and universal metaprogramming read timing are outside the LLD claim. Fallback may inspect ordinary descriptors twice. No source-parse cache, arena COW, packed-publication or string-index prototype is in this artifact.

## Independent VM and timer precision

Host: AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4. Three complete reports share the same harness: `control.json` uses `f475927` (including the output-limit scan optimization), `short-keys.json` adds `6c63a71`, and `direct-comparison.json` adds `fa9f4e8`. All 21 workloads × two measurers × three public stages are present.

Same-origin isolation enables measured 5-microsecond clock ticks. The previous non-isolated reports remain historical; do not combine their percentages with this series. The checker now requires isolation and observed ticks no coarser than 10 microseconds, alongside exact package/harness hashes and a complete matrix. A focused one-condition diagnostic was correctly rejected as incomplete.

Nine alternating JS/WASM paired trials follow at least 500 ms warmup per API. Calibrated batches target 12 ms and stay capped at 128. Retained models are fresh per batch; setup and one warm render are outside timing. Parse includes disposal. Complete SVG and dimensions match before/after. All samples, batch counts and shared-host load remain available.

Direct comparison versus the immediately preceding short-key package (median of 42 per-condition percentage changes):

| Stage | Median time reduction | Median WASM/JS | WASM/JS range |
| --- | ---: | ---: | ---: |
| render-source | -0.7% | 2.53× | 1.67–5.09× |
| parse | -1.1% | 11.67× | 3.59–44.65× |
| render-model | 11.1% | 4.41× | 2.01–22.22× |

Retained rendering improved at the median by 11.1%. Flowchart/medium/Canvas regressed 6.5% in the complete run. A separate reversed-order, 21-trial/1-second-warmup repeat measured direct comparison 11.7% faster with fixed metrics and 13.6% faster with Canvas; this diagnostic does not replace the complete matrix or erase its regression.

The preceding short-key layer reduced parse time 3.8% at the stage median, with 11 parse conditions regressing more than 5%. Source and retained stage medians were effectively unchanged. The direct-comparison layer likewise has source/parse regressions despite not optimizing those paths. Every condition and regression is retained in the comparison files. GC variation is substantial: for example short-key Pie/medium/Canvas parse samples range 0.071–0.182 ms. These sequential shared-host observations are not an isolated causal estimate for each change; no regression is declared resolved.

`release-gate.json` fails 126/126 matching-JS conditions after validating all eight tracked runtime hashes against the exact measured package. Reproduce with that artifact:

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/web-vm.mjs \
  --package EXACT_PACKAGE --warmup-ms 500 --output web.json
node scripts/perf/check-web-parity.mjs web.json EXACT_PACKAGE
```

Large Canvas source-render results:

| Workload | Pele JS | Rust/WASM | WASM/JS |
| --- | ---: | ---: | ---: |
| flowchart | 1.356 ms | 2.534 ms | 1.87× |
| sequence | 0.517 ms | 1.066 ms | 2.06× |
| er | 2.744 ms | 6.742 ms | 2.46× |
| pie | 0.274 ms | 0.659 ms | 2.40× |
| sankey | 0.846 ms | 1.588 ms | 1.88× |
| gantt | 0.765 ms | 2.865 ms | 3.74× |
| railroad | 0.794 ms | 1.967 ms | 2.48× |

## Functional and Swift proof

19 public Rust integration tests passed for the native short-key layer, including the 16,000,000/+1 construction-work boundary, raw UTF-16, duplicate string IDs, readonly method errors, binary transport and the 1,023 all-family numeric corpus.

63 production-package browser E2E checks passed across Chromium, Firefox and WebKit, covering all 33 numeric families, callback/error restoration, graph comparison, descriptors, detached aliases, Unicode, exact numbers, collection changes and recovery. TypeScript consumer checking passed. Astra independently reviewed the comparator and supplied an additional binary/host-state regression slice.

Fresh ARM64 SDK: seven local Swift output-limit/numeric integration tests passed; `swift-verified.json` records 168 exact output checks across 21 workloads and eight Swift APIs. Source fingerprints and byte identity of the linked archive establish freshness. Latest native change has not yet received the universal Intel rebuild required for release.

Apple M1 Pro medium-workload timings: seven trials after at least 100 ms warmup, with compiler/Pele-test quiet guard. Shared desktop activity is recorded. Workloads use 48 units; Railroad uses 15 rules. No cross-machine JS ratio is implied.

| Workload | Source render | Retained immutable render | Mutable render |
| --- | ---: | ---: | ---: |
| flowchart | 0.605 ms | 0.479 ms | 0.596 ms |
| sequence | 0.272 ms | 0.230 ms | 0.283 ms |
| er | 1.622 ms | 1.376 ms | 1.523 ms |
| pie | 0.294 ms | 0.284 ms | 0.301 ms |
| sankey | 0.376 ms | 0.339 ms | 0.392 ms |
| gantt | 0.454 ms | 0.408 ms | 1.144 ms |
| railroad | 0.477 ms | 0.501 ms | 0.473 ms |

Public API ownership costs are included; startup, transfer, painting and GPU work are excluded. Raw Swift samples and exact build/input fingerprints remain in `swift-timing.json`.
