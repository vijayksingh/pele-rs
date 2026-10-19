# SVG allocation and disposal checkpoint

All 126 measured WASM conditions remain slower than pinned Pele JS. No 0.3 release.
[VM samples](web.json) and [failed gate](gate.json) identify the exact runtime artifacts,
current harness, oracle and isolated clock. Stage median ratios use the mean of the
two middle rows: source 2.51x, parse 11.02x, retained mutable render 4.25x.
Differences from historical reports are not causal speedup percentages.

## Changes and controlled evidence

Disposed JS bridges now drop their transport graphs/arrays and per-document invoke
closures after native disposal succeeds (e8316b2). Caller-owned models and aliases
remain intact. [Reference census](disposal-retained.json) observes 200 public
parse/dispose operations per workload within the same synchronous WeakRef job:
each old package retains 200 attached graphs; the candidate retains zero.
Unique invoke functions decrease from 200 to one. These are retained-reference
counts, not bytes, RSS, a cross-job leak, or elapsed-time gains.

[Disposal comparison](disposal-comparison.json) uses matched batch counts and 15
balanced trials. Parse differences range from roughly 0.2% slower to 10.9% faster.
[Identical-byte control](aa-comparison.json) still varies roughly 4–6% for parse.
[Disposal full report](disposal-web.json) and [gate](disposal-gate.json) independently
remain 126/126 slower than JS. No universal CPU improvement is claimed.

SVG escaping now borrows unchanged UTF-8 rather than copying it, and entity text
reuses its decoded String when no escaping is required. Changed strings preserve
the original character loop. This removes a copy/allocation on unchanged nonempty
strings; escaping still scans and may allocate. This design was independently
identified in the Go port; its advertised 2x speedup was not established here.

[Matched SVG comparison](svg-comparison.json) and [reversed package order](svg-comparison-reverse.json)
cover seven medium families, source and retained stages, 15 balanced trials and
1000ms warmup. Most differences are only a few percent and overlap observed
measurement noise; these reports do not establish a broad speedup. Full output
checks and every package-file hash are preserved. The private comparison harness
loads two WASM heaps and is diagnostic; the single-package full report is the
release gate. See [methodology](../../../scripts/perf/COMPARISONS.md).

## Correctness and provenance

The SVG source differs from base 75e2525 only as identified by
[source fingerprints](source-fingerprints.json). Runtime hashes identify the
actual generated WASM and JS; no generated package is committed. Native source,
numeric and coordinate-boundary integration checks passed. The shared SVG helper
also passed 123 browser integration checks across Chromium, Firefox and WebKit,
including the pinned diagram corpora and all 33 numeric families.

[Swift consumer](swift-verified.json) verifies all eight actual API paths across
21 workloads (168 checks), complete SVG hashes/bytes, exact dimensions and frozen
source/SDK/archive correspondence. Eight local Swift integration tests passed,
including all 33 numeric families, output limits and 31 coordinate boundaries.
Only ARM64 SDK built; universal Intel verification remains required before release.

The compact unchanged-graph transport remains a private, uncompiled prototype.
It has no measured speedup and is not included in this package. No Go timing
claim, GPU speedup, or complete release readiness is implied.
