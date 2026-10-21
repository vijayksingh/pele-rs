# Independent direct-ER results review

Audit completed against frozen runtime `175daa8fd76fc1e25f488e4957b6970b4b6d4fe2`, with normalized-ER `4e9faee` controls. This review covers the original six accepted Swift windows, five native timing windows, one separate allocation report and 30-row VM comparison. Later confirmations/A/A, first-use and full release gate are reviewed separately below and are never pooled into the original windows. No builds, benchmark execution or browser/VM activity by this reviewer.

All assertions in `audit-results.py` passed; detailed recomputed statistics, hashes and exclusions are in `results-audit.json`. The frozen source capture precedes any later runtime layer. This establishes evidence consistency, not regression freedom or JavaScript parity.

## Provenance

- Swift: all 348 current source/SDK/consumer input hashes match the current build manifest. Aggregate `7c37902a471c01561ec3d927887a5e3b15e78deff6ee37d2ae7a1584955d48d0`; actual candidate binary `5d26bbd4b895a53671d6c49b2b1fe46a290ff6c77624e357b0d8a5af7541bc54`. The actual XCFramework archive matches the source-built target archive and manifest host hash. Frozen control binary and manifest hashes match each accepted report. Changed inputs are the SDK archive and four ER runtime files (new direct_parse, mod, parser and public_model); consumer and cases inputs remain identical.
- Native: current harness-defined source aggregate `a81c89b02eccb359b288c4af2de85b8d465c9f770c4979033a958c91c69376a9` matches the manifest. Actual candidate/control timing and allocation binaries all match reported hashes. Candidate timing binary `9e3772a3a8a59df1fda0c7932f53331d03160105ffbc13255678f656ddb9e174`; control `fa70a5a879a3b87dba39cd1b8f45eeb2c511b51ca1588ba2ee3ef506ffce78f2`. This hash domain is exactly the native harness's listed source/manifests, not a claim to reconstruct every historical toolchain/dependency input.
- Web: all 105 actual files in `dist/package-web/package` match the candidate report. All 105 control hashes match the preceding normalized-ER comparison's candidate. Only generated `pkg/pele.js` and `pkg/pele_bg.wasm` differ. The control package comparison here is recorded-chain verification, not an independently reconstructed historical build.
- Current executed comparator, cases and pinned oracle hashes match the reports. Public staged JSON/report copies and available compressed log copies match private originals. Public summary numbers reproduce raw data. `swift-verified.json` is current-manifest correctness evidence: 21 cases × eight APIs, 168 exact output checks, no elapsed timing fields.

## Swift accepted timings

Each row has 15 matched pairs, at least 1000 ms measured warmup per binary, identical warmup operation counts and bounded batches (maximum 128). Alternating order offsets are zero for Sequence and one for ER: aggregate 45 control-first and 45 candidate-first pairs. Calibration/count rules, before/after exact output, every raw output/checksum, nanosecond arithmetic and summary statistics were checked. The comparator's controlMs/candidateMs statistic fields are arrays, as already documented; derived ratios are correct.

| Case | API | Control ms | Candidate ms | Median paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| sequence/small | parseMutable | 0.068425 | 0.069875 | +1.251% | 6/15 |
| er/medium | parseMutable | 0.352467 | 0.289611 | -17.912% | 15/15 |
| sequence/small | lifetime | 0.229072 | 0.229654 | +0.772% | 6/15 |
| er/medium | lifetime | 2.469172 | 2.382401 | -3.228% | 11/15 |
| sequence/small | parse | 0.062275 | 0.061318 | -1.363% | 11/15 |
| er/medium | parse | 0.621520 | 0.618549 | -1.366% | 11/15 |

Five compiler-guard failures are wholly excluded, with no pooling or timing-based selection. The first Sequence parseMutable attempt retains eight partial pairs; the other four failed reports retain zero. All five log/report failures are the declared build/Pele-test guard. The controller retries only that failure and stops on other errors. The accepted rows contain no prohibited build/Pele-test process in recorded guards. The host is not idle: Chrome/WindowServer/other application activity remains visible, so “guard accepted” means the declared guard passed, not elimination of all system interference.

