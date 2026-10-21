# Gantt construction scratch storage

The native change is `02ee639`, confined to fresh Gantt public-arena
construction. Fixed raw/task records pass owning array iterators instead of
temporary growable vectors; the root reserves its known field count. Runtime
properties, parsing algorithms, adapters and wire formats are unchanged. See
the [ordering and ownership design](../../LLD_GANTT_CONSTRUCTION_FIELDS.md).

The control is the adaptive-property implementation (`129ad96` runtime), with
Gantt source SHA-256 `b7cdd2671c322aab3410632554fb2d5733375c8ae180a64ea69e736f6361fe4d`.
Candidate Gantt source SHA-256 is
`0628796030be17123e9b78ab445cee9154e46c54c0348ef384660d051881ed4d`.
Independent native consumer manifests pin complete source-input hashes
`bc196da8df086f9f348d79335f90ad5181f073ad614747adbf7bc72be08f9d90`
and `5f3eb388e501c61712ce49be0995f514fe8430b93f118f2423587a387d143cf9`.
Manifest Git revisions precede source commits; the hashes establish exact inputs.

## Deterministic allocation result

The separate allocation binary measures one 48-task public parse, including
dropping the parsed document. These are requested allocator bytes, not RSS.

| Counter | Control | Candidate | Change |
|---|---:|---:|---:|
| Allocations | 1,971 | 1,875 | -96 |
| Reallocations | 195 | 48 | -147 |
| Cumulative requested bytes | 339,525 | 241,733 | -28.80% |
| Peak additional requested bytes | 109,626 | 109,370 | -0.234% |
| Live byte delta after operation | 0 | 0 | unchanged |

Zero live delta means the timed stage drops its result, not that retained
documents require no memory. Canonical graph storage is unchanged. Typed parse,
source rendering, typed retained rendering and mutable publication have exactly
equal counter/checksum objects. These operations do not use the changed fresh
construction helper within their measured region.

[Raw allocation counters](native-allocations-1.json) and the independent
[allocation review](allocation-review.json) retain both binary hashes and the
limits of this measurement. Busy-host records do not invalidate deterministic
counters; elapsed timings require separate coordinated runs.

## Initial native timing

The [count-one comparison](native-comparison-1.json) uses fifteen balanced
three-way trials, a 1,000 ms work warmup and matched native batch counts on the
same M1 Pro. Runtime-local timers exclude process communication and verification.
The guard rejects active Rust/Swift builds and Pele tests; unrelated interactive
load still exists. Whole output checks pass before and after.

| Stage | Control ms | Candidate ms | Median change |
|---|---:|---:|---:|
| Typed parse | 0.068666 | 0.062791 | -8.56% |
| Public parse | 0.194875 | 0.178917 | -8.19% |
| Source render | 0.302291 | 0.298667 | -1.20% |
| Typed retained render | 0.242834 | 0.245750 | +1.20% |
| Mutable render/publication | 0.514375 | 0.520417 | +1.17% |

Public parse improves in thirteen of fifteen paired trials, but unchanged typed
parsing also improves markedly. The full observed public-parse difference cannot
be attributed to scratch-array removal. Do not subtract those medians to
manufacture an adjusted speedup. Count-one samples include a substantial
shared-host outlier in typed parsing; all samples remain in the report.
[Independent review](native-timing-review.json) records ranges, paired changes,
descriptive intervals, checksums and negative controls. Serial samples and
exploratory comparisons limit inference beyond this run.

One justified [count-sixteen follow-up](native-comparison-16.json) reduces the
sensitivity to individual short operations. It uses the same frozen binaries,
source hashes, fifteen balanced trials and 1,000 ms warmup; matched native batch
counts are now sixteen for typed parse, public parse and source rendering. The
original report remains unchanged. A different batch count changes cache and
history behavior, so this is not an identical replication.

| Stage, count sixteen | Control ms | Candidate ms | Median change |
|---|---:|---:|---:|
| Typed parse | 0.056099 | 0.055474 | -1.11% |
| Public parse | 0.179513 | 0.165690 | -7.70% |
| Source render | 0.297083 | 0.290508 | -2.21% |

