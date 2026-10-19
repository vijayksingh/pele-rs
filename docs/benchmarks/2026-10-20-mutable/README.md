# Mutable transport checkpoint

Measured web source: `a7696d9`, Rust core `9c6995d`, before the subsequent
fresh-Flowchart record-construction experiment. No v0.3 release or JS parity.

## Changes and boundary

- Flowchart/ER typed projections initialize only when rendering needs them.
- All 33 built-ins borrow native graphs for binary snapshot transport and
  discarded pre-operation validation. Owned public outcomes remain owned.
- Only explicitly opted-in modules may omit applying an exactly acknowledged
  graph. Failed scans/imports/transport leave it unacknowledged and force retry.
- Fresh publication skips unnecessary cleanup; existing publication compares
  descriptors before redefining identical values. All callback mutations still
  undergo full publication/restoration. Duplicate keys, `__proto__`, readonly
  array length, detached aliases and raw number transport remain supported.

The preceding coordinate optimization is already in the control. No packed
publication, storage-release, O3 or schema-cache experiment is in this artifact.

## Independent VM measurements

`mutable-layers-control.json` is a fresh run of the preceding coordinate package;
`mutable-layers.json` is the candidate. Both use the same current harness and
pinned Pele 1.6.0 oracle on the independent Linux `vm-agent` host: AMD EPYC-Genoa,
8 vCPUs, 16 GiB, Chromium 156.0.8078.4. Shared VM services stayed running.

21 workloads × two measurers × three public API stages = 126 conditions. Nine
alternating JS/WASM paired trials, at least 500 ms/6-operation warmup, calibrated
12 ms batches capped at 128. Retained state is fresh per batch, with setup/warm
render outside the timer. Public parse includes explicit disposal. Complete SVG
and exact dimension comparisons run before and after timings. Warm CPU work
excludes startup, transfer, DOM/SVG painting and GPU work.

`comparison.json` gives candidate/control changes. Stage medians are conventional
medians of 42 case/profile percentage changes, not pooled operation timings.

| Stage | Median time reduction | Candidate median WASM/JS | WASM/JS range |
| --- | ---: | ---: | ---: |
| Source render | 0.0% | 2.68× | 1.74–5.27× |
| Mutable parse/dispose | 14.6% | 13.08× | 3.73–69.00× |
| Retained mutable render | 23.0% | 4.79× | 2.23–22.36× |

All 42 retained-render conditions improved, ranging from 0.8% to 40.1%.
Parse regressions above 5%: Sequence/small/Canvas 16.2%, ER/large/Canvas 23.9%,
Railroad/large/fixed 15.5%. Six source conditions also regressed more than 5%;
the stage median is unchanged. These regressions are not hidden or declared
resolved. Raw ER parse batches show substantial GC variation (candidate
3.47–9.93 ms per operation); focused diagnostics are not a replacement for a
complete repeated acceptance matrix. Both distributions and every raw sample
remain available. Candidate/control runs are sequential shared-host observations,
not statistical proof that every difference is caused by the patch.

`release-gate.json` fails **126/126** conditions against matching JS medians.
Reproduce the complete matrix and strict artifact check:

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/web-vm.mjs \
  --package mutable-layers/package --warmup-ms 500 --output mutable-layers.json
node scripts/perf/check-web-parity.mjs REPORT.json EXACT_PACKAGE_DIRECTORY
```

The report records hashes of all eight runtime files tracked by the harness,
including the actual WASM; the release check rejects a different artifact or
partial matrix. These historical reports require the matching source/package,
not a later candidate silently substituted at the same directory.

## Functional verification

11 borrowed-snapshot/native-contract integration tests passed, including all
33 families through success, dispatched output-limit failure and recovery with
and without unchanged apply; extension export fallback, stale revisions,
malformed post-state and binary stress remain checked. Earlier lazy projection
checks additionally covered Flowchart/ER/Swimlane operation and partial errors.

90 public browser integration checks passed across Chromium, Firefox and WebKit.
They cover all 33 mutable families, numeric cases, nested Class/Gantt callbacks,
exact host exceptions, repeated rejected uploads/scans, restoration, descriptors,
special keys, collection identities, aliases and complete pinned SVGs. TypeScript
consumer checking also passed.

The universal macOS SDK was rebuilt from this native core. Four local Swift
`RenderNumberTests` passed: 1,023 cases across all 33 families through source,
immutable and mutable lanes (3,069 all-family drawings), the additional 287-case
numeric corpus, and Date/error/recovery checks. `swift-verified.json` records
21 shared workloads through eight actual Swift APIs: 168 correctness checks,
with exact SVG hashes, sizes and dimension bits. The consumer verifies source
hashes and byte identity of the linked host slice against the freshly built
explicit-target archive.

`swift-timing-status.json` records refusal before timing because unrelated
compilation was active. No samples from that attempt are published. Correctness
builds may share the host; source/SDK freshness checks remain mandatory. Timed
runs still require the compiler/Pele-test quiet guard. The older README Swift
latencies remain explicitly historical.
