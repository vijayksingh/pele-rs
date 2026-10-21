# Independent normalized-projection results review

The completed Swift evidence supports a Gantt medium immutable-parse improvement against the immediately preceding eager-bridge binary: **0.858211 → 0.629154 ms, −26.69% paired median, 15/15 faster pairs**. Sequence parse and source/retained-render controls remain close in median. The VM diagnostic is mixed and does not establish either a browser improvement or absence of regressions. No adoption or memory conclusion follows from this review. The separately audited full VM gate still fails all 126 rows against JS.

Read-only audit on 2026-10-20. No build, benchmark consumer, browser, VM command, or production edit was performed. `audit-results.py` independently checks the six saved Swift reports, all 24 VM rows, and both raw profile trees; `results-audit.json` records reconstructed statistics and counts. Current source/SDK and local package hashes were checked separately through read-only hashing. Parent-reported integration passes are separate evidence, not tests rerun by this reviewer.

## Provenance and selection

All six `swift-*-attempt2.json` reports are complete and match current `scripts/perf/swift/compare.mjs` SHA256 `034e659a98c206b5cb09b4d9fe67a30f54bd9593f16312c589c4fdc956f75f80`. The prior duplicate summary-key computation is absent; raw arrays and paired statistics remain intact.

| Item | Frozen control | Candidate |
| --- | --- | --- |
| Revision | `364c509c4d2b2dcbeb7e63438a658c5dd6403d09` | `058fcaabe21547b0ea3c03c4b9567b65feb71f7c` |
| Binary SHA256 | `1b94f3e492374f6f0c6028c59a1020da2ed47c91f5fcdeeb5572197e428b9df6` | `c0a50071ae34c9fb5b2f1269ba01b4dec93b6b4abb5a58b3080f49e1406ccb6b` |
| Manifest SHA256 | `198268b4f4e7e7e60690212a1248e2aeab0bda998f7d4eb9e63485e704c4a7df` | `f121f83c8c8bb50d41e60018457027ff441832064cbc627a378d1f28258a3290` |
| Source/SDK aggregate | `77391725bcdf1d564af92b2496764aead70ccc020dcd7c92f380101356afec91` | `8de3d3bdb4ac141d0f5aa3815137a3236a99de43e2174b3df2c40e068dfb4de2` |

The frozen control is byte-identical to the previous eager candidate. Both on-disk executables/manifests match every saved report; manifest aggregate hashes recompute. The candidate aggregate matches current complete source/SDK inputs. Consumer files, workload generator, Swift version, SDK, architecture and oracle match between sides. Exactly four input records differ: rebuilt CPele archive plus `compat/engine.rs`, `compat/mod.rs` and new `compat/normalized_projection.rs`. Documentation added outside the enumerated runtime inputs does not invalidate this build. This is materially narrower than the earlier 02ee-to-364 comparison.

The event file is named **`quiet-windows.json`**, not guard-windows.json. It records one unsuccessful predecessor followed by all six successful windows. Attempt 1 has `completed:false`, zero cases, zero rows and a control-executable EACCES error; it supplies no measured samples. The controller starts at attempt 2, accepts the first zero exit, and retries only the specific build/Pele-test guard failure. It contains no performance-based acceptance or selection. No guard rejection occurred among these six completed attempts.

## Swift arithmetic and protocol

| Case / API | Batch | Control median ms | Candidate median ms | Ratio-of-medians change | Paired change | Faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Gantt medium parse | 16 | 0.858211 | 0.629154 | −26.69% | −26.69% | 15/15 |
| Sequence small parse | 128 | 0.059591 | 0.059992 | +0.67% | +0.70% | 7/15 |
| Gantt medium source render | 32 | 0.443055 | 0.441564 | −0.34% | +0.26% | 6/15 |
| Sequence small source render | 128 | 0.082368 | 0.083354 | +1.20% | +0.58% | 4/15 |
| Gantt medium retained render | 32 | 0.366394 | 0.367415 | +0.28% | +0.40% | 3/15 |
| Sequence small retained render | 128 | 0.069865 | 0.069023 | −1.20% | +0.10% | 7/15 |

All raw nanosecond-to-operation arrays, paired ratios, percent changes, summary median/MAD/min/max/sample-p95 and win counts independently reproduce. Gantt parse's paired range is −90.31…−23.44%; the extreme comes from a 107.44 ms control batch versus a 10.41 ms candidate batch, not the typical effect. Even its least favorable pair remains faster. These are descriptive observations, not confidence bounds.

Each single-row report has 15 pairs, 8 control-first/7 candidate-first, and identical per-side batch counts. Restarting rowIndex in six independent windows yields **48/42 aggregate first-side counts**, not globally 45/45. This small consistent order imbalance should be retained in the limitations. Every side receives matching warmup operations and at least 1,000 ms measured warmup; calibration doubles from one to the slower side's 12 ms target or common cap of 128. All saved trial/calibration guard records pass the process-name rejection criteria. Warmup stores totals, so per-request guard execution is established by the hashed harness rather than a complete warmup process log. Other desktop load remains visible; guard snapshots do not establish continuous idle-host isolation.

