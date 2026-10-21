# Static projection policy checkpoint

Source `ce559f1`. The preceding normalized Gantt rollout stored a runtime
strategy tag beside every builtin handler. That changed zero-sized unit handlers
into heap-allocated boxes. A private zero-sized type policy now selects the same
projection through the erased module's concrete type/vtable. Public providers,
captured registrations, model validation and serialization rules are unchanged.
[Architecture](../../LLD_NORMALIZED_PROJECTION.md), [source review](REVIEW.md),
[independent results audit](RESULTS_REVIEW.md), [validation](validation.json).

## Allocation evidence

The public `cargo run --release -p pele-rs --example registry_allocations`
diagnostic brackets builtin construction after warmup. Five repeated captures
are identical for each binary:

| Counter | Runtime tag control | Static policy |
| --- | ---: | ---: |
| Allocation calls | 69 | 36 |
| One-byte allocation requests | 33 | 0 |
| Reallocation calls | 4 | 4 |
| Cumulative requested bytes | 3,948 | 3,915 |

Raw [control](control-registry-allocations.json) and
[candidate](candidate-registry-allocations.json) captures retain all five rows;
[control build](control-registry-manifest.json) and
[candidate build](candidate-registry-manifest.json) identify binaries, three
relevant source hashes and the exact instrumented build command. These focused
manifests are not complete dependency/toolchain aggregates. Requested bytes
include whole replacement sizes for reallocations. They are not allocator size
classes, retained memory, RSS or elapsed time. Only construction is captured;
destruction and capability checks are outside capture. The registry still owns
its Arc records and vector, so it is not allocation-free.

## Matched local Swift timing

The frozen control is the previous normalized Gantt binary. Only engine.rs and
the rebuilt CPele archive differ. All six shorter windows complete on their
first attempts after the full Swift corpus ends. Each row has 15 matched pairs,
1000 ms measured warmup per executable and compiler/Pele-test guards.

| Case / API | Control median ms | Candidate median ms | Paired change | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| Sequence small parse | 0.059501 | 0.058422 | -2.163% | 13/15 |
| Gantt medium parse | 0.605967 | 0.600736 | -0.862% | 13/15 |
| Sequence small source render | 0.082598 | 0.081340 | -1.522% | 10/15 |
| Gantt medium source render | 0.435307 | 0.435230 | -0.057% | 8/15 |
| Sequence small retained render | 0.068224 | 0.068418 | -0.287% | 9/15 |
| Gantt medium retained render | 0.367279 | 0.364525 | -0.128% | 8/15 |

Raw [Sequence parse](swift-single-parse-sequence-small-attempt1.json),
[Gantt parse](swift-single-parse-gantt-medium-attempt1.json),
[Sequence source](swift-single-render-source-sequence-small-attempt1.json),
[Gantt source](swift-single-render-source-gantt-medium-attempt1.json),
[Sequence retained](swift-single-render-retained-sequence-small-attempt1.json),
[Gantt retained](swift-single-render-retained-gantt-medium-attempt1.json).

`--order-offset 0/1` alternates the first-side imbalance between single-case
windows: exactly 45 control-first and 45 candidate-first trials overall. The
[executed comparator](measured-compare.mjs) and
[window controller](run-single-windows.mjs) are archived; restore the comparator
at scripts/perf/swift/compare.mjs for relative imports. Four earlier two-case
attempts were compiler-interrupted and excluded entirely, including completed
rows inside incomplete reports. [Rejected attempts](quiet-windows.json) retain
their logs and the [previous comparator](measured-before-order-offset.mjs) hash.
The [successful selection log](single-windows.json) records no performance-based
selection. Shorter windows change scheduling, not guard criteria or warmup.

Separate medians and paired medians differ. A Gantt retained pair is +400.46%
despite its near-flat paired median; controls are not uniformly stable. Guard
snapshots do not establish an idle desktop. These scoped observations support
neither all-family speedups nor JS parity. Earlier gains are not multiplied.

## Independent VM

[Controlled comparison](web-comparison.json) against the runtime-tag candidate
uses 21 trials, 2000 ms warmup and fixed/Canvas resources. Gantt source paired
medians are -1.28%/-0.66%, each 13/21 faster. Parse remains mixed.

[Comparison against the earlier direct-Gantt package](web-before-normalization.json)
uses the same protocol. Gantt source paired medians are +0.14% fixed and -0.36%
Canvas, with 8/21 and 13/21 faster respectively. This returns source medians near
the earlier control, but does not establish absence of regressions: Gantt fixed
parse is +3.53% paired, only 6/21 faster; Canvas parse is +1.11%, 8/21 faster.
All original rows remain published. Both control packages and the candidate
independently match all 105 reported file hashes; only WASM differs.

The separate [full gate](gate.json), backed by the complete [matrix](web.json),
still fails **126/126 conditions**. Stage median WASM/JS ratios are **2.39×
source, 11.34× parse and 4.19× retained**. The gate uses nine alternating trials,
500 ms warmup and the actual package. Separate gate runs are not causal
comparisons. VM: AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4. Timers
exclude startup/download, painting and GPU. **No 0.3 release.**

## Compatibility checks

- [20 native public integration cases](native-integration.log.gz), including
  captured registration/provider isolation, complete Gantt projection behavior,
  exact model byte boundaries and all 33 plain-model construction families.
- [Current full 33-family Swift corpus](swift-corpus.log.gz), plus seven eager
  integrations: eight cases pass in 652.187 seconds. Corpus includes 167,473
  mutable drawings and 270,358 error cases, with five existing explicit
  transport-budget differences. These differences remain asserted, not waived.
- [All 168 real Swift consumer API conditions](swift-verified.json), across
  21 workloads and eight APIs, match pinned SVG/dimensions with exact source/SDK
  hashes. The model remains eager and detached values retain their contract.
- [Nine full browser corpus checks](web-corpus.log.gz) cover ER, Gantt and
  Sequence in Chromium, Firefox and WebKit; [three identity checks](web-identity.log.gz)
  preserve complete Gantt native identity tables in those engines.
- Three current strict ARM64 C programs pass ownership, public models and parse
  limits. TypeScript checking and diff checks pass.

The old full-suite AppKit activation/focus failures are still open; this selected
corpus run does not claim UI acceptance. Current SDK is ARM64, not current Intel
validation. The earlier 324-browser checkpoint is archived separately. Release
acceptance remains blocked by JS performance and these remaining delivery checks.
