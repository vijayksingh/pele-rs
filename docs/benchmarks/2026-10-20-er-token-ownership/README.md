# ER parser ownership

Source `0b32797` removes two unnecessary ownership layers from ER parsing.
The lexer retains exact source slices; grammar actions create owned Strings
only when consuming text. Returned models remain fully owned. Lexer state,
byte offsets, grammar, captured providers, fallback and error/limit checks are
unchanged. No JS parsing or layout is added. [Architecture](../../LLD_DIRECT_ER_PARSE.md),
[borrowed-token source review](REVIEW.md), [proposal manifest](manifest.json).

The final timing control is `175daa8`, before both token changes. The intermediate
`cbd6290` move-only results remain [separate](intermediate/summary.json). Its
proposal/review predates execution; current correctness and measurements below
supersede the proposal's not-run status. The borrowed proposal starts at cbd;
its final measured candidate is 0b. Gains from different windows are not pooled
or multiplied.

## Native computation

Five accepted windows use 15 balanced JS/control/candidate trials, 1000 ms
warmup and fixed batch count 16. Timers exclude process transport; public parse
includes creation/drop. One compiler-interrupted attempt is excluded entirely.

| Case / stage | Control ms | Candidate ms | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| er/medium / parse | 0.175737 | 0.154237 | -13.180% | 15/15 |
| er/medium / parse-public | 0.273372 | 0.252049 | -7.567% | 13/15 |
| er/medium / render-source | 0.944182 | 0.920477 | -2.278% | 10/15 |
| er/medium / render-retained | 0.738661 | 0.744107 | +1.168% | 5/15 |
| gantt/medium / parse-public | 0.157375 | 0.160768 | +2.156% | 4/15 |

Core parse wins all 15 pairs. Public parse improves but remains slower than JS;
retained and unchanged-family controls include adverse observations. These are
shared-host matched observations, not confidence intervals or a regression-free
claim. Native retained render uses a typed model, not mutable web publication.

## Swift APIs on this Mac

Six accepted ARM64 windows use 15 pairs, at least 1000 ms measured warmup and
matching counts. Aggregate first-side order is 45/45. One interrupted attempt
is excluded in full; no guard is bypassed and no unrelated process stopped.

| Case / stage | Control ms | Candidate ms | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| sequence/small / parseMutable | 0.069648 | 0.068981 | -0.466% | 8/15 |
| er/medium / parseMutable | 0.292592 | 0.272179 | -7.595% | 15/15 |
| sequence/small / lifetime | 0.229420 | 0.229257 | +0.844% | 7/15 |
| er/medium / lifetime | 2.354917 | 2.354516 | -0.119% | 8/15 |
| sequence/small / parse | 0.060740 | 0.061561 | +1.203% | 4/15 |
| er/medium / parse | 0.622250 | 0.606398 | -2.654% | 12/15 |

Lifecycle includes mutable parse, snapshot, no-op apply, render and close.
All 348 source/SDK/consumer hashes and the actually linked source-built host
archive are frozen in [source evidence](source-freeze.json) and the
[build manifest](candidate-swift-manifest.json). Native inputs and actual timing/
allocation binary hashes are [retained](native-source-freeze.json). The desktop
has other active applications; guard acceptance does not mean an idle host.

[Recomputed samples and raw report links](summary.json),
[offline arithmetic script](summarize.py),
[native window selection](native-windows.json), [Swift selection](single-windows.json).
Both interrupted reports/logs remain; retries select only guard failures, not
performance outcomes. Exact whole-output verification surrounds each timing.
Native reports retain integer durations/counters; SVG assertions are in the
hash-bound comparator rather than complete SVG payloads in every native row.

## Allocation evidence

[Separate instrumented reports](native-allocations.json) count one operation,
with prepare/reset before counters and JSON reporting after counters stop.
ER-medium typed parse allocations fall **2069 to 1259**, public parse **3407 to
2597**, both 810 fewer. Public parse requested bytes fall **321618 to 303287**;
peak additional requested bytes **133197 to 123026**, with zero live delta after
drop. Reallocation counts are unchanged. These are allocator requests, not RSS
or total retained document memory.

