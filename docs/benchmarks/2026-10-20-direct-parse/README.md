# Direct public parsing checkpoint

[Architecture](../../LLD_DIRECT_PUBLIC_PARSE.md) prepares source once and offers
an optional module interface for constructing canonical public state directly.
Gantt is the first implementation. One grammar and task-declaration parser feed
either typed storage or the public arena. Other modules retain typed adaptation;
raw prepared input, imported models and retained mutation keep existing paths.

The candidate removes temporary typed task records and their owned string/index
copies. It retains the complete public graph eagerly. It does not defer parsing,
model population or reconciliation to the first render, method or callback.
Near construction limits, it drops partial direct state and retries the exact
typed path once to preserve historical admission and error precedence.

## Native

[Matched native timing](native-comparison.json) uses independent frozen
executables, 15 balanced three-way trials, 1000 ms measured warmup and identical
16-operation native batches. The compiler/Pele-test quiet guard passed. Whole
output is checked against the pinned JS oracle before and after measurements.
Build manifests identify exact source and executable hashes; the build HEAD can
precede the source commit. The control contains the preceding fixed-field code.

| Gantt medium | JS | Native control | Native direct |
| --- | ---: | ---: | ---: |
| Typed parse | 0.08185 ms | 0.05619 ms | 0.05508 ms |
| Public parse | 0.08247 ms | 0.16788 ms | 0.14943 ms |
| Source render | 0.30474 ms | 0.28571 ms | 0.28216 ms |
| Typed retained render | 0.21879 ms | 0.22706 ms | 0.22377 ms |
| Mutable publication | 0.22040 ms | 0.51363 ms | 0.51251 ms |

Public parse has an **11.0% lower median**, winning all 15 native pairs. It still
takes **1.81 times** the paired JS parse time. The other stage medians differ by
0.2–2.0%; those values do not establish general rendering improvements. Mutable
publication includes history-dependent graph growth and its extra return contract.

[Allocation-only measurements](native-allocations.json), collected separately
from timing with count one, verify the same public outputs:

| Native public parse | Control | Direct |
| --- | ---: | ---: |
| Allocations | 1,875 | 1,437 |
| Reallocations | 48 | 40 |
| Cumulative requested bytes | 241,733 | 197,885 |
| Peak additional requested bytes | 109,370 | 87,681 |

This removes 438 allocations (23.4%), 18.1% of cumulative requested bytes and
19.8% of peak additional requested bytes. These are allocator counters, not RSS.
Typed parse/source/retained/mutable allocation counts and cumulative requested
bytes are unchanged. Typed parse's peak differs because shared task construction
changes allocation order; not every counter is identical. Both public-parse
operations release their requested live allocation balance after disposal.

## Independent browser VM

[Controlled comparison](web-comparison.json): AMD EPYC-Genoa, eight vCPUs,
16 GiB, Chromium 156.0.8078.4, 15 balanced trials, 1000 ms measured warmup.
Two immutable WASM packages and pinned Pele JS share one isolated page. Exact
SVG and dimensions match before and after timing. Gantt covers all three sizes,
both fixed/Canvas resources and all three stages; Sequence-small is a control.

Gantt parse medians are 2.5–9.5% lower across all six size/resource conditions.
Medium fixed/Canvas reductions are 2.9%/4.3%. Candidate parse still takes
13.9–31.2 times JS time in this diagnostic. Source and retained results are
mixed; small Canvas source is 3.1% slower. This is a scoped parse improvement,
not a general browser speedup or a replacement for the full release matrix.

[Complete parse plus first-use comparison](first-use.json) includes disposal
inside every operation. Gantt-medium first unchanged render is 1.3% lower and
first method invocation 5.0% lower; first edited render/callback are 1.7% slower.
Negative controls also vary, including a 7.6% Sequence first-render regression.
These mixed results do not establish a general first-use gain. No work is moved
out of parse to produce the measured parse result.

The [fresh complete matrix](web.json), nine alternating trials with 500 ms
minimum measured warmup, verifies all source/fresh/retained output and provenance.
Its [performance gate](gate.json) still fails **126 of 126 conditions**. Median
ratios across 42 rows per stage are 2.39 times JS source render, 11.39 times JS
parse and 4.02 times JS retained mutable render. These are this artifact's
results, not historical speedup estimates or all-33 performance certification.

## Compatibility and release boundary

Native public integration checks cover the full historical Gantt model/SVG/error
corpus, normalized direct-versus-typed graph aliases, repeated apply/render,
extension registration, source/edge policies, host functions and limit fallback.
Oversized UTF-16 and storage-pressure inputs exercise the exact legacy fallback,
including a later syntax error after a valid over-limit prefix.

Fresh direct graphs can allocate different initial IDs. Historical typed-layout
hashes remain checked separately; direct-layout hashes are identical across
Chromium, Firefox and WebKit. IDs in a live or imported graph are never renumbered.
Independent normalized graph/lifecycle comparisons protect observable values,
aliases, descriptors, number bits and operations instead of replacing behavior
checks with new hashes.

**This checkpoint does not meet JS performance parity. No 0.3 release.** Swift's
eager projection handoff is a separate measured layer; its results must not be
combined with these ratios or inferred from the implementation language.
