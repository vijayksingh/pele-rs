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
