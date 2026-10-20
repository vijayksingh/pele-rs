# Borrowed Gantt lookup checkpoint

Production source `712ed1f`; JS `e8316b2`. The
[private lookup design](../../LLD_GANTT_LOOKUP.md) reuses existing stack UTF-16
encoding and borrowed HashMap lookup. It removes temporary owned strings on
existing-key reads/interning, preserving canonical IDs, budgets and lookup order.
Builds preceded the commit; exact input/binary hashes identify the candidate.

## Native measurements

[Attribution](native-attribution.json) comes from the
[preceding native profiles](../2026-10-20-gantt-builder/native-mutable-profile.json).
Inside the measured 16-render subtree, UTF-16 construction beneath `own`/`set`
has 322 of 1,813 inclusive samples; Context construction has 38. Categories
overlap; these are observed stack samples, not predicted removable percentages.

Allocation-only reports use separate instrumented binaries and no operation
timer: [one operation](native-allocations-1.json),
[16 operations](native-allocations-16.json).

| Medium Gantt operation | Allocations control / candidate | Requested bytes control / candidate |
| --- | ---: | ---: |
| Public parse | 3,071 / 1,971 | 374,617 / 339,525 |
| Mutable publication, one render | 6,350 / 3,562 | 566,288 / 483,048 |
| Mutable publication, 16 renders total | 101,720 / 57,112 | 9,488,000 / 8,156,160 |

Mutable allocations fall 43.9%; requested bytes fall 14.7% for one operation
and 14.0% for sixteen. Mandatory retained live-byte deltas remain exactly
14,528/103,424 respectively. Typed parse/source/retained counters are identical.
These are logical allocator requests, not RSS, JS heap or elapsed-time claims.

Guarded same-Mac CPU comparisons use 15 balanced trials, 1000ms measured warmup
and equal JS/control/candidate batch counts: [count one](native-comparison-1.json),
[count sixteen](native-comparison-16.json). Medians below are per operation:

| Medium Gantt operation | Native control | Native candidate | Candidate / control |
| --- | ---: | ---: | ---: |
| Typed parse, count 1 | 0.06608 ms | 0.06238 ms | 0.944 |
| Public parse, count 1 | 0.27013 ms | 0.20692 ms | 0.766 |
| Source render, count 1 | 0.32663 ms | 0.32221 ms | 0.986 |
| Typed retained, count 1 | 0.23858 ms | 0.23913 ms | 1.002 |
| Mutable publication, count 1 | 0.79942 ms | 0.68892 ms | 0.862 |
| Mutable publication, count 16 | 0.81013 ms | 0.66407 ms | 0.820 |

Mutable medians are 13.8%/18.0% lower; public parse is 23.4% lower, so its
extra short-miss hash does not regress this workload. Unchanged typed paths
vary -5.6% to +0.2%. Tiny fixed-count batches, forced JS GC outside batches
and shared-host activity remain disclosed. JS reference timings are in the raw
reports; its render does not return the native graph/revision contract. These
are not universal native/JS comparisons or percentages to combine with prior
checkpoints.

Each native batch reparses/warms before its timer. Count one grows 254 to 353
nodes; count sixteen grows 254 to 1,838 in every recorded native trial. These
matched histories are not indefinite steady-state latency. One count-sixteen
attempt and a broad Swift timing attempt aborted on unrelated compiler activity;
the guard remained intact. Successful reports above are complete separate runs.

## Independent VM

[Controlled comparison](web-comparison.json) checks all three Gantt sizes,
fixed/Canvas resources and three stages with 15 balanced trials/1000ms warmup.
Observed parse medians are 3.7–9.9% lower and retained medians 3.9–6.9% lower.
The small rows overlap earlier A/A variability; no precise causal gain is claimed
for each one. Unchanged source rows vary -2.2% to +2.0%. All complete outputs
match pinned JS before/after; packages are fingerprinted and immutable.

The [complete single-package report](web.json) still fails its
[performance gate](gate.json) in **126/126** conditions. Stage median ratios
(mean of middle two of 42) are 2.44x source, 12.15x parse and 4.02x retained;
source ratios range 1.60–5.30x. Cross-run aggregate differences are not causal
speedup/regression percentages. **No 0.3 release.**

## Swift and public integration

Fresh ARM64 SDK passes 14 Swift integration tests and
[168 exact actual-API conditions](swift-verified.json). A
[focused current Swift timing](swift-timing.json) covers all eight Gantt APIs,
nine trials/500ms warmup under compiler/test guards. Source/typed retained/mutable
render medians are 0.436/0.370/0.935ms; immutable parse remains 5.430ms versus
0.195ms mutable parse. No historical Swift speedup is inferred. Universal Intel
packaging remains required before release.

Thirteen native integration tests pass: full Gantt method/model/SVG/error corpus,
host callbacks, complete arena fingerprints and full string-table admission.
The last test reuses existing text at one million strings, rejects a new sanitized
link with the same limit error, preserves the whole graph and recovers with
correct revisions/Date aliases. Ten frozen constructor cases include repeated
ASCII, accented and emoji labels around the 64-byte threshold. Thirty-three
Chromium/Firefox/WebKit checks pass. No unit tests or new runtime dependencies.
