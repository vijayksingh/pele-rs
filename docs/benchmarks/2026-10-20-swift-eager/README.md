# Eager Swift projection checkpoint

Source `364c509`. [Architecture](../../LLD_SWIFT_EAGER_PROJECTION.md) keeps
`Pele.Engine.parse` eager and throwing, with the same immutable `Document.model`
and retained document operations. A synchronous, bounded C event stream replaces
the second complete model JSON buffer and Foundation's recursive model decode.
The first bounded native model JSON normalization and exact envelope-size
counting remain; their CPU work is not claimed removed. Mutable/provider paths
and public Swift signatures remain unchanged.

## Measured motivation

[Control provenance](control-profile.json),
[Gantt sample](control-gantt.sample.txt.gz),
[Sequence sample](control-sequence.sample.txt.gz) and
[attribution](control-attribution.json) record the old frozen Swift consumer.
Inside the public exercise call, Foundation JSON decoding appears in 90.05% of
Gantt-medium samples and 87.43% of Sequence-small samples. Inclusive categories
overlap. These sampling percentages locate the old bottleneck; they are not
latency percentages or predicted speedups.

## Completed matched timing

Three separate guarded runs cover parse, source render and typed retained render.
Each row has 15 matched pairs and at least 1000 ms measured warmup per executable.
The [independent Astra audit](TIMING_REVIEW.md) recomputed every pair, checked both binary/manifests,
current candidate inputs, oracle and executed harness hashes.

| Workload / API | Previous Rust Swift API | Current Rust Swift API | Candidate / control median | Faster pairs |
| --- | ---: | ---: | ---: | ---: |
| Sequence small parse | 0.308373 ms | 0.058730 ms | 0.1905 | 15/15 |
| Gantt medium parse | 5.308823 ms | 0.845084 ms | 0.1592 | 15/15 |
| Sequence small source render | 0.082231 ms | 0.082172 ms | 0.9993 | 8/15 |
| Gantt medium source render | 0.435927 ms | 0.431550 ms | 0.9900 | 12/15 |
| Sequence small retained render | 0.068515 ms | 0.068635 ms | 1.0018 | 8/15 |
| Gantt medium retained render | 0.368180 ms | 0.368146 ms | 0.9999 | 7/15 |

Raw reports: [parse](swift-parse-timing.json), [source](swift-source-timing.json),
[retained](swift-retained-timing.json). The paired parse medians are 81.05% and
84.02% lower. Source/retained medians stay near flat, with individual outliers;
these are not bounded-noise claims. Broad attempts interrupted by external
compiler activity were rejected, not pooled into these completed reports.

The exact [executed comparator](measured-compare.mjs) has SHA-256
`a3b3d82f8a36e2d411851efec25537f60ec83d7f5f862ef7db49c43ef5b1ed3f`.
To reproduce that revision, restore it at `scripts/perf/swift/compare.mjs`;
its imports depend on that location. The maintained comparator removes an unused
variable and redundant overwritten summary calculations; raw reports are unchanged.

[Candidate profile](candidate-profile.json),
[attribution](candidate-attribution.json),
[Gantt samples](candidate-gantt-medium.sample.txt.gz) and
[Sequence samples](candidate-sequence-small.sample.txt.gz) use the same verified
candidate executable. Foundation appears in 0.87% of Gantt samples and 8.99% of
Sequence samples; recursive ConfigValue JSON decoding disappears from both.
The C entry includes synchronous Swift callbacks: its samples cannot be called
Rust-only CPU. Gantt model projection still appears in 37.58% of samples,
Swift event receiving in 39.44%, and envelope counting in 4.21%; inclusive
categories overlap. These locate the next investigation, not phase timings.

## Compatibility evidence

[Validation](validation.json) retains both successes and limitations:

- [Seven focused public integrations](swift-eager.log.gz) pass: legacy C versus
  Swift number bits, canonical key spelling, embedded NUL/non-BMP/U+FEFF strings,
  deep containers and exact depth admission, batch boundaries, eager budget/error
  timing, config, retained render/query, detached models and later-batch abort.
- The [full Swift run](swift-full.log.gz) passes the 33-family fixture slice,
  including 167,473 mutable drawings and 270,358 error cases, with five existing
  explicit transport-budget differences. The same production sources were used
  for the later focused depth-boundary check.
- The [real Swift consumer](swift-verified.json) verifies 168 conditions across
  21 workloads and eight APIs, with exact SVG/dimensions and source/SDK hashes.
- All 324 Chromium/Firefox/WebKit checks pass. A fresh WASM rebuild is
  [byte-identical across every measured runtime file](wasm-rebuild.json), so the
  independent VM report still identifies the current web package.

The full Swift suite **does not pass**: 19 of 126 test cases fail 53 assertions
in `NativeResourceUITests`, `PresentationUITests` and `ZoomTests`. The host stays
inactive and windows fail to become key; focus/keyboard-dependent assertions
also fail. A bare AppKit process, without loading Pele, reproduces a visible
window with `active=false` and `key=false` after activation and event pumping.
No UI assertion was weakened or skipped. Actual native activation/focus delivery
remains unverified; this is not a release-ready Swift claim. The SDK is ARM64;
universal Intel validation also remains outstanding.

Full-corpus validation caught a real handoff defect before commit: Foundation's
`String(bytes:encoding:)` consumes a leading UTF-8 byte-order mark. Model U+FEFF
is content. The adapter now uses standard UTF-8 decoding with exact byte
round-trip validation on macOS 14; dedicated public fixtures protect keys and
values. The subsequent full 33-family model corpus passes.

## Reproduction and scope

Build a source-matched host SDK with `scripts/build-native.sh --host`, then
`node scripts/perf/swift/build.mjs`. Before replacing a control build, copy its
executable as `PeleSwiftPerf` and its unchanged `swift-build-manifest.json` into
a separate directory. Do not use an old manifest with a newly built executable.

`node scripts/perf/swift/compare.mjs --control-dir PATH --quiet-run` compares the
two real consumers with 15 balanced pairs, 1000 ms measured warmup per binary,
identical batch counts and compiler/Pele-test guards before every measurement.
Without `--quiet-run`, it runs correctness only. `--cases` selects exact workload
IDs; `--stages` selects APIs. Defaults are Gantt-medium and Sequence-small,
through all eight APIs. Warmup starts with six count-one pairs, then uses matched
workload-bounded batches of at most 128 until both measured-time targets pass.
Complete source/SDK and both executable/manifest hashes are checked; exact
outputs and cross-binary checksums/arena counts are compared before and after.

Control source `02ee639` predates direct Gantt construction. Comparisons identify
all changed inputs and measure combined executable differences; mutable Gantt
changes cannot be attributed solely to the eager handoff. The consumer includes
parse and disposal and keeps model creation eager, but does not separately time
`decodeModel` or model access. No cross-machine JS ratio or allocation/RSS claim
is supported by these Swift timings.

**The web performance gate still fails 126/126 conditions. No 0.3 release.**
