# Shared constructor decoder: rejected V1 and bulk-storage V2

The first extraction (V1) was rejected after all six ER parse conditions increased: median paired shift **+5.45%**, range **+1.51% to +6.31%**. Both candidates passed the same 57 focused browser checks across Chromium, Firefox and WebKit. Correctness checks did not justify accepting the V1 timing regression.

V2 keeps one bounded event/atom decoder and moves whole fresh-record/dense-array loops into storage primitives. The ER primitives preserve the original direct assignment, NaN descriptor, ownership and accounting paths. They remove the V1 per-record temporary wrapper and per-field sink call. Flat storage remains a separate unselected consumer of that decoder.

V2's six ER parse shifts range from **−0.84% to +1.86%**, with four lower and two higher; their conventional median is **−0.33%**. Medium/fixed remains **+1.21%** (7/21 lower pairs), and large/canvas **+1.86%** (7/21). These results do not prove neutrality or a speedup. All rows, including adverse controls, are retained below and in the raw reports.

| Run | Stage | Rows lower | Median paired shift | Row range |
|---|---|---:|---:|---:|
| Rejected V1 | Parse | 0/6 | +5.45% | +1.51% to +6.31% |
| Rejected V1 | Retained render | 3/6 | −0.08% | −0.49% to +3.97% |
| Rejected V1 | Source render | 2/6 | +0.58% | −1.52% to +1.50% |
| V2 | Parse | 4/6 | −0.33% | −0.84% to +1.86% |
| V2 | Retained render | 2/6 | +0.45% | −0.56% to +1.34% |
| V2 | Source render | 4/6 | −0.14% | −0.45% to +0.48% |

Each separate VM run has 18 conditions: ER small/medium/large, fixed/canvas resources, and source/parse/retained stages. Each condition has 21 cyclic/reversed balanced three-way trials, matched control/candidate batch counts, three-sample calibration and 1,000 ms warmup. Per-row values are medians of paired candidate/control percentages. Stage aggregates are conventional medians (average the central two of six rows). These are batch means, not individual-call tail latency. Source/retained control shifts are observations, not proven random variation. V1 and V2 are separate runs against the same frozen control; this is not a directly paired V1-to-V2 causal estimate.

Both measured packages use identical `pele_bg.wasm`, SHA256 `3f6c40133130905f170ba582b3b9ef0c8718e795f2305e402200d5877563fc5b`, from Rust source `365ec6d0df050199f9efb93d23d9ce7d352c60fa`. The control browser code is the accepted V2 traversal at `e48ea33f65294f752fa5f3bee85c67611fb5db36`. Only `constructor-storage.js`, `flat-constructor.js`, `host-er-store.js`, and `package.json` differ per package freeze. Newer native graphless core, emitter and adapter work is **not compiled into the measured WASM**. No new Sankey backend is selected by these experiments. This is an enabling-construction experiment, not a release gate or API-parity result.

Evidence: [V1 raw 18 rows](v1/comparison.json), [V2 raw 18 rows](v2/comparison.json), [all paired arithmetic](audit.json), [source/provenance and syntax checks](provenance.json), and per-run package manifests/source patches/browser+VM logs. Both complete 109-file frozen packages were locally rehashed against the manifests and reports. The 57-check browser runs overlap; they are not 114 distinct feature tests. Source patches preserve the measured code without copying generated package or binary bytes.

Run `node verify.mjs` from this directory (or any cwd with its path). It verifies portable evidence hashes, exact package-map bindings, unchanged WASM/control identity, harness/case/oracle hashes, browser log counts, all 36 row statistics and aggregate arithmetic. It does not run benchmarks or recreate browser proof.
