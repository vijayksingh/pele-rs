# ER normalized immutable projection

Source `4e9faee` selects the existing private normalized projection policy for
captured builtin ER. This removes its full model JSON buffer and recursive
reparse before eager Swift projection. Public providers keep the original
single-serialization path. No formatter, public interface or mutable web graph
construction changes. [Architecture](../../LLD_NORMALIZED_PROJECTION.md),
[independent source review](REVIEW.md), [results audit](RESULTS_REVIEW.md),
[validation](validation.json).

## Matched Swift consumer

Frozen ce559 control and current source-built ARM64 candidate differ only in
engine.rs and the CPele archive. Each window has 15 matched pairs, 1000 ms
measured warmup per binary and the same consumer, workload and SDK compiler.

| Case / API | Control median ms | Candidate median ms | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| ER medium parse | 0.848445 | 0.621891 | -25.669% | 15/15 |
| Sequence small parse | 0.061209 | 0.061271 | +1.698% | 5/15 |
| ER medium source render | 1.550229 | 1.565438 | +0.233% | 7/15 |
| Sequence small source render | 0.084794 | 0.085701 | +1.316% | 5/15 |
| ER medium retained render | 1.402742 | 1.421430 | +0.946% | 6/15 |
| Sequence small retained render | 0.069115 | 0.069853 | +2.071% | 5/15 |

[Summary and raw report links](summary.json), [window selection](single-windows.json),
[executed comparator](measured-compare.mjs), [controller](run-single-windows.mjs).
Reports retain raw samples, paired ratios, guards, output hashes and complete
source/SDK manifests. Separate medians and paired medians differ. First-side
order totals 45/45. Four compiler-interrupted attempts are excluded entirely;
their logs remain alongside completed reports. No unrelated build was stopped,
no guard bypassed and no timing result selected by its performance outcome.
These are shared-host observations, not confidence intervals, an idle-desktop
claim or general JS parity. The control slowdowns are retained. Earlier gains
are not multiplied.

## Independent VM

The [controlled comparison](web-comparison.json) runs 21 balanced three-way
trials, 2000 ms warmup, matching WASM batch counts and both fixed/Canvas metrics.
It covers ER medium, Gantt medium and Sequence small across source render,
parse/dispose and mutable retained render. Complete SVG/dimension checks precede
and follow timing. All 105 candidate package files match the actual package;
control bytes identify the preceding static-policy checkpoint.

The ordinary web mutable parse path does not call immutable model projection.
Results remain mixed: ER fixed parse is +2.66% paired, only 6/21 faster; Canvas
parse is -0.89%, 11/21 faster. ER source paired changes are -0.24%/-0.88% and
retained changes -1.16%/-1.11%. Other-family parse controls are mixed too. This
establishes no web parse improvement and no new complete release gate. The
[preceding complete matrix](../2026-10-20-projection-policy/) still fails all
126 conditions. **No 0.3 release.**

## Compatibility

- [24 native public integrations](native-integration.log.gz), including all
  17,437 frozen ER cases through actual normalized/legacy model projections,
  complete envelope/errors, exact byte boundaries and captured provider rules.
- [Swift ER corpus and seven eager integrations](swift-er-corpus.log.gz):
  eight cases pass; 7,535 mutable drawings, 9,902 error cases, zero budget
  differences in this selected family. This is additional to the previous
  complete 33-family Swift checkpoint, not a new all-family execution.
- [All 168 real Swift API conditions](swift-verified.json) pass, with exact
  current source/SDK hashes and complete output comparisons.
- [Nine browser corpus checks](web-corpus.log.gz) pass ER/Gantt/Sequence in
  Chromium, Firefox and WebKit; three strict current ARM64 C programs, TypeScript
  checking and diff checks pass.

Existing AppKit activation/focus failures and current Intel verification remain
open. ARM64 corpus/API success is not UI, universal-SDK or release acceptance.

## Additional baseline callback failure

A new ER first-use check finds an existing compatibility failure before the
next direct-construction patch: a cyclic host object assigned to the model
inside its first measuring callback loses identity when native render publishes
its graph. Pinned JS passes; the ce559 native control fails. The combined check
aborts before timing. [Exact reproduction](callback-baseline-failure.json),
[failure log](callback-baseline-failure.log.gz),
[executed harness](measured-first-use-check.mjs). This remains an open release
blocker; no assertion was removed and no callback timing is claimed.
