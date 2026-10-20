# Independent static-policy results review

The allocation diagnostic confirms removal of 33 one-byte allocation requests from public engine construction. The VM comparisons show modest Gantt source-render improvements against the stored-enum implementation and near-flat source-render medians against the earlier direct-Gantt control. Parse remains mixed, including a **+3.53% paired Gantt fixed-profile observation** against that earlier control. These measurements do not establish absence of regressions. The current complete JS performance gate still fails **126/126 rows**.

Six new first-attempt Swift timing windows are now accepted and independently audited below. Parse paired medians improve **−0.862% for Gantt and −2.163% for Sequence**, each 13/15 faster. The four earlier interrupted attempts remain excluded, including the completed Sequence row inside an otherwise failed attempt. The preceding normalized-projector speedup is not transferred to this policy binary.

Read-only evidence audit, 2026-10-20. No builds, tests, browser sessions, VM commands or benchmark consumers were executed by this reviewer. `audit-results.py` and `results-audit.json` preserve independent arithmetic/hash checks. Parent-reported integration/corpus execution is separate evidence.

## Artifacts and measurement protocol

All files in both mirrored frozen control packages and the current candidate package exactly match their recorded manifests: **105 files per package, including complete file-set equality**. Each comparison differs solely in `pkg/pele_bg.wasm`:

| Runtime | WASM SHA256 |
| --- | --- |
| Earlier direct-Gantt control | `5280e2211f6a133314f914156ea95c4792b5ee561fc433704f3897daff388207` |
| Normalized projector with stored enum | `1cef29703a89a502d765958f0429af74c92f379b400293bf616af8ce0717ea76` |
| Current static policy | `b71e7365d757889d76d7e8b1b5e6eb4a6e8d95dc44d09f1d20c6acf76b6b834e` |

Controls are the exact mirrors under `parse-architecture/{normalized-projection,direct-gantt}/package`; candidate is `dist/package-web/package`. The static-policy allocation manifest records revision `ce559f1be92a751576d241440140a28bb728c508`. Package hashes, rather than that Git label alone, identify the measured browser artifacts.

Both diagnostic reports have **eight rows**, not 24: Sequence small and Gantt medium × fixed/canvas measurers × source render/parse. Each row has 21 trials and at least 2,000 ms warmup per runtime. Control/candidate batch counts match, are powers of two and do not exceed 128. The cyclic/reverse three-runtime order gives each runtime seven placements in each position; it is established by hashed source, not stored per-trial order records. Warmup is duration-based and operation counts differ between runtimes.

Diagnostic harness SHA `7d632fa69ddfbd18fe716388f6e277e6b48ed267fa378b9f73d4ff74f7e1a808`, workload-generator SHA `b16a28135adb4206a10af40f12b3a1508c2648e6dc1be3b1a4cff5a62f537a58`, and pinned oracle SHA `d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d` match current local files. Every diagnostic and full-gate case source hash/byte count matches the generator. Both diagnostic runs completed; their hashed harness verifies unchanged package hashes after timing and whole SVG/Object.is dimensions before/after. Saved reports retain verification completion and SVG byte counts, not the full SVG strings, so independent correctness replay is not claimed.

All raw sample lengths, medians, nearest-rank sample p95 values and reported ratios independently reproduce. At 21 samples the p95 is the twentieth ordered sample, not the maximum. Paired ratios below use samples with the same trial index.

## Recomputed paired observations

Each cell is paired median percentage change and faster-pair count. Negative means candidate lower. Full independent medians, ranges and individual paired ratios are in the audit JSON.

| Workload / profile / API | Versus stored enum | Versus earlier direct Gantt |
| --- | ---: | ---: |
| Sequence small / fixed / source render | −0.66%, 13/21 | +0.33%, 9/21 |
| Sequence small / canvas / source render | +0.11%, 10/21 | −1.52%, 14/21 |
| Gantt medium / fixed / source render | −1.28%, 13/21 | +0.14%, 8/21 |
| Gantt medium / canvas / source render | −0.66%, 13/21 | −0.36%, 13/21 |
| Sequence small / fixed / parse | −0.03%, 11/21 | +1.50%, 6/21 |
| Sequence small / canvas / parse | +0.16%, 10/21 | −1.43%, 11/21 |
| Gantt medium / fixed / parse | +1.22%, 8/21 | +3.53%, 6/21 |
| Gantt medium / canvas / parse | −0.61%, 13/21 | +1.11%, 8/21 |