## Original native diagnostic

Five separate accepted windows, 15 three-runtime trials each, 1000 ms warmup and fixed batch count 16. Recomputed every sample from integer elapsedNs/count/1e6, medians, ratio-of-medians, paired changes and output-counter equality. The source records cyclic/reversed order giving five first positions per condition. Raw reports do not serialize per-trial order, so that part is verified from the hash-bound executed harness, not independent order telemetry. The succeeded flag follows exact SVG hash/byte/dimension checks before/after in that harness; per-condition full SVG verification payloads are not retained in these native reports.

| Case | Stage | Control ms | Candidate ms | Median paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| er/medium | parse | 0.176880 | 0.182250 | -0.270% | 9/15 |
| er/medium | parse-public | 0.331042 | 0.268477 | -18.503% | 15/15 |
| er/medium | render-source | 0.905971 | 0.922904 | +2.400% | 3/15 |
| er/medium | render-retained | 0.727385 | 0.743490 | +2.229% | 6/15 |
| gantt/medium | parse-public | 0.154305 | 0.160115 | +4.985% | 1/15 |

ER public parse improves in all 15 original pairs. The adverse Gantt public-parse result and ER render-source/retained results must remain disclosed. ER typed parse illustrates why paired change and separate medians differ: paired −0.270%, separate-median ratio +3.036%. ER retained has a large +616.6% paired outlier; this does not erase the +2.229% paired median. Separate fresh-process confirmations and A/A can investigate reproducibility but cannot retroactively remove or pool these observations. No “regression-free” conclusion.

Timers are inside Rust/Node. Protocol, prepare/reset, pre/post graph snapshots and output verification occur outside native timed intervals; parse/public-parse creation and drop occur within the measured operation. Retained native rendering uses the prepared typed model; it is not a full application lifetime or mutable publication measurement. Checksums containing native u64 bit combinations can exceed JS's exact integer range; exact result correctness relies on separately asserted SVG hash/bytes/dimension bits, not checksum equality alone.

## Allocations

Ten one-operation rows were collected through separate allocation-instrumented binaries, with timing disabled. Prepare/reset happens before counters enable; serialization occurs after counters disable. These measure allocator requests, reallocations, frees, net live-byte change and peak additional requested bytes within that operation. They do not measure RSS, physical memory, total retained document size or elapsed speed. Compiler activity in this allocation-only recording does not make it a timing sample.

ER public parse changes from 3838 to 3407 allocations (431 fewer), 971 to 775 reallocations (196 fewer), and 380866 to 321618 requested bytes (59248 fewer). Peak additional requested bytes changes 133398→133197 (201 fewer); live delta is zero for both. Every other allocation row is identical. Gantt mutable-publish retains a +14528-byte scoped live delta on both sides; this is equal retained-operation growth, not evidence of a leak or zero whole-document retention. All requested-minus-freed identities match reported live deltas.

## Web diagnostic

30 rows, 21 trials per condition, **1000 ms** warmup, matched control/candidate counts. Every recorded median, sample p95 and ratio recomputes. The hash-bound harness compares exact whole SVG and dimensions before/after and freezes package hashes. Its one-page shared JS/control/candidate heaps and GC differ from the release gate. No latency distribution, full-gate or release-parity claim follows.

| ER case | Profile | Separate-median change | Median paired change | Faster pairs |
|---|---|---:|---:|---:|
| er/small | fixed | -0.048% | -2.387% | 13/21 |
| er/small | canvas | -0.420% | -0.856% | 12/21 |
| er/medium | fixed | -3.114% | -3.535% | 17/21 |
| er/medium | canvas | -4.661% | -2.133% | 14/21 |
| er/large | fixed | -6.063% | -5.960% | 19/21 |
| er/large | canvas | -6.323% | -7.354% | 16/21 |

