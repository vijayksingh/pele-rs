# Browser CPU profiles

Baseline: main `e5bd5f5`, default size-optimized WASM, SHA-256
`0314e866996d2bd08934f26a1a07c375a006afb1459c72e9b9c33714ddb3a1d7`.
Measured on the independent `vm-agent` Linux AMD EPYC-Genoa host, using
Chromium 156.0.8078.4 and the pinned Pele 1.6.0 oracle. Workloads and Canvas
measurer match the latency harness. Existing shared VM services stayed running.

`summary.json` retains self-time summaries for 18 profiles: Flowchart, ER,
Gantt; large source render, mutable parse, and mutable retained render; JS and
WASM separately. Three compressed raw source-render WASM profiles are retained
here for independent call-tree inspection. Full raw profiles remain in
`/home/admin/pele-perf.jL9wzP/profiles-current` on the VM.

CDP samples at 250 microseconds after 200 ms warmup per runtime, for 1.5 seconds
per profile. Complete source SVG and dimensions match before profiling. CPU
profiling changes execution overhead: operation counts are diagnostic, **not
latency acceptance results**. Use `scripts/perf/web-vm.mjs` for paired timings.
These initial captures preceded the profiler's added post-run checks and
complete runtime hash metadata; the exact WASM hash and browser version are
recorded in their summary.

Measured cumulative source-render sample shares:

| Path | Flowchart | ER | Gantt |
| --- | ---: | ---: | ---: |
| SVG coordinate formatting (`svg::num`) | 11.23% | 11.37% | 3.62% |
| Label layout | 7.03% | 13.89% | 6.40% |
| SVG root generation | 9.71% | 8.70% | 7.38% |
| Render result JSON envelope | 5.67% | 5.81% | 3.25% |

These are inclusive call-tree shares; nested rows must not be summed. Mutable
parse spends 20.88% (Flowchart) and 28.48% (ER) of self samples in JS
`reconcile`. Mutable render also spends substantial time scanning, serializing,
and reconciling its externally mutable graph. Arbitrary host callback order,
object identity and detached aliases remain compatibility requirements.

The first isolated optimization removes two-stage `format!` coordinate
construction while preserving the existing rounding and non-finite branches.
No release readiness or JS parity follows from these profiles.

Reproduce from the VM workspace, using the published-package directory:

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/profile-web.mjs \
  --package current/package --output profiles-current
```

To inspect a raw profile, decompress its `.cpuprofile.gz` file and load it in
Chrome DevTools' JavaScript profiler.

## Coordinate-format optimization

The candidate writes the existing rounded hundredths directly into a stack
buffer, then creates one owned string. No numeric rounding, zero, non-finite,
callback, model or public API policy changes. Default WASM optimization remains
`s`; no schema-cache or O3 experiment was applied.

The complete 126-condition VM benchmark was run for the candidate, then a fresh
unmodified baseline control, then the candidate again. All three raw reports
are retained: `coordinate.json`, `coordinate-control.json`, and
`coordinate-confirmation.json`. Every run verifies complete SVG and dimensions
before and after each workload/resource pair. Package hashes freeze inputs.

Against the adjacent control, confirmation reduces the median of case-wise
Canvas source-render times by 10.9% (9.8% when normalized to each run's JS
control). Fixed-metric source rendering improves by 9.5% raw / 8.5% normalized.
These are medians of relative changes across cases, not pooled operation times.
Large Canvas source renders improve 2.3–16.4%. Mutable parse is not the target
of this change; no parse improvement is claimed.

Two source-render conditions exceed the fresh control by more than 5%:
Flowchart/small/fixed +7.1%, Sankey/small/Canvas +8.9%. Small results and GC/JIT
variance remain visible in the raw reports. This is not a regression-free or
universal speed claim. The first candidate run showed larger improvements;
the adjacent control/confirmation pair is used for the current headline.

Canvas source rendering still takes 1.75–4.76 times JS (median case ratio 2.54).
`release-gate.json` fails all 126 conditions, including mutable parse/render.
**0.3 must not be released.** The new checker validates the whole matrix,
source and harness hashes, warmup/trials, and the actual package bytes before
checking each WASM median against its matching JS median:

```sh
node scripts/perf/check-web-parity.mjs   docs/benchmarks/2026-10-19-profile/coordinate-confirmation.json   dist/package-web/package
# Expected exit 1 until every measured condition matches or beats JS.
```

Functional checks: six Rust numeric integration tests, including all 1,023
all-family numeric cases through source/immutable/mutable lanes; twelve browser
checks covering all 33 families and numeric/error/callback behavior across
Chromium, Firefox and WebKit. The universal native SDK was rebuilt from the optimized core. Four local Swift
`RenderNumberTests` passed, including all 1,023 cases across 33 families through
source, immutable and mutable rendering (3,069 all-family drawings), plus the
287-case numeric fixture and explicit Date/error/recovery checks.


## Local Swift validation and timing boundary

The current standalone Swift consumer verifies all 21 shared workloads through
eight public API paths (168 rows), including immutable and mutable ownership,
graph snapshot/apply, rendering, and disposal. See `swift-verified.json` for
exact source/SDK/binary input hashes and complete SVG/dimension comparisons.
This report contains **correctness only**, not latency samples.

A new timed run aborted after one workload because `guardQuiet` detected a
concurrent Kamui `swift-build` process. Partial samples were discarded; no
unrelated process was stopped. `swift-timing-status.json` records the boundary.
The root README keeps earlier Swift baseline timings explicitly labelled;
no current Swift speed improvement is claimed. Retry the existing Swift timing
command when the local build/profiling work is idle.
