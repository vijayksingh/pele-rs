# Independent entry-frame evidence audit

Completed source, artifact, arithmetic and profile audit for `53b035e`. No compilation, consumer, benchmark or browser execution by this reviewer. A new read-only SSH SHA walk and `build.mjs inputs()` read were authorized and performed. The original private audit script produced `audit.json`, which records all rows, profile hashes and identity checks. The bundled `summarize.py` recomputes saved timing/profile arithmetic and indexes this evidence; it does not rerun remote/source verification.

## Finding

All six ER retained-render rows improve: paired medians −11.020% to −14.367%, with 20 or 21 wins out of 21 pairs. Candidate retained rendering remains 3.820–5.511× pinned JS by separate medians. Negative controls are mixed: ER small/fixed parse +3.850% (5/21 wins), medium/fixed +1.600% (8/21), medium/Canvas +4.803% (3/21), large/fixed +4.418% (5/21). The two other ER parse rows improve slightly. These observations do not establish absence of regressions.

## Identity and frozen source

Both frozen VM package directories were independently read over SSH at `2026-10-20T07:56:09.813Z` using a fresh Node SHA256 walk, outside the benchmark harness. Every one of 106 files per side matches `comparison.json`. Control also exactly matches the preceding lexical candidate package. The retained local `../er-entry-read-frame/default-package` independently matches all 106 candidate files. `profile/summary.json` records the identical candidate package, including WASM `1036593e94ae9113a848aaf1a0cbb118e05991d6abdc3c6e977fec17bb4d66f2`.

The source freeze reads immutable `53b035e` Git objects, not the working tree being changed by the next dispatcher layer. It verifies 447 source/consumer/web blobs, including 95 directly packaged binding files byte-for-byte. Six package files change from control: `full.js`, `host-er-store.js`, WASM, and the three generated WASM bindings/declaration files. Build-log completion predates the source commit; these checks bind the retained generated output and recorded source identity but are not an independently reproduced build.

A fresh read-only `inputs()` check produces native aggregate `8f4fbc262e2ce2b1784823f4dd635cb6e725b0084a7a75a507c36690763b6bea` across 351 files, exactly equal to the df3597c Swift build manifest. Native source/consumer Git blobs at 53b035e agree with that check. No new native/Swift timing or rebuild is claimed.

## Timing arithmetic and controls

| Case | Profile | Stage | Paired change | Wins / 21 | Candidate / JS |
|---|---|---|---:|---:|---:|
| sequence/small | fixed | parse | -0.885% | 12 | 15.942× |
| sequence/small | fixed | render-model | +0.128% | 10 | 7.921× |
| sequence/small | fixed | render-source | -0.423% | 12 | 2.992× |
| sequence/small | canvas | parse | -2.217% | 11 | 16.089× |
| sequence/small | canvas | render-model | -0.370% | 11 | 5.884× |
| sequence/small | canvas | render-source | -1.319% | 14 | 2.596× |
| er/small | fixed | parse | +3.850% | 5 | 12.201× |
| er/small | fixed | render-model | -13.703% | 21 | 5.511× |
| er/small | fixed | render-source | -0.665% | 12 | 2.508× |
| er/small | canvas | parse | -0.741% | 13 | 11.748× |
| er/small | canvas | render-model | -14.367% | 21 | 4.528× |
| er/small | canvas | render-source | -0.719% | 16 | 2.264× |
| er/medium | fixed | parse | +1.600% | 8 | 9.717× |
| er/medium | fixed | render-model | -12.726% | 21 | 4.625× |
| er/medium | fixed | render-source | -0.834% | 14 | 2.216× |
| er/medium | canvas | parse | +4.803% | 3 | 8.788× |
| er/medium | canvas | render-model | -11.020% | 20 | 4.000× |
| er/medium | canvas | render-source | -0.948% | 15 | 1.997× |
| er/large | fixed | parse | +4.418% | 5 | 8.662× |
| er/large | fixed | render-model | -12.408% | 20 | 4.368× |
| er/large | fixed | render-source | -1.065% | 14 | 2.235× |
| er/large | canvas | parse | -0.848% | 11 | 5.776× |
| er/large | canvas | render-model | -12.395% | 20 | 3.820× |
| er/large | canvas | render-source | -0.592% | 14 | 1.865× |

