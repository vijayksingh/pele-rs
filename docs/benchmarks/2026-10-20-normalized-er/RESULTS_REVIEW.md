# Independent normalized-ER results audit

The frozen evidence supports a substantial local Swift immutable ER parse improvement. It does not establish a web improvement, absence of regressions, allocation savings, or JS parity. All read-only audit assertions passed. No build, consumer, browser or timing execution was performed for this review.

`audit-results.py` reproduces pair arithmetic, settings, correctness-record and report-copy checks into `results-audit.json`. `audit-source-freeze.json` and `audit-web-freeze.json` record live-file verification completed before the next runtime layer. These are intentionally point-in-time checks; subsequent direct-ER changes are not normalized-ER evidence.

## Source, executable and package provenance

At 2026-10-20 03:55:37 UTC, independently calling only the exported read-only `inputs(root)` from `scripts/perf/swift/build.mjs` recomputed all **347** source/consumer/SDK inputs. The entire map and aggregate matched the current candidate manifest: `a5515fc0b18fe01dcb7de6c36844fb6a4db9340b3874402d64c1c07f5afe38a5`. Its revision is 4e9faee. The frozen control was built at ce559f1, with input aggregate `4a1ffee9c1b87c8513edfdd8345db9af0cb7ddad1c867908d22a33e353d743a2`.

Actual candidate binary SHA256: `33da27d09780c14b97d4af5ce121afea020f3602dd48851e1b92c34bde33c0e3`. Actual frozen control: `61db2248fd0db87c698be194f9642ce8f2271dd49da45f70f35aceaa8209d6a8`. Both matched their manifests. Only `root/crates/pele/src/compat/engine.rs` and `CPele/macos-arm64/libpele_ffi.a` differ between input maps; there are no missing paths. Compiler, SDK, architecture, consumer sources and workload inputs match. Historical control source was not reconstructed; its frozen executable/manifest is the comparison artifact.

The executed comparator, current public comparator and archived `measured-compare.mjs` all match SHA256 `3f6853435e45703c2aa3b94f36026444666a963a71c03e8e04e21fb46bab4549`. Every accepted report's embedded manifests, input differences, binary hashes and oracle hash agree. A private exact candidate manifest copy preserves that binding after future builds overwrite the active manifest.

At 03:56:21 UTC, all **105** current package files matched the VM candidate hashes. All **105** VM control hashes match the previous projection-policy comparison's candidate package. The prior full-gate `web.json` records only eight runtime hashes, not 105; all eight also match this control. Only `pkg/pele_bg.wasm` differs between the current VM sides. This is a complete live candidate package check plus control provenance against previously audited reports, not a new read of a remote control package. Harness, cases and pinned oracle files match their recorded hashes.

## Accepted Swift measurements

Six separate accepted windows, one case/stage each. Every row has 15 complete paired trials, 1000 ms warmup per binary, matched warmup operation counts, maximum batch 128, matched calibrated measured counts and alternating order. Sequence uses offset 0 (8 control-first/7 candidate-first); ER uses offset 1 (7/8). Across six windows the raw order totals are exactly **45/45**. No rows or trials were pooled from interrupted attempts.

Control/candidate columns are medians of their per-operation times. Paired change is the median of same-trial candidate/control ratios, so it need not equal the ratio of those medians.

| Case | API | Control ms | Candidate ms | Paired change | Faster pairs |
| --- | --- | ---: | ---: | ---: | ---: |
| sequence/small | parse | 0.061209 | 0.061271 | +1.698% | 5/15 |
| er/medium | parse | 0.848445 | 0.621891 | -25.669% | 15/15 |
| sequence/small | render-source | 0.084794 | 0.085701 | +1.316% | 5/15 |
| er/medium | render-source | 1.550229 | 1.565438 | +0.233% | 7/15 |
| sequence/small | render-retained | 0.069115 | 0.069853 | +2.071% | 5/15 |
| er/medium | render-retained | 1.402742 | 1.421430 | +0.946% | 6/15 |

Recomputed all reported arrays, medians, MADs, extrema, sample p95 values, ratios, paired changes and win counts. The public `summary.json` agrees exactly. No unfavorable pair was trimmed. ER parse improves in all 15 pairs. The other five paired medians are positive and must remain disclosed; neither these small shifts nor shared-host variability justify a blanket no-regression statement. The measured API excludes process startup, IPC, verification rendering/hashing and retained setup, but includes the consumer's per-operation autorelease-pool drain. This is a Swift API timing result, not a native-only parse estimate.

The ten controller events contain six accepted windows and four compiler-interrupted windows: three Sequence source attempts and one Sequence retained attempt. The three failed source reports contain zero measured pairs. The failed retained attempt was rejected by initial guardQuiet before consumer/report creation; its log exists, and a missing JSON report is expected. The retry controller permits retry only for the guard message, not slow measurements. It records every attempt and accepts the first successful window for each row. All four are excluded in their entirety.

Saved guards contain no forbidden compiler/linker or Pele-owned test process. The comparator calls the guard before every timed warmup, calibration and measured request; aggregate warmup counts and elapsed targets were checked. Guarding does not reserve the machine or eliminate transient scheduling/GC effects. Other application and renderer load remains visible, including high renderer CPU in some accepted ER parse trials. There is no claim that the host was otherwise idle.

Each accepted pair has identical observable outputs/checksums between binaries before, during and after measurements, with SVG hash/byte count and dimension bits matching the recorded pinned-oracle result. Five preparation/verification outputs per case match before/after. The separate `swift-verified.json` contains **21 cases × 8 APIs = 168** correctness rows with this same candidate manifest; every row's output matches its case record, and no elapsed timings are present. That correctness run coexisted with compiler activity, which is permissible because it contributes no timing samples. These output checks do not replace full model/Unicode/error integration coverage.

## VM comparison

Verified all **18** rows, 21 samples per runtime, 2000 ms warmup, matched WASM batch counts, medians/p95s/ratios and positive finite samples. The hashed harness specifies cyclic/reversed three-way order, exact whole-SVG/dimension checks before and after, and isolated high-resolution clocks. Per-trial order is reconstructed from that harness rather than recorded individually in this report. The candidate/control/JS instances share one page and heap environment; this diagnostic is not the release gate.

ER fixed parse is **+2.658% paired**, candidate faster in **6/21** pairs (ratio of independent medians +1.807%). Canvas parse is **-0.888% paired**, **11/21** faster (independent-median change -0.226%). Pair ranges are wide: fixed -49.30% to +83.02%; canvas -65.33% to +92.82%. All rows and ranges remain in the audit JSON. These mixed results do not support claiming that immutable ER projection improved mutable web parse, and they do not establish that unfavorable rows are harmless noise.

This layer changes immutable normalized model projection. Ordinary mutable web parse uses the separate typed/public arena path. WASM code/layout can still change despite identical wrappers, so the VM observations remain useful negative controls. No fresh full 126-row gate is included in this evidence slice; the preceding gate's failure is not superseded by this diagnostic.

## Public evidence copies

All matching non-Markdown public report/script/manifest copies were checked byte-for-byte against private originals, including accepted and interrupted reports and the window ledger. Gzip copies of available original logs were checked by decompression. Authored README/review prose is intentionally not treated as a byte-identical copied report. Public summary arithmetic matches the raw accepted rows. No report was modified during this audit.
