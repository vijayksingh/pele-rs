# Direct ER public parsing

Source `175daa8` constructs the canonical ER mutable graph through the shared
Rust grammar, avoiding a complete temporary typed entity/relationship model and
its conversion. Eager publication, final graph validation, public methods and
wire bytes remain unchanged. A conservative declaration-first subset uses this
path; other syntax retains one complete legacy fallback. This is a scoped parse
change, not a replacement for the web object bridge or a callback parity fix.
[Architecture](../../LLD_DIRECT_ER_PARSE.md), [source review](REVIEW.md),
[independent results audit](RESULTS_REVIEW.md), [validation](validation.json).

## Matched Swift APIs on macOS

The frozen control is `4e9faee`; candidate is the source-built ARM64 SDK at
`175daa8`. Six separate windows use 15 balanced pairs, 1000 ms measured warmup
and matching operation counts. All 348 current source/SDK inputs and actual
binaries are independently verified. Separate medians and paired changes are
different statistics.

| API / workload | Control ms | Candidate ms | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| ER medium mutable parse | 0.352467 | 0.289611 | -17.912% | 15/15 |
| ER medium mutable lifecycle | 2.469172 | 2.382401 | -3.228% | 11/15 |
| ER medium immutable parse | 0.621520 | 0.618550 | -1.366% | 11/15 |
| Sequence small mutable parse | 0.068425 | 0.069875 | +1.251% | 6/15 |
| Sequence small mutable lifecycle | 0.229072 | 0.229654 | +0.772% | 6/15 |
| Sequence small immutable parse | 0.062275 | 0.061318 | -1.363% | 11/15 |

Lifecycle includes mutable parse, snapshot, no-op apply, render and close.
[Raw report links](summary.json), [window selection](single-windows.json),
[controller](run-single-windows.mjs), [executed comparator](measured-swift-comparison.mjs).
First-side order totals 45/45. Five compiler-interrupted attempts are excluded
in full; logs and partial reports remain archived. The process guard was never
bypassed and unrelated builds were not stopped. Accepted windows do not imply
an idle desktop. Earlier gains are not multiplied and no Swift-versus-JS claim
follows.

## Native computation and allocation diagnostics

Five original windows use 15 balanced JS/control/candidate trials, 1000 ms
warmup and fixed batch count 16. Rust timers exclude process transport. Native
public parse includes construction/drop; retained render uses the typed model.

| ER medium stage | Pele JS ms | Control ms | Candidate ms | Paired change |
| --- | ---: | ---: | ---: | ---: |
| Typed parse | 0.219776 | 0.176880 | 0.182250 | -0.270% |
| Public parse | 0.225253 | 0.331042 | 0.268477 | -18.503% |
| Source render | 0.896003 | 0.905971 | 0.922904 | +2.400% |
| Typed retained render | 0.686729 | 0.727385 | 0.743490 | +2.229% |

Public parse wins all 15 pairs but remains about 1.19 times JS by separate
medians. Gantt medium public parse, an unchanged-family control, is +4.985%
paired with only 1/15 faster. These adverse observations remain part of the
result; this is not a regression-free claim.

Fresh-process confirmations are [separate](supplemental-summary.json), never
pooled with the original rows: ER public parse repeats -18.343% (15/15 faster),
source +0.004% (7/15), retained +0.481% (6/15), and Gantt public parse +2.932%
(6/15). An identical-candidate-binary Gantt A/A window measures +2.975% (4/15).
A/A demonstrates observed run variation, not proof that an A/B regression is
absent. [Confirmation selection](native-confirmation-windows.json).

[Instrumented allocation counters](native-allocations.json) are separate from
timing: ER public parse requests 3838 to 3407 allocations, 971 to 775
reallocations, and 380866 to 321618 cumulative bytes. Peak additional requested
bytes changes 133398 to 133197, with zero live delta after drop. Other measured
allocation controls are identical. These are allocator requests, not RSS or
whole-document retained memory.

## Independent VM

