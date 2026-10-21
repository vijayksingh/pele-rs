# Rejected initial binary-publication candidate

**Not adopted. Production wrappers and WASM remain unchanged.** This candidate
passed 54 browser feature checks but did not improve complete first-use costs.
The [raw cold/retained comparison](web-comparison.json),
[first-use comparison](first-use-comparison.json), [identical-build check](first-use-aa.json)
and [independent attribution](attribution.json) remain published, including the
unfavorable rows. No 0.3 release or JS performance parity follows.

## Proposed change and scope

Current public parse builds a native identity arena, serializes it, decodes a JS
wire graph and publishes the real caller objects. The candidate's initial
publication consumes wasm-bindgen's already-owned snapshot bytes, validates
references during one structural traversal, and builds caller objects directly
from indexed offsets. UTF-16 scratch grows geometrically instead of allocating
16 KiB for every small frame. A packed comparator handles a first unchanged
render; methods, edits and callbacks materialize the legacy graph when needed.
Every final native outcome remains on the existing decoder/restoration path.
No Rust code, native ABI, diagram algorithm or wire format changes.

The [patch](candidate.patch) applies to wrappers at `cf358ea` and is retained only
for reproduction. Package reports pin the three changed JS files and all
unchanged files, including the identical WASM artifact. Native construction,
mandatory caller-object creation and full host restoration remain necessary;
removing an intermediate structure is not itself performance evidence. Prior
packed experiments also failed to establish a general cold-parse win.

## Measurement

Independent vm-agent: AMD EPYC-Genoa, eight vCPUs, 16 GiB, Chromium
156.0.8078.4. Both comparators use 15 balanced cyclic/reversed three-way trials
against pinned Pele JS, matched control/candidate batch counts, immutable
package/harness hashes, and verified high-resolution isolated clocks.

Candidate comparisons warm each runtime for 1,000 ms. The first-use A/A check
uses 500 ms; it verifies identical package files and exact behavior, but is not
a numerical correction for the later run. Shared heaps, serial warmup,
background load and JIT/GC effects remain limitations.

Cold parse includes dispose. Complete first-use operations include parse, first
API use and dispose inside the timer. The edited-render lane adds a caller-owned
cyclic extension and checks its identity; it does not change diagram geometry.
Gantt's first method is the declared `getTasks(now)`, with a complete value,
alias, descriptor and number-bit signature captured before verification
rendering can reschedule anything. The callback lane wraps `getTasks`, delegates
to the captured native method and changes the first task's label. Complete SVG,
exact dimensions and relevant identities are checked before/after trials.

| Workload | Cold parse | Parse + unchanged render | Parse + edited render |
|---|---:|---:|---:|
| Sequence small | +7.69% | +5.71% | +3.42% |
| Flowchart medium | +3.91% | +1.82% | +4.89% |
| ER medium | +0.31% | -0.58% | +2.79% |
| Gantt medium | +1.63% | +3.99% | +1.80% |

Positive means slower than the default; these are ratios of reported medians,
not universal causal estimates. Gantt parse + first invoke is 5.01% slower,
with 13/15 paired trials slower and a descriptive paired interval above zero.
Edited first-render intervals are also positive for Sequence, Flowchart and ER.
Gantt callback's -1.70% median ratio has a near-zero paired median and a broad
interval: no supported improvement. Steady retained medians remain mixed near
zero, as expected for an initial-publication-only change.

Source controls are mixed: Flowchart is 2.81%/4.23% slower in the two reports,
while others move both ways. A/A medians range from -5.73% to +1.30%, with
estimator sensitivity and even one positive source paired interval despite
identical files. Therefore no blanket attribution of every slowdown is made.
The candidate is rejected because it provides no demonstrated cold/lifecycle
benefit and exposes first-use costs. Fewer allocations/copies did not justify
adoption. Typed-array string conversion and repeated DataView reads are
unmeasured mechanism hypotheses, not established causes.

## Correctness and reproduction

54 candidate browser integration checks passed across Chromium, Firefox and
WebKit: public mutable models, captured/native host calls, rejected-apply retry,
partial errors, disposal, aliases, numeric payloads, Error brands, UTF-16,
readonly/duplicate properties and combined malformed/budget/truncation cases.
No native/Swift rebuild was needed because native bytes are unchanged. Existing
production Swift proof remains at the preceding checkpoint; it is not new proof
for a changed native artifact.

The two additional transport/publication integration cases are archived in
`integration/`; ordinary production tests never load this rejected code. To
reproduce in an isolated copy of `cf358ea`, copy this evidence folder and the
first-use harness into that copy, apply the patch there, build its web package,
and run:

```sh
npx playwright test -c docs/benchmarks/2026-10-20-initial-publication/integration/playwright.config.js
```

Then replay the public models, host-functions, render-operation and disposal
specs against that package. The archived cases were run with equivalent route
interception of exactly these three candidate files; syntax checks pass on the
reproduction files. No claim is made that rejected code ships in the default
package or that the reproduction command was rerun after archiving.
