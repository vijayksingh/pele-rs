# Direct normalized Gantt projection

Source-equivalent commit `6b2d593`; [exact runtime hashes](source-commit.json).
The saved consumer manifest identifies its actual build checkout `058fcaa` plus
three measured runtime source changes and the rebuilt SDK. Reports retain that
original provenance. The [architecture](../../LLD_NORMALIZED_PROJECTION.md)
removes Gantt's first full JSON buffer and recursive normalization parse. The
normalized Value and eager Swift model remain; providers and other builtin
families keep the original projection. Public APIs and budgets are unchanged.

## Matched local Swift results

Frozen control `364c509` is the previous eager Swift binary. Exactly four input
records differ: three Rust runtime files and CPele archive. Consumer code,
workloads, compiler, architecture, SDK and oracle match. Each API/case has a
separate compiler-guarded window, 15 matched pairs and at least 1000 ms measured
warmup per executable. All outputs and lifetimes are verified before/after.

| Case / API | Control ms | Candidate ms | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| Gantt medium parse | 0.858211 | 0.629154 | -26.69% | 15/15 |
| Sequence small parse | 0.059591 | 0.059992 | +0.70% | 7/15 |
| Gantt medium source render | 0.443055 | 0.441564 | +0.26% | 6/15 |
| Sequence small source render | 0.082368 | 0.083354 | +0.58% | 4/15 |
| Gantt medium retained render | 0.366394 | 0.367415 | +0.40% | 3/15 |
| Sequence small retained render | 0.069865 | 0.069023 | +0.10% | 7/15 |

Raw reports: [Gantt parse](swift-parse-gantt-medium-attempt2.json),
[Sequence parse](swift-parse-sequence-small-attempt2.json),
[Gantt source](swift-render-source-gantt-medium-attempt2.json),
[Sequence source](swift-render-source-sequence-small-attempt2.json),
[Gantt retained](swift-render-retained-gantt-medium-attempt2.json),
[Sequence retained](swift-render-retained-sequence-small-attempt2.json).

The Gantt parse result includes eager model construction and disposal. Its least
favorable pair is still 23.44% lower; one control outlier produces a -90.31% pair.
No confidence interval, all-family speedup, JS ratio or allocation/RSS result is
claimed. Separate medians and paired medians are not interchangeable.

Each single-row window restarts order: 8 control-first/7 candidate-first, giving
48/42 overall rather than 45/45. Host load remains shared; process guards are
snapshots, not continuous machine isolation. The [window log](quiet-windows.json)
retains the first permission failure before either consumer exercised a case,
then all six first successful windows. The [controller](run-quiet-windows.mjs)
retries only explicit compiler/test guard failures, never performance outcomes.
The exact [measured comparator](measured-compare.mjs) is archived; restore it to
`scripts/perf/swift/compare.mjs` for its relative imports.

## Independent VM results

[Full gate](gate.json), backed by [complete raw matrix](web.json), still fails
**126/126 conditions**. Matrix median WASM/JS ratios are **2.46× source**,
**11.67× parse**, **4.11× retained**. These describe this compiled artifact and
run; comparisons of separate gate runs are not causal slowdown percentages.

The [24-row controlled diagnostic](web-comparison.json) compares independent
frozen packages on the same Linux VM with 15 trials and 1000 ms warmup. Only WASM
bytes differ between packages. It does not establish a browser improvement or
absence of regressions:

- Gantt parse paired medians vary from -1.73% to +3.05%, with 4–9/15 faster.
- Sequence canvas parse has +22.81% separate medians but -0.60% paired median,
  8/15 faster. Both statistics are correct; neither licenses ignoring the other.
- Gantt medium fixed source render is +2.36% separate medians and +1.70% paired,
  only 1/15 faster. This consistent slowdown observation remains disclosed.

A [follow-up diagnostic](web-followup.json) retains all eight rows from a fresh
browser process, 21 trials and 2000 ms warmup. Sequence canvas parse is +2.11%
separate and +2.17% paired, 10/21 faster; the previous +22.81% separate figure is
not reproduced. Gantt medium source remains slower: fixed +2.02% paired, 3/21
faster; canvas +2.43%, 1/21 faster. These repeated source observations remain an
open regression. No run replaces or hides the preceding evidence.

Source inspection identified a plausible concrete contributor: adding a runtime
strategy enum to the boxed unit builtin module changes it from zero-sized to
nonzero-sized. Recreating Engine::with_builtins can consequently allocate one
additional handler box per builtin. A static-policy correction is separate work;
these measurements describe the runtime-enum candidate before that correction.
This source observation is not an isolated allocation or elapsed-time benchmark.

The diagnostic shares a browser process/GC/cache across independent instances.
The separate gate measures pinned JS versus the actual package. Both verify
complete SVG and exact dimensions before/after and hash their runtime inputs.
The VM is AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4. No GPU work or
paint is measured. No 0.3 release.

## Public compatibility checks

- [14 targeted native integrations](native-integration.log.gz): six new public
  projection tests, historical Gantt corpus and module/provider contracts.
- [10 additional native integrations](native-contracts.log.gz), including all
  33 declared plain-model families, source preparation and registry behavior.
- [Eight Swift integrations](swift-integration.log.gz): complete Gantt corpus
  (8,048 drawings, 10,946 errors, no explicit budget differences) and seven eager
  handoff integrations. Exact 16 MiB/depth/error/Unicode checks remain eager.
- [Real Swift consumer](swift-verified.json): all 168 conditions across 21
  workloads and eight APIs match pinned outputs with current source/SDK hashes.
- [Six browser checks](web-integration.log.gz): complete Gantt corpus and native
  identity tables in Chromium, Firefox and WebKit.
- Three strict ARM64 C consumer programs pass: ownership, public models and
  independent parse limits. No C ABI or Swift signature changed.

The prior full 33-family Swift and 324-browser checkpoint is preserved
[separately](../2026-10-20-swift-eager/README.md); the current layer was validated
with the targeted slices above. Previously failing AppKit activation/focus
assertions and current Intel validation remain open. This is not full release
acceptance. Provider raw/f32/depth/custom-error tests are isolation controls,
not claims that unreachable formatter branches were exercised by builtin Gantt.

## Profiles and review

[Source review](REVIEW.md), [independent results audit](RESULTS_REVIEW.md), [profile provenance](candidate-profile.json),
[attribution](candidate-attribution.json), [Gantt samples](gantt-medium.sample.txt.gz)
and [Sequence samples](sequence-small.sample.txt.gz) retain exact evidence.
Gantt's new projector appears in 16.61% of exercise samples; Swift eager callbacks
in 50.90%, envelope counting in 5.79%. Inclusive categories overlap. No legacy
Gantt projector was sampled; that does not prove zero fallbacks. Sequence still
samples legacy projection. Symbol inlining prevents interpreting a missing
parser name as proof that legacy JSON parsing never runs. The callback's larger
fraction does not establish higher absolute callback time.

Build with `scripts/build-native.sh --host`,
`node scripts/perf/swift/build.mjs` and `scripts/build-wasm.sh`. Freeze original
executables/manifests before rebuilding; use `compare.mjs --control-dir PATH
--cases gantt/medium --stages parse --quiet-run` for Swift. VM reproduction uses
`compare-web-vm.mjs` for matched diagnostics, then `web-vm.mjs --trials 9
--warmup-ms 500` and `check-web-parity.mjs` for the complete release gate.