The [controlled comparison](web-comparison.json) has 30 rows: ER small/medium/
large, Sequence small and Gantt medium; both fixed and Canvas resources; source,
parse/dispose and retained mutable render. It uses 21 balanced three-way trials,
1000 ms warmup, matched WASM counts and exact whole-output checks before/after.
All 105 candidate runtime files match the locally built package.

ER parse separate medians improve 0.048–6.323%; median paired changes improve
0.856–7.354% across all six ER size/resource conditions. Source, retained and
other-family controls are mixed. These are scoped diagnostics, not the release
gate, and the two change statistics must not be interchanged.

[First-use checks](first-use.json) cover 38 conditions with 15 trials, 1000 ms
warmup and fresh documents/disposal inside the timer. Unchanged and edited
first renders, first existing-entity method invocation, output and retained
object signatures pass. First-use timings remain mixed: ER first-invoke ranges
from -18.376% to +41.167% paired; ER medium edited first render is about +1%.
There is no uniform first-use improvement. Resource-callback conditions are
explicitly outside this successful run because the added default callback
assertion exposes the baseline defect below; the assertion remains enabled in
the harness and no successful callback timing is claimed.

The new [complete web matrix](web.json), from this exact candidate, still fails
[all 126 conditions](gate.json). Median WASM/JS ratios are **2.40 times source**,
**10.94 times parse**, and **4.03 times retained mutable render**. Source ratios
range 1.58–5.00. Seven families, three sizes, two resources and three stages are
measured with nine alternating trials after at least 500 ms warmup. This covers
126 performance conditions, not all 33 families. Historical matrix differences
are not causal speedup percentages. **0.3 remains unreleased.**

## Compatibility and remaining blockers

- 28 native public integrations pass, including all 17437 frozen ER cases,
  byte-identical graphs/errors, source/title eligibility boundaries, real graph
  limits, captured providers and first use/retained aliases.
- Eight local Swift ER/eager cases pass: 7535 mutable drawings, 9902 error cases,
  zero selected-family budget differences. All 168 real Swift API conditions and
  three strict ARM64 C programs pass.
- 27 browser checks pass in Chromium, Firefox and WebKit, including the ER
  corpus and all selected public-model integrations. These targeted checks
  supplement the preceding full 33-family checkpoint.

The independently reproduced [resource callback defect](RESOURCE_CALLBACK_EVIDENCE.md)
remains open. Host edits during text measurement can be lost on publication;
future attribute/style/direction edits can affect the JS current render while
Rust reads a stale typed snapshot. Nested same-model methods can reject. The
[reproducer](resource-callback-diagnostic.mjs) and [84 comparisons](resource-callback-diagnostic.json)
preserve both matches and mismatches. Direct parsing does not repair this
architecture. This prevents a claim of full API parity or release readiness.
Existing AppKit activation/focus failures and current Intel SDK verification
also remain open.

## Reproduction

Use the repository-pinned oracle and production packages. On the VM, one
browser benchmark runs at a time; frozen control/candidate package directories
must match the hashes embedded in each report. Run from the repository root:

```bash
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/compare-web-vm.mjs \
  --control parse-architecture/normalized-er/package \
  --candidate parse-architecture/direct-er/package \
  --cases sequence/small,er/small,er/medium,er/large,gantt/medium \
  --profiles fixed,canvas --trials 21 --warmup-ms 1000 --output comparison.json
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/compare-first-use-web-vm.mjs \
  --control parse-architecture/normalized-er/package \
  --candidate parse-architecture/direct-er/package \
  --cases sequence/small,er/small,er/medium,er/large,gantt/medium \
  --stages render-source,parse-first-render,parse-first-edited-render,parse-first-invoke \
  --profiles fixed,canvas --trials 15 --warmup-ms 1000 --output first-use.json
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/web-vm.mjs \
  --package parse-architecture/direct-er/package --trials 9 --warmup-ms 500 --output web.json
node scripts/perf/check-web-parity.mjs web.json parse-architecture/direct-er/package
```

The last command is expected to fail for this artifact. Native and Swift exact
commands, artifact manifests, integer samples, guards and exclusions are retained
in the linked raw reports/controllers. No guard bypass, performance-outcome
retry selection or release publication is part of this checkpoint.