Gantt source-render ratios of independent medians are −0.40%/−0.87% versus enum and −0.25%/−0.78% versus direct Gantt. The slight sign difference for fixed source render against direct Gantt (+0.14% paired versus −0.25% independent medians) is real arithmetic, not a contradiction.

The fixed Gantt parse result versus direct Gantt is +3.53% paired with only 6/21 faster; its independent-median change is +2.06%. Sequence fixed parse versus that control is also positive (+1.50% paired, 6/21 faster). Do not summarize the candidate as universally flat or regression-free. Conversely, shared-page GC/cache effects, large outliers and this narrow matrix do not establish a universal slowdown. For example, Sequence canvas parse versus the earlier control spans −38.46…+198.32% across pairs. No confidence interval, causal attribution of timing changes to allocations, or browser parse improvement is claimed.

## Allocation proof

Both frozen diagnostic executable hashes and their three recorded source hashes match. The candidate binary SHA is `b38c4bacb263d6ff3b9d62c09a3273a2568ef00fe7276f204903b7c27ab4419d`; control is `c2cd0a31a563c0a1f8fa0d759d2868437b66e5aa805fed967cfe71078d84a243`. The example source is identical. Each report has five identical rows:

| Counter | Enum control | Static policy |
| --- | ---: | ---: |
| Successful allocation calls | 69 | 36 |
| One-byte allocation calls | 33 | 0 |
| Successful reallocations | 4 | 4 |
| Cumulative requested bytes | 3,948 | 3,915 |

This proves the expected request-count reduction in the instrumented native `Engine::with_builtins` constructor after one warmup. Capture excludes capability inspection, destruction and report construction. Requested bytes include the entire new size of reallocations; they are not peak/live bytes, allocator-reserved bytes, RSS, elapsed time or measured browser allocations. The focused manifests do not claim a complete dependency/toolchain aggregate. See `REVIEW.md` for the allocator and type-policy source review.

## Full gate and Swift status

Current `web.json` contains all 126 required rows and completes its correctness/measurement protocol. Its harness SHA `75667ced027238c8c97fcc0be78a1ee016a7c2f71c4c0f59ab2df427cbd170fb` matches `scripts/perf/web-vm.mjs`; every recorded runtime hash matches the current candidate, and oracle/workload hashes match. All row medians, p95 values and WASM/JS ratios independently recompute. Reconstructing `gate.json` gives **passed:false, checked:126, slowerCount:126** exactly.

Median row ratios, averaging the two middle values across 42 rows per stage, are **2.3949× source render, 11.3400× parse, 4.1928× retained render**. These summarize rows, not an aggregate workload speed ratio. The audit JSON additionally records the lower nearest-rank convention. The report's `succeeded:true` means protocol completion, not performance-gate success. JS parity and the 0.3 release condition remain unmet.

`quiet-windows.json` lists four failed Swift timing attempts, all with build-process guard errors and `completed:false`. Attempts 1, 2 and 4 have no measured trial pairs; attempt 3 has 15 Sequence pairs and an empty Gantt row. None is accepted or used here. The archived `measured-before-order-offset.mjs` matches all four reports' harness hash.

The new public comparator validates numeric `--order-offset` as 0 or 1 and initializes rowIndex from it, reversing the first-side imbalance for short single-row windows without changing timers or guard rules. The saved offset-1 correctness-mode report is complete, has no timed samples, starts candidate-first and matches current comparator SHA `3f6853435e45703c2aa3b94f36026444666a963a71c03e8e04e21fb46bab4549`. That supports the recorded correctness execution, not a timing result. Syntax checking and broader native/C/Swift/browser integration passes were reported by the parent; the full 33-family Swift mutable corpus was still running at the initial audit handoff and completed before the accepted timing windows below, according to the parent.


## Accepted current-policy Swift timing update

All six `swift-single-*-attempt1.json` reports are complete and use current comparator SHA **`3f6853435e45703c2aa3b94f36026444666a963a71c03e8e04e21fb46bab4549`**. `single-windows.json` records six first-attempt zero exits. The controller chooses offsets from case position before running: Sequence uses 0, Gantt 1. It has no performance-dependent acceptance logic. All four prior interrupted reports remain excluded and bound to the preserved pre-offset harness.