The [isolated borrowed-token comparison](isolated-native-allocations.json),
against the move-only binary, removes another 333 allocations. The
[intermediate report](intermediate/native-allocations.json) removes 477. The
source-only count of 332 skipped medium payloads excludes the prepared trailing
newline; the actual counter difference is 333. Retained/mutable-publication and
other-family allocation controls remain identical.

## Independent VM

The [30-row comparison](web-comparison.json) uses 21 balanced three-way trials,
1000 ms warmup, matched WASM counts and fixed/Canvas resources. It covers three
ER sizes and Sequence/Gantt controls across source, parse/dispose and retained
mutable render. All 105 actual candidate package files match the
[freeze](package-freeze.json); control hashes match the prior direct-ER package.

| ER case | Resources | Paired change | Faster pairs |
| --- | --- | ---: | ---: |
| er/small | fixed | -7.093% | 15/21 |
| er/small | canvas | -3.156% | 14/21 |
| er/medium | fixed | -2.820% | 14/21 |
| er/medium | canvas | -3.566% | 15/21 |
| er/large | fixed | -2.547% | 16/21 |
| er/large | canvas | +45.218% | 9/21 |

Five ER parse paired medians improve, but the large Canvas row is substantially
slower. Other stages/controls and [38 first-use conditions](first-use.json) are
mixed. No universal web improvement or absence of regression is established.
First-use timings use fresh documents/disposal and 15 trials; source, first
unchanged/edited render and first existing-entity helper retain complete output
and identity signatures. The resource-callback assertion remains enabled in the
default harness, but is explicitly outside this successful first-use run because
it exposes the already recorded baseline failure; no callback timing is claimed.

The [new complete matrix](web.json) still fails [126/126 conditions](gate.json).
Median WASM/JS ratios are **2.38 times source**, **11.06 times parse** and
**4.02 times retained mutable render**. Source ratios range 1.58–5.12. This matrix
covers seven families, three sizes, two resources and three stages, not all 33
families. Nine alternating trials follow at least 500 ms warmup. Differences
between historical full matrices are not causal improvement percentages.
**No 0.3 release.**

## Correctness and open work

- 28 native public integrations pass, including all 17437 frozen ER parser,
  complete graph/error, admission/fallback and retained-alias cases.
- Eight local Swift ER/eager cases pass: 7535 mutable drawings, 9902 errors,
  zero selected-family budget differences. All 168 real Swift API conditions
  and three strict current ARM64 C integrations pass.
- 27 browser checks pass in Chromium, Firefox and WebKit. These targeted slices
  supplement the preceding full 33-family checkpoints; they are not new
  all-family executions.

The previous [84 callback comparisons](../2026-10-20-direct-er/RESOURCE_CALLBACK_EVIDENCE.md)
remain an open compatibility blocker. An additional [nested-render diagnostic](intermediate/nested-render-diagnostic.json)
proves same-model nested render succeeds in pinned JS but the old native owner
rejects it as already in use; edited rows can also lose state. Nested source and
cross-document controls match. Its [executed probe](intermediate/nested-render-diagnostic.mjs)
and log are retained. This parser layer fixes none of those ownership defects.
Existing AppKit activation/focus and Intel SDK gates also remain open. Full
API parity and release readiness are not established.

## Reproduction

Use pinned oracle and production binaries/packages at the source revisions
above. Official scripts and exact settings are recorded in raw reports and
controllers. Run the VM scripts from its repository root, with one browser job
at a time:

```bash
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/compare-web-vm.mjs \
  --control parse-architecture/direct-er/package \
  --candidate parse-architecture/er-borrowed-tokens/package \
  --cases sequence/small,er/small,er/medium,er/large,gantt/medium \
  --stages render-source,parse,render-model --profiles fixed,canvas \
  --trials 21 --warmup-ms 1000 --output comparison.json
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/web-vm.mjs \
  --package parse-architecture/er-borrowed-tokens/package \
  --trials 9 --warmup-ms 500 --output web.json
node scripts/perf/check-web-parity.mjs web.json parse-architecture/er-borrowed-tokens/package
```

The last command fails for this artifact. First-use settings/stages and all raw
samples are in the report. Swift runs locally because this Linux VM cannot
supply the macOS SDK. Source/transport/GPU language labels are not speed claims.
