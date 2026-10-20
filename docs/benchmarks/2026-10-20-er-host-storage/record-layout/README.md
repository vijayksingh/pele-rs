# Fresh ER record publication — unreleased

Runtime **`bd7df0d`**, default optimized build. Five fixed ordered ER record shapes publish through bounded layout/record events, avoiding a temporary property vector and generic native Node for those records. The parser, eager public object ownership, descriptor/number semantics, admission limits and generic fallback remain. This layer follows the `a472853` error-brand dispatcher; the later Map-index experiment and separate O3 experiment are excluded from these accepted results. A separate [rejected-index appendix](rejected-index/README.md) preserves the Map experiment’s mixed 24-row report, patch, static review and 45-check log.

**No 0.3 release or JS parity claim.** Ordinary ER parse improves **7.67–17.15%** in paired medians, but remains **5.07–10.53× JS**. All six ER source-render controls are adverse in both independent VM runs. The latest complete performance release gate remains the earlier 126-condition failure. The 390 passing browser checks below are correctness coverage, not a performance gate.

## VM results and adverse controls

Ordinary comparison: 24 rows × 21 trials. First-use comparison: 36 rows × 15 trials. Both use 1000 ms warmup, fixed/Canvas resources, the same frozen packages and pinned JS. Every row is retained in [RESULTS.md](RESULTS.md), [summary.json](summary.json), [comparison.json](comparison.json), and [first-use.json](first-use.json).

| ER case/profile | Ordinary parse | Ordinary source control | First-use source control |
|---|---:|---:|---:|
| Small / fixed | −7.67% | **+4.05%** | **+6.03%** |
| Small / Canvas | −8.12% | **+1.54%** | **+1.99%** |
| Medium / fixed | −10.19% | **+1.68%** | **+2.16%** |
| Medium / Canvas | −13.68% | **+0.89%** | **+2.23%** |
| Large / fixed | −17.15% | **+1.94%** | **+3.45%** |
| Large / Canvas | −12.31% | **+0.86%** | **+2.38%** |

These are paired median changes. They are not ratios of separate medians, statistical confidence intervals or proof of causality. The repeated adverse source controls are retained without a “noise” dismissal.

Complete ER first-render changes range **−3.55% to +1.02%**; first edited render **−2.41% to +0.08%**; first callback **−4.10% to +0.31%**. First `addEntity` improves **5.44–12.98%**, but still costs **9.78–19.14× JS**. Ordinary retained controls range **−3.40% to +1.27%**. Sequence controls also remain in both reports.

First-use operations include fresh parse, first operation and disposal inside the timer. Source rendering is measured independently as a control. Ordinary retained setup and its warm render are outside batch timing. One page shares GC/cache pressure; serial warmup and different JS batch counts remain limitations. These VM comparators have no per-row process quiet guard. They require a high-resolution isolated clock and verify exact outputs/package hashes before and after. Individual observed order/timer timestamps are not saved; scheduling is derived from the hash-verified harness.

## Swift/native scope

A single complete guarded Swift window contains six conditions × 15 pairs, **90 pairs with 45/45 first order**, 1000 ms minimum matched warmup and batch cap 128. There were no retries or pooled partial windows.

| Case | Mutable parse | Lifetime | Immutable parse |
|---|---:|---:|---:|
| ER medium | −22.69% (15/15) | −3.50% (12/15) | −9.44% (14/15) |
| Sequence small | +0.20% (7/15) | −1.54% (9/15) | +0.35% (6/15) |

**Swift compares combined native changes against frozen `0b32797`, not an isolated record-layout effect.** Its candidate build manifest is at docs-only `31edd85`; runtime source is bd7df0d. All 351 native/SDK inputs match aggregate `ee5e9dfc671379d3a6065e8633bc8c6488cee7107343a74380ce73aca0bd5f2b`. Compiler/Pele-test guards reject matching processes regardless of CPU percentage; other host load is recorded, not excluded. No JS, allocator or peak-memory comparison follows from Swift timing.

The full Swift report is [native/swift-comparison-attempt1.json.gz](native/swift-comparison-attempt1.json.gz); its build manifest, summary, 168-condition correctness report and guard/log evidence are included.

## Profiles and correctness