| Provenance | Frozen enum control | Static-policy candidate |
| --- | --- | --- |
| Build revision | `058fcaabe21547b0ea3c03c4b9567b65feb71f7c` | `ce559f1be92a751576d241440140a28bb728c508` |
| Binary SHA256 | `c0a50071ae34c9fb5b2f1269ba01b4dec93b6b4abb5a58b3080f49e1406ccb6b` | `61db2248fd0db87c698be194f9642ce8f2271dd49da45f70f35aceaa8209d6a8` |
| Manifest SHA256 | `f121f83c8c8bb50d41e60018457027ff441832064cbc627a378d1f28258a3290` | `7061727aab5cffb13181eb56e52e178c786878a443002c7028ed187a7bf25999` |
| Source/SDK aggregate | `8de3d3bdb4ac141d0f5aa3815137a3236a99de43e2174b3df2c40e068dfb4de2` | `4a1ffee9c1b87c8513edfdd8345db9af0cb7ddad1c867908d22a33e353d743a2` |

All six reports match the same on-disk binaries/manifests, and both manifest aggregates recompute. Candidate complete current source/SDK aggregate was independently read through `build.mjs` inputs without invoking its build entry point and matches the recorded candidate. The only changed runtime inputs are **engine.rs and the rebuilt CPele archive**. Consumer source, SDK version, Swift compiler, architecture, case generator and oracle match. Native diagnostic allocation counts and Swift timing are distinct instruments; their executables are not interchangeable.

Each window has 15 matched pairs. Sequence windows are 8 control-first/7 candidate-first; Gantt windows are 7/8. Total across the six windows is exactly **45 control-first / 45 candidate-first**, verified from raw orders rather than assumed from settings. Each workload still has one fixed offset within this run; aggregate balance does not remove all case-specific ordering uncertainty.

| Case / API | Batch | Control median ms | Candidate median ms | Independent-median change | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Gantt medium parse | 32 | 0.605967 | 0.600736 | −0.863% | −0.862% | 13/15 |
| Sequence small parse | 128 | 0.059501 | 0.058422 | −1.813% | −2.163% | 13/15 |
| Gantt medium source render | 32 | 0.435307 | 0.435230 | −0.018% | −0.057% | 8/15 |
| Sequence small source render | 128 | 0.082598 | 0.081340 | −1.522% | −1.522% | 10/15 |
| Gantt medium retained render | 64 | 0.367279 | 0.364525 | −0.750% | −0.128% | 8/15 |
| Sequence small retained render | 128 | 0.068224 | 0.068418 | +0.284% | −0.287% | 9/15 |

Every raw per-operation array, paired ratio/change, median/MAD/min/max/sample-p95, independent-median ratio and win count independently recomputes. Warmup operations match between binaries and each side accumulates at least 1,000 ms; calibration doubles from one to the common cap or slower-side 12 ms target. Final counts match the recorded calibration and never exceed 128. Every saved trial/calibration guard record passes the configured build/Pele-test exclusions. The guard remains a pre-request snapshot; visible WindowServer/desktop load is not eliminated.

Every saved case source/hash/byte count, five-output preparation and verification record, before/after check and pair checksum/output/arena observable matches. The matched timer still includes eager immutable construction and destruction; retained setup and output verification/hashing remain outside it. No model-access, decodeModel or combined parse-plus-first-render measurement has been added.

An important retained-render tail remains: Gantt trial 5 contains **23.2805 ms control versus 116.509583 ms candidate per 64-operation batch**, a **+400.46% paired outlier**. It is retained in all calculations; the paired median is −0.128%. Thus “near-flat controls” describes their medians, not uniformly stable latency or absence of regressions. These modest local gains are descriptive observations on a shared host, not statistical significance or proof that all elapsed gains come from the 33 eliminated requests.

The parent reports the current full 33-family Swift mutable corpus completed in **652.187 s**, covering 167,473 drawings and 270,358 errors with five existing budget differences, plus seven eager integrations (eight test cases total); all 168 consumer API checks passed against the current SDK/source. Those results were not rerun by this reviewer and the five known differences are not silently called exact parity. The independently audited VM gate remains **126/126 slower than JS**, so no 0.3/JS-parity conclusion changes.
