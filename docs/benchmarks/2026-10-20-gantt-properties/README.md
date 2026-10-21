# Native Gantt property storage

This checkpoint replaces operation-local hash indexes for small Gantt records
with bounded scans of their existing arena storage. Records above 32 properties
retain randomized hashing; the 33rd append promotes the index. Property values,
descriptors and identities remain owned by the graph. See the
[design and semantic ledger](../../LLD_GANTT_PROPERTIES.md).

## Matched native comparison

The control is the borrowed-lookup implementation (`712ed1f` runtime). Its
source-input hash is `29011327a6530e979e5497421de53ecb8eda721dfa97cc7df818de16141167ef`.
The only production change in the candidate is adaptive property lookup in
`public_model.rs`, SHA-256
`b7cdd2671c322aab3410632554fb2d5733375c8ae180a64ea69e736f6361fe4d`.
Reports pin both complete source-input and binary hashes; their manifest Git
revisions precede source commits and must not substitute for those hashes.

The independent release consumers run on the same M1 Pro. Each condition has
15 balanced three-way trials with pinned Pele JS, a 1,000 ms work warmup,
matched native batch counts, fresh public state per batch, and whole output
checks before and after. Timers run inside each runtime; process control and
verification stay outside. The host guard rejects Rust/Swift builds and Pele
tests. This shared computer still has unrelated interactive load.

| Native stage | Control ms | Candidate ms | Change |
|---|---:|---:|---:|
| Typed parse, count 1 | 0.061792 | 0.061416 | -0.6% |
| Public parse, count 1 | 0.196459 | 0.189541 | -3.5% |
| Source render, count 1 | 0.312708 | 0.309708 | -1.0% |
| Typed retained render, count 1 | 0.250750 | 0.239459 | -4.5% |
| Mutable render/publication, count 1 | 0.688667 | 0.536459 | -22.1% |
| Mutable render/publication, count 16 | 0.698656 | 0.527359 | -24.5% |

The first four stages have unchanged code paths and exact allocation counters;
their timing movement is control variation, not an attributed improvement.
Mutable batches retain exactly the same history: 254 to 353 nodes at count one,
and 254 to 1,838 nodes at count sixteen. These bounded growing histories are
not an indefinitely fixed-size steady-state measurement. Native mutable time
still includes extra publication/validation work compared with JS retained
rendering; do not subtract medians or claim general Rust/JS parity.

All 15 paired mutable trials improve at each count; observed control/candidate
ranges do not overlap. [Independent attribution](native-attribution.json)
reconstructs source hashes by swapping only this file and records the raw
checks, allocation deltas and statistical limits. Serial samples and shared
load still limit generalization.

Raw evidence: [count one](native-comparison-1.json),
[count sixteen](native-comparison-16.json).

## Allocation evidence

A separate instrumented consumer records requested allocation counts and bytes,
not elapsed timings or process RSS. At count one, mutable publication changes
from 3,562 to 3,315 allocations and from 483,048 to 426,536 requested bytes
(11.7% fewer bytes). Peak additional requested bytes fall from 159,376 to
127,992. Required retained graph storage stays 14,528 bytes.

At count sixteen, allocations change from 57,112 to 53,160 and requested bytes
from 8,156,160 to 7,251,968 (11.1% fewer bytes). Retained storage remains
103,424 bytes; the later batch's peak remains 259,158 bytes. Typed parse,
public parse, source and typed retained allocation counters are exactly equal.
See [count-one counters](native-allocations-1.json) and
[count-sixteen counters](native-allocations-16.json).

## Profile that selected the change

The guarded pre-change [profile](native-profile.json) contains 1,406 samples
inside `State::once` for count sixteen. Property indexing has 180 inclusive
samples (12.8%); `own` has 341, `set` 175, and Context construction 40.
Categories overlap and optimized inlining can hide work, so they cannot be
added or converted into a speedup forecast. No temporary UTF-16 construction
samples remain under `own`/`set` after borrowed lookup.

The [timer-free graph census](parsed-property-lengths.json) finds 17-field task
records, three-field raw records and a 30-field root. Large task arrays remain
hashed. The 32-property cutoff is a bounded design choice, not a tuned optimum.
Raw sample trees are retained for [count one](render-mutable-publish-1.sample.txt.gz)
and [count sixteen](render-mutable-publish-16.sample.txt.gz).

