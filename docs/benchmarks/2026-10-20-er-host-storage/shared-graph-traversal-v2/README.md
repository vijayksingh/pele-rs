# Shared browser traversal V2: architecture evidence, mixed timing

Browser source `e48ea33` factors graph capture, matching and restoration into one
storage-bound traversal factory. The default nested representation keeps direct
paths; this prepares a later flat owner without selecting that backend. These
measurements establish neither performance neutrality nor a speedup. The earlier
[rejected traversal experiment](../shared-graph-traversal-rejected/) remains a
separate adverse result.

Both measured packages contain identical Rust/WASM from `365ec6d`, SHA-256
`3f6c40133130905f170ba582b3b9ef0c8718e795f2305e402200d5877563fc5b`.
Only `graph.js`, new `graph-storage.js` and the package manifest differ. The
native graphless Sankey owner and subsequent native layers are **not** included.
The recorded workspace native HEAD at freeze is not the measured WASM revision.

## Warm matched diagnostic

[comparison.json](comparison.json) contains all 42 rows and 21 samples/runtime/row:
seven workloads, fixed/canvas resources, parse, retained render and source render.
Independent control/candidate WASM instances and pinned JS share one browser page.
The harness balances cyclic/reverse order, uses matched WASM batch counts,
1,000ms warmups and complete SVG/dimension comparisons before and after timing.
Shared heaps, JIT, serial warmup and GC/cache effects differ from the release gate.

Headline metric is the median of paired candidate/control percentages for each
row. The stage aggregate is the conventional median across its 14 row medians
(average the two central values). Negative means lower elapsed time.

| Stage | Stage median paired change | Row range | Rows below control |
|---|---:|---:|---:|
| Parse | −0.19% | −3.23% to +3.11% | 8/14 |
| Retained render | +0.22% | −3.08% to +2.99% | 5/14 |
| Source render | +0.14% | −2.28% to +2.70% | 6/14 |

All 42 row medians:19 below control,23 above. Sequence small/canvas parse is
+3.11% (10/21 lower pairs); small/fixed retained is+2.99% (6/21), and Sankey
small/canvas retained is+2.78% (7/21). Flowchart medium/fixed retained is+0.10%
(10/21), compared with the separate rejected experiment's+9.16% (4/21).
The latter difference is not a controlled V1-versus-V2 causal estimate.

Source rendering and ER host operations do not use the changed nested traversal
algorithms; they are negative controls, not proof that observed movement is
random. Sequence medium/canvas source is+2.32% with only 3/21 lower pairs. All
adverse observations remain in the raw report and audit.

## First-use diagnostic

[first-use.json](first-use.json) retains 40 rows and 15 samples/runtime/row. Each
operation includes parse, first API use and disposal. First unchanged/edited
render covers six workloads under both resources; ER additionally covers first
helper invocation and first measuring callback. Whole output, dimensions and
edited-object identity are checked before/after. Source rows remain controls.

| Stage | Median paired change | Row range | Rows below control |
|---|---:|---:|---:|
| First unchanged render | +0.05% | −1.84% to +2.96% | 5/12 |
| First edited render | −0.51% | −3.77% to +1.86% | 8/12 |
| ER first helper | +1.80% | +1.77% to +1.82% | 0/2 |
| ER first measuring callback | −2.03% | −3.13% to −0.93% | 2/2 |
| Source control | −0.47% | −1.88% to +3.79% | 7/12 |

All 40 row medians:22 below control,18 above. Sankey large/canvas first unchanged
render is+2.96% (5/15 lower pairs); small/canvas first edited render is+1.86%
(5/15). Flowchart fixed source control is+3.79% (2/15). No unchanged-path movement
is dismissed as established noise, and no first-use win is inferred from a
parse-only result. These scoped diagnostics do not replace the 126-condition
release gate or establish Pele performance parity.

## Source, package and correctness binding

[provenance.json](provenance.json) pins source commits and selected source hashes;
[package-freeze.json](package-freeze.json) and
[package-local-recheck.json](package-local-recheck.json) preserve complete package
hash maps before/after the runs. Candidate source hashes were checked against
committed `e48ea33`, control against its parent. The copied implementation manifest
is the historical pre-execution revision, not a claim that later checks were absent.

Compressed full-browser and focused-browser logs record 417 and 54 passing tests,
respectively, across Chromium, Firefox and WebKit. The runs overlap; they are not
471 distinct features. Package recheck binds both to the same frozen candidate.
VM comparison logs are included too. No new browser, build or benchmark ran to
prepare this bundle. No generated WASM or package files are duplicated here.

Run `node verify.mjs` from this directory or by absolute path. It recomputes all
row medians, ratios, p95 batch means, paired changes and stage summaries; verifies
report/package/source-pin consistency, harness hashes, raw log hashes and bundled
file hashes. [audit.json](audit.json) preserves every row's descriptive statistics.
This portable verifier validates the supplied evidence and arithmetic; it cannot
rerun the browser, authenticate omitted runtime bytes, or turn a diagnostic into
a release gate. The copied harness sources document the measured procedure;
executing those harnesses requires their original repository/tool dependencies.
