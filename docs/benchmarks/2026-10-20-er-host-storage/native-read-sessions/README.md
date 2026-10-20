# Native graph read sessions: experimental checkpoint

Runtime `4033e413faefb7f502ed874fbe39904418d6e361` batches native graph publication reads in two JavaScript files. It remains opt-in. Against the preceding cursor implementation, all six Sankey parse rows are lower by 4.05-11.08% paired and all six retained-render rows by 4.17-15.91%. Against the ordinary arena, all six parse rows are **35.65-46.68% higher** and all six retained-render rows **19.84-40.20% higher**. This is progress within an experimental backend, not an accepted default replacement or JavaScript parity.

Both independent VM comparisons contain 24 rows: Sankey small/medium/large and Flowchart medium, fixed/Canvas resources, parse/retained/source render. Each row retains all 21 samples per runtime, with 1,000 ms warmup. Candidate/control batch counts match. The frozen harness cycles and reverses three-way order across JavaScript, control and candidate. Reports do not separately store each trial's order. Paired medians below differ from ratios of separate medians; both are retained in `summary.json`. Do not compound separate windows or subtract cross-host ratios.

| Comparison | Sankey parse paired change | Sankey retained paired change | Sankey source control change |
| --- | ---: | ---: | ---: |
| Native cursor control | -11.08% to -4.05% | -15.91% to -4.17% | -1.51% to +0.95% |
| Ordinary arena control | +35.65% to +46.68% | +19.84% to +40.20% | -0.59% to +1.33% |

Sankey candidate parse remains 14.46-21.25 times pinned JavaScript in the cursor window and 12.82-20.83 times in the arena window. Retained render remains 2.64-4.42 and 2.70-4.64 times JavaScript, respectively. Whole SVG and exact dimensions are checked before/after by the archived harness; compact timing reports retain SVG byte lengths and explicit backend probes, rather than full output strings.

All Flowchart negative controls are disclosed here; no adverse row is dismissed as noise:

| Window | Resource | Stage | Paired change | Candidate lower pairs |
| --- | --- | --- | ---: | ---: |
| Cursor | Fixed | Parse | +1.9647% | 10/21 |
| Cursor | Canvas | Parse | +2.1004% | 7/21 |
| Cursor | Fixed | Retained render | 0.0000% | 10/21 |
| Cursor | Canvas | Retained render | +0.7821% | 8/21 |
| Cursor | Fixed | Source render | -0.2506% | 12/21 |
| Cursor | Canvas | Source render | -0.2577% | 12/21 |
| Arena | Fixed | Parse | +6.7125% | 2/21 |
| Arena | Canvas | Parse | +3.1535% | 4/21 |
| Arena | Fixed | Retained render | +2.9122% | 5/21 |
| Arena | Canvas | Retained render | +3.4811% | 6/21 |
| Arena | Fixed | Source render | -1.0597% | 12/21 |
| Arena | Canvas | Source render | +0.1879% | 10/21 |

All 48 rows, full paired ranges and every adverse Sankey source control are also present in the generated summary and raw reports.

## Profiles

Six raw weighted CPU profiles cover medium fixed Sankey parse, retained render and source render, once per arena control/native candidate. These are **arena comparisons**, not cursor profiles. Both packages use the identical WASM SHA `2310934dd43dc52c20723c3e0217377f9819fef8e146d03ef3a33845e06bdcce`. HTTP `.wasm` frames are classified as WASM even when V8 labels shared compiled code with the other package URL.

Candidate parse samples partition into 35.59% JavaScript, 52.55% WASM, 8.92% GC and 2.93% engine/profiler; retained render into 36.33%, 56.34%, 0.23% and 7.10%. The disjoint JavaScript storage-adapter self roles are 13.74% parse and 15.21% retained; shared observer/publisher self roles are 18.31% and 16.82%. Inclusive groups overlap and must not be added. The broad canonical-boundary union is 75.88% parse and 97.35% retained; it includes work beneath owner-wrapper ancestors and is not an estimate of removable overhead. Source render is a separate control.

`profile-summary.mjs` extracts the pure classification/weighted-summary function from the exact executed profiler. The data-only verifier recomputes every stored language category, ordered disjoint role, inclusive group, deduplicated union and ranked frame table from raw samples. Profiling does not establish call/allocation counts or latency gains; shared heaps, inlining and regex attribution limit interpretation.

## Correctness and provenance

- Frozen `4033e41` sources: 587 entries mapped to immutable Git blobs; all unchanged through the recorded after-check. Package: 113 files. Candidate and each control differ only in `graph-storage.js` and `native-storage.js`; WASM is identical. The cursor and arena control packages themselves differ in `native-storage.js`, and selection differs: cursor enables native storage, arena disables it. They are not one interchangeable control.
- The build/package freeze originally records `5db9f35` plus worktree changes; exact source hashes map to the later committed `4033e41`. This is not a claim of a clean rebuild after commit. The preserved local candidate and cursor packages were independently rehashed while preparing this evidence; no new physical remote rehash was performed.
- `84/84` focused browser checks passed at the runtime checkpoint: 28 per Chromium/Firefox/WebKit, 37.3 seconds. The full `468/468` suite belongs to earlier `3166b3d`, not this checkpoint.
- Test-only `34da2664a9846fe44469988c1a35545639f2018e` adds the factory-failure slice. Three engine tests passed in 9.5 seconds, each with six normal/raw x object/undefined/symbol comparisons. Arena is loaded before opt-in; candidate must report native storage. Tests compare original throw identity, exact restoration/descriptors/cycles/aliases, full SVG/dimension-bit recovery and callback traces proving no replay. Explicit JSON files are retained.
- First factory attempt failed all three engine tests because the fixture expected a deleted/recreated root property to retain insertion order. Both backends actually matched. Setup was corrected to place that property last among retained keys; exact ordered comparison stayed intact. Initial source, failed logs and all six-case reports remain, alongside corrected source and successful reports.
- Read-only before/after checks retain all 362 native/Swift-consumer/SDK inputs at aggregate `1f89890b534bf1fe2958ed9b03bbc91470cae260a615925e1b15df752cb157ee`. No SDK rebuild or local latency rerun occurred for this JavaScript/test layer. Prior SDK correctness is separately documented.

## Portable verification

```sh
node verify.mjs
```

Only packaged bytes are read. The verifier checks provenance hashes and complete file index; frozen source/package identity; all 48 rows' sample counts, warmups, medians/P95, ratios and paired statistics; profile attribution; focused test counts; all failed/successful factory records; and native-input continuity. It executes no application, browser, benchmark, Git, build or network request. Exact raw JSON/logs/profiles and source snapshots are gzip-compressed; binaries and build targets are omitted.

`node verify.mjs --write-summary` regenerates `summary.json` from captured evidence. Regenerate the file index if bytes change. Runtime code and test source are separate pins. Passing this evidence verifier is not a new runtime test, full release gate, adoption decision or 0.3 release claim.