## Validation

The public host-record integration passed on the original runtime before the
change, then passed on the candidate. It exercises growth, live replacement,
canonical duplicate keys, readonly NaN bits, recovery and complete SVG output
across two operations. Existing Gantt corpus, host callback, one-million-string
admission and binary identity integrations also pass: 17 tests in total.

Current source (`129ad96`) also passes 36 Chromium/Firefox/WebKit checks,
including host task growth/shrinkage, Date aliases, raw entry delegation,
calendar boundaries and frozen complete arena hashes. A source-built ARM64 SDK
passes 14 selected Swift integrations and [168 exact checks](swift-verified.json)
through all eight real APIs across the 21 workloads. Intel universal rebuild
and broader final release acceptance remain outstanding.

## Independent VM results and tradeoff

The Linux VM uses AMD EPYC-Genoa, eight vCPUs, 16 GiB and Chromium
156.0.8078.4. The [matched comparison](web-comparison.json) has three Gantt sizes,
fixed/Canvas metrics, 15 balanced trials and a 1,000 ms work warmup. Complete
SVG and exact dimension equality pass before and after. Runtime and harness
hashes are retained. Only WASM executable code changes; ordinary JS wrappers
are identical. AppleDouble transfer metadata is not executable package code.

All six retained medians are 2.1–8.0% lower, but paired descriptive intervals
support only four; two remain inconclusive. This does not establish a universal
browser gain. Source medians are 0.5–2.7% higher in this run; three conditions
show systematic upward paired shifts. Parse medians range from 2.4% lower to
6.2% higher, with small/fixed worsening and the other apparent costs unresolved.
Unchanged source/parse paths do not justify dismissing these results as noise.

One justified [source-only follow-up](web-source-comparison.json) reduces mixed
workload pressure while retaining the same immutable packages/harness. Medium
source remains 1.57% slower with fixed metrics and 3.08% slower with Canvas;
both paired descriptive intervals stay above zero. Small/large shifts do not
reproduce clearly. This harness still shares two WASM instances and performs
retained correctness setup; it does not identify the cause. Parse was not
remeasured. Both runs remain published in full; the later run does not replace
the first. [Independent attribution](native-attribution.json) includes paired
statistics and serial/shared-host limitations.

The optimization is kept for its strong native improvement and smaller retained
browser gains, with the medium-source/parse costs explicitly unresolved. It is
not a regression-free result. No gain percentages are combined with earlier
checkpoints.

The fresh [release matrix](web.json) uses all 21 workloads, both resource
profiles, all three stages, nine alternating trials, a 100 ms minimum work
warmup and the verified 5 microsecond clock. Package hashes match locally and
on the VM. [The gate](gate.json) still rejects **126/126** conditions. Median
ratios across 42 conditions per stage are 2.43x source, 11.30x parse and 4.04x
retained mutable render. Source ratios range from 1.61x to 4.86x. These are
current-package observations, not causal changes from older matrix runs.
0.3 remains unreleased; native gains do not establish web performance parity.

## Current Swift API timings

The [focused report](swift-timing.json) covers Gantt medium through all eight
APIs, nine trials and a 500 ms work warmup, with fresh retained state per batch.
Current Rust/FFI archive, SDK and release consumer bytes are verified. Timings
include API ownership/transport costs and exclude process startup, control
pipes and output verification. The first attempt [aborted on an unrelated
Swift build](swift-timing-rejected.json); a clean retry passed the guard. No
partial measurements from the rejected attempt are used.

| Swift API | Median ms |
|---|---:|
| Immutable parse | 5.497813 |
| Source render | 0.448892 |
| Typed retained render | 0.394488 |
| Mutable parse | 0.203924 |
| Snapshot | 0.092593 |
| Apply | 0.247040 |
| Mutable render | 0.849820 |
| Full mutable lifecycle | 1.312414 |

This is current local Swift behavior, not a controlled historical Swift speedup
or a comparison with the Linux VM. Other desktop activity remains recorded.
