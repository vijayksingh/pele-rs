# Owned-ID index: rejected and reverted

Experiment `6c400a83820b4af0a278ba4621f9fa94f87eb08a` replaced ModelBridge's document-local WeakMap ID index with a Map and cleared that index during disposal. Registry receiver ownership remained weak. The [ownership audit](architecture/CONTRACT.md) established that each indexed object already had a strong reference in the same append-only object history; it did not establish a performance or memory benefit.

**The experiment was rejected and reverted** in `0b42a0689a4d5cc569bfe55f5b83a693120eb737`. The recorded revert proof verifies all 558 runtime inputs against accepted source `11f2751`. Candidate measurements below describe the rejected experiment, not the restored current runtime.

## Same-run diagnostic

Timed Rust control is the immediate `11f27516d98c3d89799a7fe2065eec0bd4e147cf` package; candidate is `6c400a8`; pinned Pele JS is the third lane. Both Rust packages contain 117 files and share WASM `c0ddd411aebbcd3a2293d66cf7de50f8b0c10fafdf335d7554962922ee42f07c`. Only `full.js` and `graph.js` differ. **e148 is probe-only and was not timed in this experiment.** Its old-capability probe lane is distinct from the timed 11f control, whose prior activation proof is hash-bound in provenance.

All 24 conditions retain 21 balanced paired trials, 1000 ms warmup, matched Rust batch counts, exact outputs and 36 untimed activation/continuity/ingress checks. Small Sankey parse regresses +3.78%/+1.27%, with 6/21 lower pairs in each condition. Medium/large parse changes range from -3.39% to -0.59%. Five of six retained conditions are adverse; the range is -0.96% to +1.52%. Source and Flowchart controls are mixed and all remain in the [complete results](RESULTS.md), including absolute lane medians and ratios to JS.

Paired deltas and ratios of lane medians differ; neither statistic is substituted for the other. These data do not demonstrate a broad performance win. No cross-window compounding, allocation count, memory saving or isolated e148 comparison is claimed. No first-use, profile or full release-gate run occurred. No 0.3 release.

## Correctness and restoration

Candidate Node 122, browser 117 across Chromium/Firefox/WebKit, TypeScript and API 168/21 pass. Actual browser/Node reports and logs are preserved. The 370 native inputs and SDK are unchanged from fresh 11f validation: native 57, Swift 16 corpus and C 3 results are reused, not reported as another fresh SDK run. API 168 was rechecked. Source/package before/after maps and the independent source review are bound by hashes.

The original source-revert record explicitly precedes package restoration; it is retained without rewriting history. The supplemental `restoration-complete.json.gz` record verifies that all 117 package files exactly match accepted 11f, alongside the 558 restored runtime inputs. The portable verifier compares its complete package hash map with the timed 11f control. No candidate JS ratio is presented as the restored runtime's current measurement. Accepted 11f evidence remains in the [source-symbols report](../2026-10-21-source-symbols/).

```sh
python3 docs/benchmarks/2026-10-21-owned-ids/verify.py
```

The verifier reads data only: exact original/stored SHA256, deterministic gzip, all paired arithmetic, probe scope, source/package identity, candidate correctness and native-proof reuse. No browser, build or benchmark is launched. Preparatory runner/review statuses are archived verbatim; completed execution records establish what ran.
