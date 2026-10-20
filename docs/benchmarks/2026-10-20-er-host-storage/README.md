# ER host storage: correctness checkpoint, performance blocker

Source: `ddab7da`, unreleased. Rust owns the ER grammar, methods, validation,
layout and SVG. Browser objects become the authoritative mutable model; normal
ER parse no longer retains a native public graph, decoded JS baseline or typed
ER reconstruction. Native/Swift and exported low-level arena APIs retain their
existing contracts. [Architecture](../../LLD_HOST_ER_STORE.md).

The later [prepared-entry follow-up](prepared/README.md), source `80bed95`, fixes
the prepared callback failures recorded below and includes fresh all-family
Swift prepared/raw checks. The [raw caller-root follow-up](raw/README.md),
source `51f1269`, fixes the six raw callback comparisons in each browser, but
its repeated raw rendering regresses 71–88% in paired VM medians against the
preceding arena raw path. The JS performance target remains open. Historical
measurements below retain their original source/artifact scope.

The [result-transport follow-up](result-packet/README.md), source `7339f38`,
subsequently improves retained ER by 9.4–14.9% paired against the raw host
artifact, while remaining 4.4–6.3× JS. It does not replace the complete gate.

The [shared lexer follow-up](lexer/README.md), source `df3597c`, records
6.1–8.6% lower paired ER parse VM medians against the packet artifact. Local
Swift parse improves against the older native baseline; those comparisons
are separately scoped and do not establish JS parity.

**This is not a performance solution or release candidate.** Retained rendering
regresses consistently. The latest complete release gate remains the preceding
[126-condition failure](../2026-10-20-er-token-ownership/README.md). No 0.3 release.

## Independent VM

The [30-condition comparison](inline/comparison.json) uses the same independent
8-vCPU AMD EPYC-Genoa VM, Chromium 156, pinned Pele 1.6.0, frozen `0b32797`
control and candidate packages. It covers three ER sizes plus Sequence/Gantt
controls, fixed and Canvas resources, parse/retained/source stages, 21 balanced
three-way trials and 1,000 ms warmup. Complete SVG and IEEE dimensions match
before/after timing. Parse disposal and first-use work stay in their declared
timers. Package contents are hashed recursively before and after each run.

Percentages below compare candidate with the preceding Rust/WASM backend.
Negative is faster. Both paired medians and separate-median ratios are retained;
their disagreement on large parse is evidence against a broad speed claim.

| ER size/resources | Parse paired | Parse ratio of medians | Parse faster pairs | Retained paired | Retained faster pairs |
| --- | ---: | ---: | ---: | ---: | ---: |
| Small/fixed | -2.01% | -1.86% | 12/21 | +78.95% | 0/21 |
| Small/Canvas | -8.10% | -5.12% | 18/21 | +75.73% | 0/21 |
| Medium/fixed | -5.79% | -6.86% | 15/21 | +79.93% | 0/21 |
| Medium/Canvas | -2.48% | -4.47% | 13/21 | +76.87% | 0/21 |
| Large/fixed | -2.84% | +11.13% | 12/21 | +91.92% | 0/21 |
| Large/Canvas | -5.01% | +1.27% | 16/21 | +90.21% | 0/21 |

ER parse medians remain 6.22–13.27 times JS; retained render remains 5.26–7.51
times JS. All 30 rows, including every adverse control, remain in the raw report
and [derived paired summary](inline/paired-summary.json). No row is a confidence
interval or physical-app smoothness result.

The [22-condition first-use comparison](inline/first-use-stable.json) includes
fresh parse, unchanged/edited first render, first helper and disposal. ER-medium
first render is 53.1%/47.9% slower by separate-median ratios (fixed/Canvas), edited
first render 25.3%/21.2% slower, while first helper is 21.1%/27.3% faster. Source
and other-family controls remain recorded. The old ER backend has a separately
reproduced callback-state defect, so it cannot be a successful callback timing
control. That stage was not requested in this comparison; its correctness
assertion was not disabled or weakened.

## Rejected construction stages and profile

Earlier focused experiments each used nine balanced trials and 500 ms warmup.
These are separate windows with different JS timings, not isolated causal
comparisons between experimental packages.

| Experimental host backend, ER-medium fixed | Parse / old backend | Retained / old backend |
| --- | ---: | ---: |
| [Unbatched](unbatched/first-comparison.json) | 4.19× | 4.25× |
| [Per-node batches](node-batched/first-comparison.json) | 2.32× | 4.07× |
| [Streamed events and typed reads](streamed/first-comparison.json) | 1.26× | 1.96× |
| [Inline nodes and Map transfers](inline/first-comparison.json) | 0.99× | 1.77× |

All adverse reports remain archived. The final focused result alone does not
replace the broader comparison above. Frozen experimental packages remain
identified by complete content hashes; earlier uncommitted prototypes are not
claimed reproducible from `ddab7da`. The current source and build recipe are.

