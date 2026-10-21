# WASM compiler optimization experiment: keep opt-level 3 opt-in

Source `a472853`; default `s` versus `3`, with identical hand-written adapters, LTO, one codegen unit and panic=abort. **Default remains `s`.** This run supports a warmed-throughput tradeoff, not a general startup or end-to-end improvement. It is not a release gate or Pele parity claim.

## Matched warm results

[Raw report](comparison.json) retains all 126 rows and nine trial samples for every runtime. Seven families × three sizes × fixed/canvas resources × source/parse/retained stages. The browser holds independent control/candidate WASM instances and pinned JS in one page, uses matched WASM batch counts, 500 ms warmup and balanced cyclic/reverse order, and verifies complete SVG/dimensions before and after timing. Shared GC/cache pressure differs from the release harness.

Headline: median of nine paired candidate/control ratios per row, then the conventional median across the 42 stage rows (average the two middle values). Negative means lower elapsed time. [Audit](audit.json) also records the upper-middle statistic, ratios of medians, all row wins and adverse rows.

| Stage | Median paired change | Rows with paired median below control |
|---|---:|---:|
| parse | -5.71% | 41/42 |
| render-model | -1.86% | 32/42 |
| render-source | -4.51% | 41/42 |

All 126 rows: 114 paired medians below control; 119 ratios of medians below control. Seven median-ratio regressions range from +0.15% to +1.64%; twelve paired medians are adverse, up to +2.24%. These shifts are retained as observations, not dismissed as proven random noise. Examples: Railroad small/fixed retained has only 1/9 paired wins (+1.87% paired median), Sequence medium/fixed retained 2/9 (+1.31%), Sankey medium/fixed retained 3/9 (+1.27%), and ER large/fixed source 4/9 (+0.72%). Gantt medium/canvas parse is+0.82% by ratio of medians but−0.45% paired. O3 affects all families; they are coverage comparisons, not unchanged-native-path controls.

## Size and startup tradeoff

| Artifact | Default bytes | O3 bytes | Change |
|---|---:|---:|---:|
| Raw WASM | 8,125,488 | 11,663,447 | +43.54% |
| gzip | 2,475,982 | 3,355,687 | +35.53% |
| Brotli | 1,709,000 | 2,166,917 | +26.79% |

O3 adds 3,537,959 raw bytes, 879,705 gzip bytes and 457,917 Brotli bytes. Download, browser compilation/instantiation, cold first call and memory effects were not measured. The modest warmed median benefits do not establish that this larger artifact is a better default. Keep `PELE_WASM_OPT_LEVEL=3` available for explicit experiments; no runtime/compiler default change is part of this evidence.

## Provenance and correctness scope

[Provenance](provenance.json) records the source commit/tree, build settings, source-file hashes and both complete package hash maps. Frozen packages match every report hash. Only generated `pkg/pele.js`, `pkg/pele.d.ts`, `pkg/pele_bg.wasm.d.ts` and WASM differ; hand-written adapters match. [Sizes](sizes.json) retains its historical build path; frozen package identity is authoritative after subsequent builds. No generated binary is included here.

[Validation](validation.json) indexes compressed build/VM/browser logs and nine nonduplicate browser JSON artifacts. The focused O3 browser run passed 18 tests across three engines, covering all 33 low-level entries plus prepared entries, registry replacement, entry-frame behavior, error brands and lexical dispatch. This is bounded correctness evidence, not a new full-suite or release-gate pass. The audit performed no new build, browser run or benchmark.