Public parse improves in eleven of fifteen paired trials with a paired median
of -7.78%. Its descriptive paired bootstrap interval spans -11.69% to +1.95%,
so the direction is observed but not statistically settled. The unchanged typed
parse shift is much smaller in this follow-up.
Source's ratio of medians is negative but its paired median is +0.11%, so do
not attribute a source speedup. These are bounded native observations, not a
claim about all parsers or complete browser API cost.
[Independent follow-up review](native-timing-16-review.json) retains the
matched-input checks, paired ranges and serial-sample limitations.

## Independent browser comparison

The [VM comparison](web-comparison.json) uses the prior immutable
`gantt-properties/package` and a fresh `fixed-fields/package` on AMD EPYC-Genoa,
eight vCPUs, 16 GiB, Chromium 156.0.8078.4. Three Gantt sizes plus unchanged
Flowchart medium, fixed/Canvas metrics and three stages produce 24 conditions.
Each has fifteen balanced trials and a 1,000 ms work warmup. Complete SVG and
exact dimensions match before and after; ordinary adapter JS is unchanged.

Gantt parse median changes range from -4.61% to +3.38%. Medium fixed/Canvas are
-4.61%/-3.69%; large Canvas is +3.38%. These mixed observations do not establish
a general web parse win. The diagnostic shares one browser page and heaps for
two independent WASM instances plus pinned JS, so it does not replace the full
release matrix.

The [independent paired review](web-review.json) supports medium/fixed parse
improvement in this run (twelve of fifteen paired wins), while the other five
Gantt parse conditions remain uncertain. Large/Canvas parse illustrates
distribution sensitivity: +3.38% ratio of medians but -4.06% paired median.
Unchanged source paths have supported paired shifts in both directions;
Flowchart/Canvas source is +3.15% by ratio of medians and Gantt large/Canvas is
+2.21%. Those regressions cannot be dismissed as proven noise. Binary layout,
JIT, allocator and shared-heap effects are possible explanations, not diagnosed
causes. The change is retained for deterministic allocation reduction and the
native public-parse observations, with these browser costs disclosed.

## Compatibility checks

Twenty existing public native integrations pass, followed by one new public
typed-model adaptation integration. That case covers prepopulated Dates,
existing fresh-Date NaN clipping, signed zero, equal-valued distinct objects,
missing fields, raw start absence, Unicode/duplicate classes, aliases, complete
binary roundtripping and ownership after the typed input changes. It exercises
the public module boundary rather than the private builder.

Thirty-six browser checks pass across Chromium, Firefox and WebKit with the new
WASM. Coverage includes ten frozen complete-arena construction fingerprints,
calendar boundaries, live host overrides, growth/shrinkage and error recovery.

A newly source-built ARM64 SDK passes fourteen selected Swift integrations.
The independently rebuilt consumer links the exact new SDK host archive and
passes [168 complete output conditions](swift-verified.json) across twenty-one
workloads and eight actual APIs. This includes SVG hash and dimension-bit
checks. Compilation, correctness and timing are separate proofs.

The [focused Swift timing](swift-timing.json) uses Gantt medium, nine trials and
a 500 ms work warmup after all local build/test jobs finish. Median API times
in milliseconds are immutable parse 5.779813, source render 0.446967, typed
retained render 0.378132, mutable parse 0.192714, snapshot 0.092801, apply
0.252357, mutable render 0.849982, and complete mutable lifetime 1.356305.
These include the real Swift/FFI ownership and representation costs; they are
not a matched historical Swift speedup or a cross-machine JS comparison.
Immutable Swift parse remains expensive. Intel universal packaging and final
release acceptance remain outstanding.

No 0.3 release is authorized by this checkpoint. The most recent complete
[release matrix](../2026-10-20-gantt-properties/web.json) still fails all 126
conditions; it predates this focused change. Lower allocation traffic does not
establish JS parity or reduce the browser's remaining graph reconstruction cost.