Every saved case source/hash/byte count, five-output prepare/verify result, before/after record, and pair checksum/output/arena observable agrees. Whole SVG hash/bytes and IEEE dimension bits match the pinned recorded oracle. The oracle binary hash was checked; the oracle was not rerun. Timing includes eager immutable parse and destruction; source render and retained render are separate APIs, not parse-plus-first-render measurements. Output hashing and retained setup/warm render remain outside the timer.

## VM diagnostic

The 24-row report is complete, uses 15 trials, 1,000 ms warmup, matched control/candidate power-of-two batches at most 128, isolated high-resolution timing, and unchanged package hashes at the end according to the hashed harness. All stored medians, sample-p95 values and ratios recompute; paired ratios are reconstructed by common trial index. Warmup elapsed time is matched by minimum duration, **not operation count**. The deterministic cyclic/reverse order balances each runtime across the three positions; raw execution-order records are not retained.

Harness SHA256 `7d632fa69ddfbd18fe716388f6e277e6b48ed267fa378b9f73d4ff74f7e1a808` and case-generator SHA256 `b16a28135adb4206a10af40f12b3a1508c2648e6dc1be3b1a4cff5a62f537a58` match local source. All four case source hashes/byte counts and the pinned oracle hash match. The reported package manifests differ only in WASM:

- Control: `5280e2211f6a133314f914156ea95c4792b5ee561fc433704f3897daff388207`.
- Candidate: `1cef29703a89a502d765958f0429af74c92f379b400293bf616af8ce0717ea76`.

Every candidate package file matches the report. After the VM gate completed, the exact frozen control at `/home/admin/pele-perf.jL9wzP/parse-architecture/direct-gantt/package` was mirrored locally under `parse-architecture/direct-gantt/package`. Independently hashing all **105 files** gives exact equality with the reported control manifest, including the complete file set. The candidate also has 105 matching files. This closes the earlier artifact-verification gap; the ownership-index package was not substituted. Both package manifests differ solely in the WASM hash above.

VM exact-output verification compares full SVG strings and Object.is dimensions before/after across retained, source and fresh-parse routes. The report stores only completion and SVG byte counts, not those full strings or their hashes; independent replay is not claimed.

Important unresolved observations:

- Sequence canvas parse: ratio of independent medians **+22.81%**, paired median **−0.60%**, 8/15 faster, paired range −34.19…+65.89%. Both statistics are correct. Their disagreement is compatible with variable paired distributions; it neither proves a regression nor licenses dismissing the result as noise.
- Gantt medium fixed source render: **+2.36%** independent medians, **+1.70%** paired, only 1/15 faster. This is a more consistent slowdown observation than the Sequence canvas aggregate and deserves explicit disclosure.
- Gantt parse paired medians range from −1.73% to +3.05% across size/profile rows, with 4–9/15 faster. There is no consistent browser parse improvement. Gantt medium fixed parse includes a +554.75% paired outlier, showing why single aggregate percentages are insufficient.

These are one-page, shared-process diagnostics with independent WASM instances and shared GC/cache pressure, not the release harness. The completed full VM gate is separate evidence at `parse-architecture/normalized-projection/{web,gate}.json`. Its harness SHA matches current `scripts/perf/web-vm.mjs`; case/oracle hashes and every recorded runtime hash match the candidate. All 126 row medians, p95 values and WASM/JS ratios independently recompute, and the gate is exactly reproduced from those ratios: **126 checked, 126 slower, passed:false**. Median row ratios are **2.4556× source render, 11.6747× parse, and 4.1103× retained render**. The completed benchmark's `succeeded:true` means the measurement/correctness protocol completed; it does not mean the performance gate passed. JS parity and the 0.3 release gate remain unmet.

## Current profiles

Both raw sample SHA256s, candidate manifest, correctness records, outermost-frame counts and percentages independently match. Gantt has 4,130 exercise samples: new projection 686 / **16.61%**, Swift eager callback 2,102 / **50.90%**, envelope counting 239 / **5.79%**, Foundation 41 / **0.99%**. No legacy project_model frames were sampled. Sequence has 1,179 exercise samples: legacy projection 324 / **27.48%**, Swift callback 297 / **25.19%**, envelope 45 / **3.82%**, Foundation 83 / **7.04%**.

This confirms the expected projector is visible in Gantt while Sequence retains legacy projection. Zero sampled legacy Gantt frames is **not a zero-fallback proof**. Categories overlap; nativeParse includes Swift callbacks and projection. The callback's larger fraction does not establish that callback absolute time regressed. A zero parse_model_json symbol count also does not prove absence of legacy JSON parsing, since Sequence still uses it and optimized symbols may be inlined. Sampling is diagnostic attribution, not an allocation result or additive time decomposition.


Parent committed the bounded runtime as `6b2d593` after these measurements. Build provenance remains the recorded 058fca worktree plus its exact source/SDK aggregate; the later commit does not retroactively change the measured binary revision.