All 24 rows contain 21 samples in each of three lanes: 504 triples / 1512 lane batches. Every saved median, p95 and ratio of medians recomputes. Paired changes are independently calculated from aligned per-lane samples. Control and candidate use equal power-of-two batch counts, capped at 128; JS count may differ. Every lane reports at least 1000 ms warmup. Exact workload hashes, bytes and pre/post profile checks match preceding lexical fixtures.

The hash-verified cyclic/reverse harness schedule gives each lane seven first positions per row; control precedes candidate 11 times versus 10. The report does not save observed raw order or timer start/end, so order is established from the executed harness rather than independent telemetry. Full SVG and Object.is dimensions are asserted for source, retained and fresh-parse output before/after; only output byte-count summaries persist here. This is selected-fixture output proof, not the full public parity gate.

There is no per-row quiet process guard. The report records load/memory before/after, requires an isolated high-resolution timer, enforces a deadline and verifies package hashes before/after. One page shares GC/cache pressure. Retained parse/setup and its warm render happen outside batch timing; each timed retained operation includes its normal entry audit and frame work. Parse timings include disposal. Separate medians and paired medians are different statistics; neither should be substituted for the other.

## Profile attribution

All six raw profiles reproduce their complete published top-35 self-attribution lists using each sample’s `timeDeltas`, not flat sample counts. The 250 µs requested sampling interval is not assumed to be the observed period. These are separate 1500 ms Canvas profile runs (200 ms warmup per lane), not paired latency trials. Profile operation counts are diagnostic throughput under profiler overhead and must not replace the matched comparison.

| Retained WASM group | Self | Inclusive |
|---|---:|---:|
| jsAudit | 10.227% | 20.551% |
| jsErrorBrandDispatcher | 6.431% | 6.431% |
| jsEntryFrameEncoding | 4.218% | 4.218% |
| nativeEntryFrame | 2.669% | 2.691% |
| jsOwnReadBridge | 1.641% | 1.641% |
| garbageCollector | 0.314% | 0.314% |

Inclusive groups count a sampled stack once within that group. Different groups overlap and must not be summed. JS frame encoding and error-brand predicates use exact URL/line identities in the retained frozen source. `errorKind` and its `.find` closure at source line 19 account for 6.431% self time together; the top-list anonymous URL bucket alone is incomplete attribution. Grouping the published top list only by function name also combines unrelated functions called `decode`/`atom`, so it is not a unique function identity.

The profile confirms the frame path executes and that live own-read bridges still execute after invalidation; it does not prove an import-count reduction or quantify removed cost. JS frame encoding is 4.218% self and native EntryFrame reads/decoding are 2.669% self in this sample; audit is 10.227% self / 20.551% inclusive. No allocation or process-peak conclusion follows. The source-render profile has no sampled frame/audit work, consistent with lane eligibility, but zero samples are not proof a branch is impossible. No control CPU profile is paired with this candidate to establish causal savings.

## Validation evidence limits

Source inspection and the final public log are consistent with the intended outermost-normal-render scope, live nested renders, optional-cap fallback, deferred UTF16 errors and callback invalidation. `logs/tests-final.log.gz` reports 18 passes, and `logs/tests-helpers.log.gz` reports three passes. Earlier attempts remain relevant: packet capability-fallback fixtures omitted the new capability; NaN assertions incorrectly assumed one engine-independent literal payload. The final test creates the input through a nonnumeric placeholder, verifies it is NaN, and requires preserved observed input bits; it does not require all engines to produce the same initial payload. These are recorded test corrections, not claimed runtime fixes. This audit does not substitute for the full release gate or promise JS parity.