All six ER parse rows have lower separate medians; the range is −0.048% to −6.323%. The often quoted −0.856% to −7.354% range is the **median paired** statistic. Pair spreads remain wide, including positive outliers, and control/render rows are mixed. Evidence supports a scoped diagnostic ER parse improvement, not universal improvement. The completed first-use and full-gate evidence is reviewed separately below. The separately recorded live resource-callback defect remains a correctness limitation and is not fixed by direct parsing.

## Separate native confirmations and A/A

All four fresh-process confirmation windows completed on their first attempt. Their current/control binary hashes, harness/cases/oracle, 1000 ms warmup, fixed batches of 16, 15 trials, raw nanosecond arithmetic, output counters and summaries independently verify. The identical-binary A/A report uses the same current 175daa8 manifest and binary hash for both sides. No data is pooled with the original five windows.

| Separate run | Median paired change | Faster pairs |
|---|---:|---:|
| gantt/medium parse-public | +2.932% | 6/15 |
| er/medium render-source | +0.004% | 7/15 |
| er/medium render-retained | +0.481% | 6/15 |
| er/medium parse-public | -18.343% | 15/15 |
| Identical-binary Gantt public parse A/A | +2.975% | 4/15 |

ER public parse improvement reproduces. ER source/retained confirmations are closer to flat than their original adverse windows. Gantt remains slower in the confirmation, while the identical-binary A/A itself shows roughly a 3% displacement. That establishes a material measurement/process variation floor for this comparison; it does **not** prove the original Gantt slowdown is harmless or attributable entirely to noise. Keep every result and avoid both a regression-free claim and a corrected/subtracted A/A estimate.

## Complete first-use diagnostic

Verified all 38 rows, 15 three-runtime trials each, 1000 ms warmup, matching 105-file control/candidate package maps, executed harness, cases/oracle and raw summary arithmetic. Before/after verification checks exact whole SVG/dimensions, edited extension cycle identity and first-method structural signatures. Each timed operation creates a fresh document and disposes it inside the timer. First-invoke times parse + method + dispose; the verification render is outside that timed method scenario. ER invokes addEntity on an existing entity and checks its alias identity; it does not establish first-use cost for every ER helper or a mutating new-entity case.

The selected stages exclude `parse-first-callback`; therefore this passing diagnostic supplies no resource-callback correctness/timing evidence and does not supersede the recorded baseline callback failure.

ER medium first-render paired changes are −1.428% fixed and −2.312% canvas (9/15 faster each); large first-render −2.070% fixed (10/15) and −2.998% canvas (13/15). Edited and first-method rows are mixed. In particular ER medium canvas first-invoke is **+41.167% paired, 6/15 faster**, with a +38.352% separate-median ratio. Sequence fixed first-edited render is +8.898% (5/15); Gantt canvas first-invoke is +4.378% (3/15). These adverse complete-operation observations prevent a broad first-use improvement claim. Both shared heap/GC state and serial warmup remain limitations; the data alone does not identify the cause.

## Full VM release gate

Independently verified all 126 unique rows (21 cases × two measurers × three stages), nine trials per runtime and 500 ms warmup. Exact source hashes, the current eight gate-listed runtime hashes, oracle/cases/harness, sample medians/p95 and every ratio match. Reproduced the checker result from saved samples and its current source: **126/126 slower than JS, gate failed**.

Median WASM/JS ratios across the 42 rows in each stage are source render **2.39535×**, mutable parse/dispose **10.94343×**, and retained mutable render **4.02953×**. This is not parity, and the gate supplies no authorization for a 0.3 release. Its selected runtime hashes match the separately verified complete 105-file current package; the gate itself hashes eight files, not all 105.

`audit-results.py` and `results-audit.json` now include these separate sections. Public audit copies were refreshed after this final append; original measured reports remain unchanged. The audit requires the retained frozen binaries/manifests and matching current inputs; it never builds or executes consumers.
