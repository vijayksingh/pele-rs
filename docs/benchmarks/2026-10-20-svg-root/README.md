# Borrowed SVG root checkpoint

Native source 0aba8ef; JS e8316b2. [Architecture](../../LLD_SVG_ROOT_ASSEMBLY.md)
replaces a full intermediate SVG and per-rectangle Strings with borrowed spans
written directly into the final output. The scanner also corrects the old
Unicode word-boundary discrepancy to Pele's ASCII regex behavior.

## Objective native allocation evidence

[Allocation-only report](native-allocations.json) compares frozen control and
candidate binaries, both hash-checked against their build manifests. No operation
timer is read and no elapsed-time rows are produced. Complete source, typed
retained, mutable and JS retained outputs are independently checked. All 14
source/retained conditions request fewer allocation bytes. For retained renders:

| Medium workload | Allocations old / new | Requested bytes old / new | Byte reduction |
| --- | ---: | ---: | ---: |
| flowchart | 4,221 / 4,072 | 797,871 / 710,497 | 11.0% |
| sequence | 1,278 / 1,261 | 251,966 / 203,528 | 19.2% |
| er | 9,677 / 9,194 | 1,824,194 / 1,561,057 | 14.4% |
| pie | 1,298 / 1,295 | 260,466 / 226,150 | 13.2% |
| sankey | 1,930 / 1,927 | 296,265 / 246,511 | 16.8% |
| gantt | 2,501 / 2,352 | 297,181 / 237,462 | 20.1% |
| railroad | 2,780 / 2,640 | 558,563 / 465,894 | 16.6% |

These are separately instrumented, one-warmed-operation allocation counters.
They are not RSS, JS heap comparisons, or elapsed-time speedups. The report also
records logical peak requested-byte deltas. Native typed documents persist across
batches; public state resets and JS retained models are separately warmed.

Local elapsed-time comparison attempts aborted when unrelated Swift/Rust builds
appeared. The compiler guard stayed active; no successful candidate CPU timing
report is claimed. The [prior native baseline](../2026-10-20-native-core/)
remains explicitly from before this rewrite. It already separates native typed
rendering from costly mutable-model representation; do not subtract Mac numbers
from browser VM numbers.

## Independent VM

[Matched comparison](web-comparison.json) covers seven medium workloads, source
and retained stages, 15 balanced trials and 1000ms warmup with matched native
batch counts. Source medians were 1.5–5.7% lower than control; retained medians
were 0.5–3.8% lower. These small diagnostic differences overlap measured noise
and two-WASM-heap effects. No broad or combined speedup percentage is claimed.

The [complete single-package report](web.json) passed output/provenance checks;
its [performance gate](gate.json) still fails **126/126** conditions. Stage median
ratios (mean of middle two rows) are 2.41x source, 11.47x parse and 4.09x retained.
Source ratios range 1.62–4.96x JS. These are this artifact's observations, not
causal gains against a historical run. **No 0.3 release.**

## Public-interface validation

Eleven native integration tests passed, including all 33 numeric families,
source contracts, coordinate boundaries and 115 new whole-output SVG-root cases
through source, typed retained and mutable lanes. The old library failed the new
Unicode-boundary oracle case; the rewrite passes it. The normal verification
script includes the new integration target. No implementation unit tests added.

Twenty-four Chromium/Firefox/WebKit integration checks passed. They cover all
33 numeric families, complete root fixtures through source/mutable WASM, fonts,
output thresholds, callback failure recovery and coordinate boundaries.

Nine local Swift integration tests passed, including the same 115 fixtures
through all three rendering lanes. [Actual Swift consumer](swift-verified.json)
passed 168 API conditions with frozen source/SDK/ARM64 archive fingerprints,
complete SVG hashes/bytes and exact dimensions. Candidate Swift timing remains
deferred until a quiet host window; previously published timing rows identify
their older native revision. Universal Intel packaging remains unverified for
this candidate and required before release.