The [current six raw profiles](inline/profile/summary.json) bind the same
106-file package as both current comparisons. Weighted, deduplicated
[retained attribution](inline/attribution/REVIEW.md) finds admission 18.21%,
entry/final audits 12.15%, capture/live-reader subtrees 15.70%, remaining draw
33.93% and other work 20.01%. Capture is inlined, so an exact split between
capture and later live reads is unavailable. These are disjoint attributed
partitions, not summed inclusive callers. Even deleting all admission projects
2.571 ms, still above the old 1.771 ms focused median. A binary admission frame
alone cannot solve the regression. Parse still spends substantial time in
object allocation/publication; fewer boundary calls did not remove that work.

## Local Swift and native

The [completed Swift comparison](inline/swift-comparison-attempt2.json) uses real
Swift API consumers, 15 matched pairs per condition and 1,000 ms warmup. The six
windows balance first-side order 45/45. It compares combined source changes
against frozen `0b32797`, not isolated WASM bridge causality or JS speed.

| Swift condition | Paired median change | Faster pairs |
| --- | ---: | ---: |
| ER-medium mutable parse | +0.71% | 7/15 |
| ER-medium lifetime | -2.04% | 10/15 |
| ER-medium immutable parse | -1.14% | 10/15 |
| Sequence-small mutable parse | +0.51% | 7/15 |
| Sequence-small lifetime | +1.02% | 4/15 |
| Sequence-small immutable parse | +1.01% | 5/15 |

Mixed results and host load remain disclosed. The interrupted
[first Swift window](inline/swift-comparison-attempt1.json) and its log are
excluded whole because a compiler appeared. The guard stayed enabled. Native
elapsed timing was initially rejected at preflight by an active compiler; its
log is retained. The later [completed eight-condition native comparison](inline/native-comparison-attempt2.json)
uses 15 balanced three-way trials, 1,000 ms warmup and 16 operations per batch.
ER source/retained render improve 3.21%/6.34% paired, faster in 14/15 and 15/15
trials respectively. Typed/public ER parse move -0.35%/-0.90%, only 9/15 and 8/15
faster. All four Gantt controls, including slower rows, remain in the report.

In that same-host native/Node diagnostic, ER typed parse is 0.145 ms versus
0.221 ms JS; public Rust parse is 0.245 ms versus 0.222 ms JS. Native source and
retained medians are near JS (0.99×/0.96×). These are distinct API representations
on this Mac, not VM/WASM measurements, Swift timing or an all-family JS gate.

The [separate allocation consumer](inline/native-allocations.json) shows native
ER typed/public parse allocations, reallocations, requested bytes and peak
requested bytes exactly match `0b32797`. Sharing the fallible construction sink
initially lost array size hints: public parse grew from 775 to 779 reallocations
and 303,287 to 313,199 requested bytes. Explicit capacity restores the previous
counts. [Pre-fix evidence](streamed/native-allocations.json) is retained. Borrowed
ER render arrays remove 668 allocation requests and 23,668 requested bytes per
measured source/retained render. Gantt allocation controls are unchanged. Every
condition returns to zero additional live bytes after drop. Allocation counts
are not timing evidence.

## Integration proof and remaining gaps

- Streamed checkpoint: [339 browser tests](streamed/browser-full.log.gz), all
  three engines. Subsequent inline/Map layer: [30 focused checks](inline/browser-focused.log.gz),
  including both 7,000-entry oversized collection encodings and retained helpers.
- Current native [seven publication/fallback integration tests](inline/native-integration.log.gz),
  including all 17,437 frozen ER sources and actual graph-budget/late-syntax
  precedence. Earlier shared backend passed 31 native integration tests.
- Fresh ARM64 SDK: [168 Swift API conditions](inline/swift-api.json) and
  [eight Swift integration tests](inline/swift-corpus.log.gz): 7,535 mutable
  drawings, 9,902 error cases and zero selected-family budget differences.
  Three strict C consumers passed ownership/error, mutable-public and parse-limit
  checks. Formatting, TypeScript and diff checks passed.
- Ordinary source host callbacks preserve complete state, original arbitrary
  throws, detached arrays, nested helpers/render, repaired retry and identities.
  [Six prepared-lane cases](prepared-baseline/reproducer.spec.js) still differ in
  each engine: [Chromium](prepared-baseline/chromium.json),
  [Firefox](prepared-baseline/firefox.json), [WebKit](prepared-baseline/webkit.json).
  Caller-authored raw roots also retain the previous arena path. Full API parity
  is not established by ordinary source results.
- Intel SDK and existing AppKit activation/focus release checks remain open.
  This report does not turn compiled/tested artifacts into a release.

[Evidence index](evidence-index.json) records source hashes; source/build/package
manifests and raw profiles accompany each relevant report. No binaries are
committed here. Rebuild with `scripts/build-wasm.sh` and
`scripts/build-native.sh --host`; use the recorded compare scripts/settings on
an independent VM and local Mac respectively. Reproduce paired summaries with
`node summarize.mjs`; validate byte-for-byte package maps before comparison.
