# Native Gantt construction checkpoint

Production source `7909cdf`; JS `e8316b2`.
[Architecture](../../LLD_GANTT_CONSTRUCTION.md) replaces general mutable property
operations during fresh construction with private Rust record/dense-array
builders. Runtime mutation, scheduling, callbacks and transport are unchanged.
Builds preceded the source commit; their exact input/binary hashes identify the
candidate even where the recorded Git HEAD is the preceding `6099c58`.

## Native measurements

[Completed pre-change parse sample](native-parse.sample.txt.gz) and
[provenance](native-profile.json) locate substantial construction work in general
record assignment, UTF-16 conversion and hashing. The later render profile in
that first attempt was interrupted by unrelated Swift compilation; it supplies
no evidence here. Sample counts are inclusive and overlap their ancestors.

[Allocation-only comparison](native-allocations.json) checks frozen independent
control/candidate binaries and full output against pinned JS, without operation
timers. Of 28 medium-workload conditions, only Gantt public parse changes:

| Gantt public parse | Control | Candidate |
| --- | ---: | ---: |
| Allocations | 3,471 | 3,071 |
| Reallocations | 1,539 | 1,344 |
| Requested bytes | 499,645 | 374,617 |
| Peak additional requested bytes | 169,236 | 111,778 |

Requested bytes fall 25.0%; these allocator counters are not RSS or CPU speedups.
The other 27 conditions have identical counters, including typed Gantt parse,
source/retained rendering and the six other diagram families.

[Same-Mac native comparison](native-comparison.json) uses 15 balanced trials,
1000ms measured warmup per runtime and matched control/candidate counts:

| Gantt medium | JS | Native control | Native candidate |
| --- | ---: | ---: | ---: |
| Typed parse | 0.06659 ms | 0.05826 ms | 0.06160 ms |
| Public parse | 0.06064 ms | 0.35420 ms | 0.23178 ms |
| Source render | 0.27307 ms | 0.28537 ms | 0.28539 ms |
| Typed retained render | 0.21234 ms | 0.22834 ms | 0.22757 ms |

Public parse median is 34.6% lower than its matched native control and still
3.82x the paired JS parse time. Typed parse code is unchanged but its negative
control varies +5.7%; source/retained medians vary +0.0%/-0.3%. Do not attribute
those unchanged-path differences to this optimization or combine ratios with
the older baseline. Native public parse constructs/owns an extra identity arena;
JS GC is forced outside batches. Compiler/Pele-test guards remained active.

## Independent VM

[Controlled browser comparison](web-comparison.json) uses frozen pre-change and
candidate packages, all three Gantt sizes, fixed/Canvas resources, all three
stages, 15 balanced trials and 1000ms warmup. Gantt parse medians are 15.6–25.0%
lower across the six size/resource conditions. Whole source, fresh-parse and
retained output match pinned JS before/after. Unchanged source/retained paths
vary around the control; no general rendering speedup is claimed. This
two-WASM-instance diagnostic differs from the single-package release harness.

The [complete VM report](web.json) passes provenance/output checks but its
[performance gate](gate.json) still fails **126/126** conditions. Median ratios
across 42 rows per stage (mean of middle two) are 2.42x source, 11.64x parse and
3.92x retained; source ratios range 1.64–5.54x. These are this artifact's values,
not causal gains against previous reports. **No 0.3 release.**

## Swift and public integration

Fresh source-built ARM64 SDK passed 14 local Swift integration tests covering
mutable models, Date aliases, calendars, callback identity, partial errors and
recovery. [Actual Swift consumer](swift-verified.json) passes 168 API conditions
across 21 workloads/eight APIs with exact SVG/dimension/source/SDK hashes.
[Current Swift timing](swift-timing.json) covers all eight APIs for seven medium
workloads, nine trials and 500ms measured warmup under compiler/test guards.
Gantt source/retained/mutable render medians are 0.455/0.396/1.096ms. Immutable
Swift parse is 5.59ms versus mutable parse 0.266ms; that API cost remains visible
in the full report. These timings are not a controlled historical comparison or
JS ratio. Universal Intel packaging still needs verification before release.

Fifteen native public integration tests pass, including the full historical
Gantt model/SVG/error corpus, methods/descriptors, binary stress and host functions.
Thirty-three Chromium/Firefox/WebKit checks pass for calendars, callbacks and
complete identity tables. Nine frozen whole-arena fingerprints preserve every
node/string ID, ordering, unused entry and numeric bit pattern. No unit tests.

## Next measured seam

[Fresh mutable profiles](native-mutable-profile.json) retain exact node counts:
one render grows 254 to 353; a 16-render batch grows 254 to 1,838. They are
history-dependent publication workloads, not fixed-size steady-state latency.
Raw samples: [one render](native-mutable-1.sample.txt.gz),
[16 renders](native-mutable-16.sample.txt.gz). The profiler also sees reset,
warmup, destruction and input waits; attribute only the measured call subtree.
Durable indexing remains unimplemented; these profiles guide the next private
representation change without removing observable Date history or copying.