Six raw CPU profiles cover ER medium source/parse/retained rendering in JS and WASM. The parse profile is still dominated by `allocate` (25.17% self), GC (12.01%), and `publishBatch` (6.76%). Retained `audit` is 9.43% self. These are separate 1500 ms Canvas samples under profiler overhead, not matched latency trials. Top lists group by function name or URL and can combine unrelated names. No causal subtraction, import-count or memory claim is made.

Recorded default-build validation:

- **390/390 browser checks passed in 12.9 minutes**, all three engines. Full log included; only the three new record-feature JSON reports are copied, avoiding a duplicate full-suite archive.
- **12 initial browser checks** and **24 native integrations**, including exact ER graph/corpus and generic writer coverage, passed. Initial and full browser counts overlap; do not add them as unique coverage.
- **15 Swift integrations** passed, including 7,535 ER drawings and 9,902 errors, with zero explicit budget differences; prepared cases also exercise all 33 families.
- **168 Swift API conditions**, **three strict C consumers**, fresh SDK/build and TypeScript checks passed.

The original [source review](review/REVIEW.md) and [audit](review/audit.json) are preserved at their earlier point in time: their “full suite running”, “first-use/profile not audited” and “no new Swift timing” fields predate later completed evidence. This bundle's summary verifies the retained later reports/logs. `native/correctness.json` likewise predates the authorized timing window.

## Frozen evidence and reproduction

[Source/package freeze](source-package-freeze.json): 448 immutable native/browser/WASM/consumer/test/design blobs at bd7df0d, 351 native/SDK inputs and full package maps. An independent [remote rehash](remote-package-hashes.json) confirms all 106 files per side after first-use/profiling. Local frozen `er-record-layout/package` matched candidate bytes. Only `host-er-store.js` and WASM differ from the preceding control. Generated artifact identity is verified; an independently reproducible rebuild is not claimed. Current map-experiment source/package output is not used here.

From this folder, recompute saved-data statistics and the deterministic index:

```sh
python3 summarize.py
```

This launches no build, browser, workload or network operation. It checks all 24 ordinary + 36 first-use + six Swift rows, profile top lists, package-table correspondence and completed logs. It writes `summary.json`, `RESULTS.md` and `evidence-index.json`. Gzip members use `mtime=0`; the index contains every other bundled file's size and SHA256. No binary/package/source copies are included.

To repeat measurements, use immutable packages matching these hash tables and the harnesses at bd7df0d. From repository root, on the dedicated VM:

```sh
node scripts/perf/compare-web-vm.mjs \
  --control "$CONTROL_PACKAGE" --candidate "$CANDIDATE_PACKAGE" \
  --cases sequence/small,er/small,er/medium,er/large \
  --profiles fixed,canvas --stages parse,render-model,render-source \
  --trials 21 --warmup-ms 1000 --output "$OUTPUT/comparison.json"

node scripts/perf/compare-first-use-web-vm.mjs \
  --control "$CONTROL_PACKAGE" --candidate "$CANDIDATE_PACKAGE" \
  --cases sequence/small,er/small,er/medium,er/large \
  --profiles fixed,canvas \
  --stages render-source,parse-first-render,parse-first-edited-render,parse-first-invoke,parse-first-callback \
  --trials 15 --warmup-ms 1000 --output "$OUTPUT/first-use.json"

node scripts/perf/profile-web.mjs --package "$CANDIDATE_PACKAGE" \
  --cases er/medium --duration-ms 1500 --output "$OUTPUT/profile"
```

For Swift, use the current-source-compatible candidate manifest and original immutable 0b control directory in a separately coordinated local quiet window:

```sh
node scripts/perf/swift/compare.mjs --quiet-run \
  --control-dir "$FROZEN_0B_SWIFT" --candidate-manifest "$CANDIDATE_MANIFEST" \
  --cases er/medium,sequence/small --stages parseMutable,lifetime,parse \
  --order-offset 0 --output "$OUTPUT/swift-comparison.json"
```

Changed packages, source/SDK inputs or harnesses define a new comparison. The complete release gate remains separate.

The worktree has since advanced to a separate native-borrow experiment. This bundle uses only the retained `bd7df0d` artifacts and prior immutable source/package checks; it does not revalidate current native inputs as record-layout inputs.
