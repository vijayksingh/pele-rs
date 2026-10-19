# Fresh records and immutable schema reuse

Web source: `fd61527`, native Flowchart construction: `53d6b91`, WASM metadata identity: `c64a836`. No v0.3 release; every measured condition remains slower than Pele JS.

## Changes

Fresh Flowchart records preserve insertion order without constructing property-index hash tables. Native schema keys identify captured immutable metadata inside one WASM instance; each renderer reuses its already validated registration on a matching key. First encounters and caller-authored raw-model adoption still validate complete metadata. Public schema APIs and custom-provider checks remain unchanged.

This artifact includes the preceding mutable transport changes. Packed publication and disposal-storage experiments were not adopted.

## Independent VM

`web.json` contains the full 126-condition run: 21 workloads, fixed/Canvas measurers, source rendering, mutable parse/dispose and retained mutable rendering. Nine alternating JS/WASM trials follow at least 500 ms warmup per condition, with 12 ms calibrated batches capped at 128. Setup and warm render stay outside retained timing; fresh retained state is used per batch. Complete SVG and dimensions match before and after. Host: AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4.

Comparison uses the same coordinate-pass control as the previous checkpoint. These are sequential shared-host observations, not an isolated causal estimate for each added change. Stage summaries are conventional medians of 42 percentage changes.

| Stage | Median time reduction vs control | Median WASM/JS | WASM/JS range |
| --- | ---: | ---: | ---: |
| Source render | -0.4% | 2.60× | 1.78–5.24× |
| Mutable parse/dispose | 25.8% | 12.73× | 2.87–45.67× |
| Retained mutable render | 21.3% | 4.95× | 2.17–29.47× |

No parse condition regressed more than 5% against that control. Retained regressions above 5%: Railroad/small/fixed 11.8%, Pie/medium/fixed 5.3%. Source regressions above 5%: ER/medium/fixed 17.9%, Sankey/medium/Canvas 9.2%, Sankey/large/fixed 5.4%, Sankey/large/Canvas 18.0%. Every sample, warmup and artifact hash remains in the raw report. Unchanged source code still varies across runs; these regressions remain recorded, not explained away.

Large Canvas source-render results:

| Workload | Pele JS | Rust/WASM | WASM/JS |
| --- | ---: | ---: | ---: |
| flowchart | 1.350 ms | 2.587 ms | 1.92× |
| sequence | 0.519 ms | 1.113 ms | 2.14× |
| er | 2.963 ms | 7.050 ms | 2.38× |
| pie | 0.278 ms | 0.684 ms | 2.46× |
| sankey | 1.050 ms | 1.963 ms | 1.87× |
| gantt | 0.691 ms | 2.750 ms | 3.98× |
| railroad | 0.813 ms | 2.025 ms | 2.49× |

`release-gate.json` fails 126/126 matching-JS conditions. The strict checker confirmed the report matches all eight actual runtime files in the measured package and the complete current harness/workload matrix. It exits nonzero because performance fails. Reproduce with the exact artifact:

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/web-vm.mjs \
  --package EXACT_PACKAGE --warmup-ms 500 --output web.json
node scripts/perf/check-web-parity.mjs web.json EXACT_PACKAGE
```

## Browser and Swift verification

105 browser E2E checks passed across Chromium, Firefox and WebKit, including all 33 mutable families, callback/error restoration, descriptor/alias preservation, actual native schema-key availability, conflicting metadata, fallback and raw adoption. TypeScript consumer checking passed.

Latest native core uses a freshly built ARM64 host SDK; the latest change has not yet received the universal Intel rebuild required for release. Four local Swift RenderNumberTests passed, covering 1,023 all-family cases through three rendering lanes, an additional 287-case numeric corpus, and Date/error/recovery checks. `swift-verified.json` records 168 exact output checks across 21 workloads and eight Swift APIs. Source fingerprints and linked SDK/archive byte identity prove the native inputs; the consumer was built at `c64a836` and measured at `fd61527`, whose intervening runtime changes are WASM-only/JavaScript.

A compiler/Pele-test quiet window allowed the local Apple M1 Pro timing refresh. Seven trials per API, medium workloads (48 units; Railroad 15 rules), 100 ms minimum warmup and a fixed UTC clock. Shared desktop processes remained active and are recorded; this is not an idle machine. Sequence retained calibration selected a two-operation batch, so its samples are especially short. These are Swift API latencies, not cross-machine JS ratios.

| Workload | Source render | Retained immutable render | Mutable render |
| --- | ---: | ---: | ---: |
| flowchart | 0.571 ms | 0.467 ms | 0.587 ms |
| sequence | 0.261 ms | 0.231 ms | 0.271 ms |
| er | 1.507 ms | 1.336 ms | 1.471 ms |
| pie | 0.286 ms | 0.271 ms | 0.283 ms |
| sankey | 0.354 ms | 0.336 ms | 0.379 ms |
| gantt | 0.447 ms | 0.372 ms | 1.112 ms |
| railroad | 0.452 ms | 0.415 ms | 0.477 ms |

Raw `swift-timing.json` includes all samples, batch counts, deviations, system load and source/build fingerprints. Public API ownership/FFI costs are included; startup, transfer, SVG painting and GPU work are excluded.
