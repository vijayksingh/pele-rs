# Arena render result packets

Runtime `9ae8142` removes an owned snapshot clone and complete result-JSON transport from arena-backed retained rendering. The additive native outcome view borrows the post-render graph through immediate encoding; existing owned APIs remain available. WASM publishes that graph and a typed rendered-result packet under the same combined canonical byte limit. C/Swift binary consumers use the borrowed view. The public graph still publishes eagerly after each operation; this does not skip validation, reconciliation or callback work.

The full browser suite passed **417/417 in 18.4 minutes**. Local Swift latency remains **unmeasured**: the sole bounded attempt was rejected before any condition started because external Swift compiler processes were active. No native-speedup, release-gate or release claim is made.

## VM comparison and controls

The main diagnostic has 126 conditions, nine three-way trials and 500 ms warmup per runtime/condition. All **36 retained-render rows for the six arena families** are paired lower by **1.20-24.16%**, with a conventional median row change of **-15.30%**. They still take **1.65-18.49x pinned JS time**, depending on workload and resource policy. Family median paired changes are Flowchart -9.37%, Sequence -14.03%, Pie -20.70%, Sankey -19.19%, Gantt -2.53% and Railroad -16.38%.

Negative controls are retained in full. Source rendering is unchanged by this path, yet 26/42 rows are adverse: median +0.71%, range -3.89 to +4.18%. Parse has 22/42 lower rows, median -0.14%, range -11.57 to +12.89%. Sequence/small fixed parse is +12.889% with 0/9 lower pairs. The separate ER host backend is also a control: five of six retained rows are adverse, median +0.715%, range -0.402 to +3.279%. These results are not described as regression-free or dismissed as noise.

A separate 24-row follow-up uses 21 trials and 1,000 ms warmup, with the same package bytes. Sequence/small parse remains adverse: fixed **+2.699% (10/21 lower)** and Canvas **+2.002% (6/21)**. Flowchart/small parse is +1.144%/+2.899%; ER/small is +2.879% fixed and a 0% paired median on Canvas. Sequence/medium parse is lower in both resources. The original +12.889% magnitude does not repeat, but this does not erase the adverse direction in the small Sequence conditions. All six affected retained-render rows are lower by 9.91-16.35%. The follow-up is retained separately and never pooled with the original window.

## Complete first use

The separate 50-row first-use diagnostic uses 15 trials and 1,000 ms warmup. Every measured operation includes parsing, first API use and disposal. All 12 first-unchanged-render rows for the six arena families are lower by 2.19-12.78%. Eleven of their 12 first-edited-render rows are lower; **Flowchart/medium Canvas is adverse +0.754% (7/15 lower pairs)**. Gantt first-callback is lower in both resources, while its Canvas first-invoke is adverse +1.272% (6/15).

ER host-backend controls have material adverse rows: first-unchanged render **+2.363% fixed / +5.963% Canvas**, first-invoke **+0.497% / +7.112%**, first-edited fixed **+2.628%**, and first-callback Canvas **+0.957%**. Source controls are mixed, including Sequence/small fixed +2.013%, Pie/small Canvas +2.544% and Railroad/medium Canvas +2.251%. All 50 rows and sample-level results are included, not only the lower rows.

Method/callback timing scenarios exist only for Gantt and ER in this harness. Other families cover source, first-unchanged render and first-edited render. These are not all-family method or callback performance claims. Main, follow-up and first-use windows remain separate; cross-window changes are not compounded.

## Correctness and identity

- Native: 41 public integration tests across nine feature files, including the new borrowed/owned outcome view, provider/export-only fallback, all-family entries/numbers, Gantt, Sequence, Sankey and host-function behavior. The initial build failed at two test-only `Object::new` calls; changing them to `Object::default` repaired the test. Initial/repaired source maps and failure/pass logs are retained. Runtime hashes did not change in that repair.
- Focused browsers: 12 checks across Chromium, Firefox and WebKit, retaining 12 explicit JSON reports for 66 normal/raw family cases, callback/error cases, original factory throws, exact graph/revision output and combined-budget rejection/recovery. The subsequent full suite passed 417/417 in 18.4 minutes; its complete log and 78 explicit JSON reports are retained, with byte-identical reports deduplicated against focused artifacts.
- Fresh native SDK, one fresh standalone Swift consumer, 16 selected Swift integrations, three strict C consumers and all 168 API conditions pass. The ER/Sankey corpus covers 12,481 drawings and 16,572 expected errors with zero explicit budget differences. Swift selected tests include seven eager, seven prepared/all-33 raw-entry checks and the existing nonfinite Sankey render/Codable case. This is correctness evidence. A separately authorized one-shot timing attempt checked the completed browser log, then stopped at the unchanged global guard after 0.308 seconds. It observed swift-build 87522 and Swift frontends 89983/89988/90071. No consumers or conditions launched, no retry occurred, and Sankey/medium render/lifetime/parseMutable/parse all remain unmeasured. The schedule retains before/after process snapshots and exact scheduler/comparator/guard hashes. Source/binary preflight in the timing script was not reached; freshness proof here is the independent completed correctness freeze, not a successful timing preflight.
- All 351 source/SDK inputs are unchanged before/after correctness. The six expected core/FFI source files plus the rebuilt SDK archive differ from the owned-construction control. Candidate binary SHA256 starts `2e1491d5`; the preserved owned control starts `368ab53b`. The SDK host archive equals the target Rust archive (`9c1c6028...`).
- Immutable Git blob verification binds all 660 frozen source inputs and 338 Git-backed Swift inputs to `9ae8142`. The build manifests retain their actual earlier HEAD plus candidate WIP; they are not relabeled as clean builds of that earlier revision. Generated SDK bytes are bound through build and correctness manifests, not Git blobs. No bit-reproducible rebuild is claimed.
- All 106 files per local frozen package match the main, follow-up and first-use VM hash maps. The same-package 18 later JS/WASM profiles are compressed with their report. No additional independent remote physical rehash is claimed, and these candidate-only profiles are not before/after causal attribution or latency samples.

The VM uses one browser/page with separate runtime instances, shared GC/cache pressure and serial warmup. Three-way positions are balanced; two-side first order differs by one in these odd trial counts. Raw reports do not explicitly retain the order list; the verifier derives it from the hashed harness. No process-wide idle-host claim follows. Historical full performance parity remains unmet; version 0.3 remains unreleased.

Run the portable data-only verifier:

```sh
python3 path/to/arena-render-packet/summarize.py
```

It verifies raw arithmetic, package/source maps, recorded correctness and test artifacts, then writes the complete paired table, summary and deterministic evidence index. It does not execute repository code, build, access the network or run workloads. No executable binaries or full package copies are included.
