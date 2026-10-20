# Owned public construction: Sankey pilot

Runtime `b19f38d` transfers the captured parsed model into its paired public codec. Sankey ordinary parsing retains that owned model after canonical graph construction and equivalent admission, avoiding immediate graph-to-typed-model reconstruction. Prepared input and arbitrary providers retain the legacy hook. The public graph, edits, helpers, snapshots and first-render behavior stay eager. This is one codec opt-in, not a rollout to every family.

The VM comparison contains 24 rows with 21 three-way trials and 1,000 ms warmup per runtime/condition. All six Sankey parse rows are paired lower by 8.65-11.19% (15-20/21 lower pairs), but still 8.91-15.08x pinned JS. Sankey source controls have a +0.63% median row change (four of six adverse; range -0.98 to +1.92%); retained controls are mixed (-0.69 to +2.11%). Sequence parse controls range -0.11 to +0.58%. Every row, sample and output-check record is retained in `comparison.json.gz` and summarized in `RESULTS.md`.

The separate first-use comparison contains 24 rows with 15 trials and 1,000 ms warmup. Each operation includes parse, first use and disposal. All six Sankey first-edited-render rows are paired lower by 0.44-8.54%; five of six first-unchanged-render rows are lower. **Medium/fixed first-unchanged render is adverse +2.242%, with 7/15 lower pairs.** Source and Sequence controls remain mixed and are retained in full. These windows are not pooled. They do not establish universal speedups or release parity.

Native allocation-only measurements use separate allocator-instrumented binaries at counts 1 and 16. At count 1, Sankey public parse changes 1,002 -> 882 allocations, 110,400 -> 93,876 requested bytes and 39,439 -> 35,171 peak additional requested bytes; reallocations stay 39 and live delta stays zero. At count 16 the allocation/requested-byte reduction scales with operations. Sequence public-parse allocation/reallocation/requested counts are unchanged, while peak additional requested bytes change 68,980 -> 68,588. Both families' immutable parse, source and retained allocation rows are unchanged. Requested/live/peak counters describe the scoped allocator operations, not RSS or whole-process memory. They are not latency measurements.

Local timing is **partial: only two complete native conditions were accepted**. Sankey/medium public parse has a paired median change of -16.2082% (15/15 lower pairs), with separate medians 0.066658875 -> 0.05575 ms. Candidate public parse is **1.4293x local pinned Node JS** (JS median 0.0390051875 ms). The typed-parse negative control is adverse +1.0464% paired (5/15 lower pairs; candidate median 0.019208375 ms). Typed parsing omits full JS public-object projection; its faster absolute time is not public API parity.

Each accepted condition has its own complete 15-trial, 1,000 ms warmup, fixed-count-16, three-way window. **Six other native conditions and all six Swift latency conditions remain unmeasured.** The first combined native attempt was rejected before sampling; a later combined window and five individual attempts were rejected by the unchanged strict guard. The eight-minute bounded scheduler then expired. Complete rejection logs and scheduler history are retained; partial trials were discarded, never pooled. Other host load remained present (including WindowServer and engine-lab), so accepted guard checks do not mean an idle machine. No cross-host subtraction or ratio arithmetic links these local measurements to the browser VM results.

## Correctness and provenance

- Native: 32 public integrations, including the full 11,616-source Sankey graph/error corpus and historical SVG corpus.
- Browser: 48 focused checks plus 3 full Sankey corpus checks, across Chromium/Firefox/WebKit. The prior full 405-test suite belongs to source-result-packet `640deee`; it is not a new full-suite result for this layer.
- Fresh host SDK and standalone Swift consumer: 16 integrations, including ER/Sankey mutable corpus (12,481 drawings, 16,572 expected errors, zero explicit budget differences), seven eager, seven prepared and the existing nonfinite Sankey case; all 168 consumer API conditions and all 3 strict C consumers pass.
- Exact immutable Git binding verifies all 657 web-source hashes, 338 Git-backed Swift inputs, the new integration file, and all 277 native benchmark inputs against `b19f38d`. Swift has 351 total inputs including generated SDK material. Before/after correctness hashes match, and the SDK host archive equals the target Rust archive.
- The native manifest records HEAD `640deee` because the binary was built with the owned-source WIP before commit. Its exact `3459ae99...` source aggregate equals immutable `b19f38d`; this is not a claim that clean `640deee` produced it. The Swift manifest similarly retains its actual build-time revision and exact input hashes.
- All 106 files per frozen local package match VM-reported input hashes. This bundle does not claim an additional independent remote physical rehash. Neither source binding nor matching artifact hashes proves a bit-reproducible rebuild.

Six later candidate-package CDP profiles (Sankey/medium, JS/WASM, source/parse/retained) are retained compressed with their report. They are not paired before/after profiles or proof that a particular sampled percentage was removed. VM comparisons share one browser/page, GC and cache pressure, and serial warmup; they are diagnostics, not the full performance release gate. Version 0.3 remains unreleased, and the historical full performance gate still fails all 126 conditions.

`design/LLD.md`, `committed-manifest.json` and `source-changes.patch.gz` describe the committed source. Files named `candidate-*`, the original `manifest.json` and `candidate.patch.gz` are archived pre-implementation proposal artifacts; their unexecuted status text describes that earlier stage. Current correctness is recorded separately above.

Run the portable data-only verifier from any directory:

```sh
python3 path/to/owned-construction/summarize.py
```

It verifies retained samples, paired arithmetic, identity maps, correctness records and frozen-harness hashes, then regenerates the complete summary, paired table and deterministic evidence index. It does not execute repository code, rebuild, access the network or run benchmarks. No binaries or full package trees are included.
