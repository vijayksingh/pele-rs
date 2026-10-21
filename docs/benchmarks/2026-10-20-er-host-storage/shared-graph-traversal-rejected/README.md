# Rejected shared graph traversal checkpoint

Traversal-v1 patch `4ce4d35f82f294bfb84fc8e1a801e7064caec9757d680ba9d3e2d67c6f53dae4`
is performance-rejected/provisional. This bundle records that checkpoint; it does
not claim an accepted traversal extraction. A revised extraction is separate and
is not included here.

## Three independent evidence lanes

The web comparison isolates the proposed JavaScript traversal extraction. Both
packages contain the identical fresh Rust/WASM built from `365ec6d`, including
captured-method source commit `243580c`. WASM SHA256 is
`3f6c40133130905f170ba582b3b9ef0c8718e795f2305e402200d5877563fc5b`.
Only `graph.js`, new `graph-storage.js`, and package inclusion differ: control
has106 files; candidate107. The source proof binds662 base files to immutable
Git365, then overlays the three explicit patch hashes to describe663 candidate
files. The candidate JS patch is not represented as a committed runtime.

The local Swift comparison is different: captured-method core365 versus preceding
core1906, using distinct SDK archives and consumers. It cannot measure the JS
traversal. Twelve architecture profiles are older again: pinned1906 WASM
`cee0b15b999c68b1317bdc70ffc90000c53d22f27c1387eae2413c243a198453`.
They explain an earlier architecture checkpoint, not the measured traversal-v1
package. No percentages are compounded across these lanes or windows.

## Web results: all82 rows retained

The main diagnostic has42 conditions,21 balanced three-way trials each and1000ms
warmup per implementation. Only13/42 paired medians were lower. Material adverse
rows include Flowchart/medium fixed retained render +9.1608% (4/21 lower pairs),
Sankey/medium fixed parse +4.5309% (6/21), ER/medium fixed parse +3.5512% (5/21),
and Sankey/small Canvas retained render +3.1579% (8/21).

The separate first-use diagnostic has40 conditions,15 trials each and1000ms
warmup;21/40 paired medians were lower. Adverse rows include Flowchart/medium
Canvas first unchanged render +3.3302% (4/15), Flowchart/medium fixed first edited
render +3.0135% (4/15), Sankey/small Canvas first unchanged render +2.4361% (5/15),
and ER/medium Canvas source render +2.2444% (3/15). These results are not dismissed
as noise. [RESULTS.md](RESULTS.md) preserves every row, control, paired change,
win count and candidate/JS ratio. Raw samples and output checks remain compressed
in the two original reports. No new full performance gate or parity pass follows.

Paired change means the conventional median of per-trial candidate/control
ratios minus one, not a ratio of independent medians. Matched WASM batch sizes,
serial warmups, fixed/Canvas lanes, pinned oracle, host-process snapshots and
three-way ordering are recorded. Individual first ordering differs by one trial;
three-way positions are balanced. Reports do not retain explicit VM order arrays,
so the verifier derives order from the hash-bound harness.

## Local Swift: separate core change

One bounded attempt accepted three complete Sankey/medium conditions:

| Stage | Paired change | Lower pairs |
|---|---:|---:|
| parseMutable | -0.9226% | 8/15 |
| parse | +0.0499% | 7/15 |
| render | +1.0223% | 5/15 |

Each condition has15 pairs and1000ms warmup per binary. Order offsets0/1/0 give
aggregate23 control-first and22 candidate-first pairs. The lifetime condition was
rejected when global compiler guard found external `rustc` PID45785. All nine
persisted partial pairs are excluded; lifetime remains unmeasured. No retry or
pooling occurred. The whole attempt ended after35.3 seconds of its180-second cap.
Passing guards did not establish an idle host; external app load remains recorded.

Candidate consumer: `920faee3e167e654c06ec7953240cc5471bbafefd1164beca1b59516920fcf90`.
Control consumer: `16900d32f60685e93d92a1e697650d3fb34e5e0f63be6ad7c5be9c8305fcfdbf`.
Fresh source/SDK aggregate: `d4019492b901d9fa0f8b62075ab48463fd10597056b0265b8139a641a8d1bb58`.
Candidate SDK archive: `134c97654cda1344b3588857539161843e584b4de96b47e9ff6eb0e2b1a4a463`.
The fresh freeze has354 inputs,341 Git-backed. Against1906, three core source
paths including new captured_write.rs plus the SDK archive changed. Native/Swift
builds and correctness remained source-frozen; JS traversal is not a Swift input.

## Correctness and incomplete coverage

The browser suite was deliberately stopped after147 passes. One Firefox test was
interrupted and269 tests did not run. This is NOT417 passed and does not establish
three-engine full-suite coverage. The exact log, last-run state, interruption
context, and explicit JSON result files are retained; duplicate attachments and
screenshots are omitted. Passed checks do not cancel the adverse performance data.

The captured-method native integration slice passed22 tests. Fresh local Swift
correctness passed16 selected tests:12,481 drawings,16,572 expected errors,zero
explicit budget differences. All168 consumer API conditions over21 cases and all
three strict C consumers passed. These verify the native core checkpoint, not JS
traversal behavior. Archived design manifest statements describe the earlier
pre-execution proposal; frozen package reports and logs describe subsequent runs.

The12 profiles in `profiles-1906/` have their own package and immutable source
proofs. Each uses250us sampling and1500ms profiling duration. Their samples,
recorded top-frame summaries, operation counts and checksums are retained. They
must not be attributed to the newer3f6c traversal benchmark package or converted
into a speedup estimate.

## Portable saved-data verification

```sh
python3 summarize.py
```

Run within this folder. The standard-library script executes no workload, build,
browser, network request or repository import. It verifies all raw paired
arithmetic, warmups, output checks, package/source hashes, accepted/rejected Swift
reports, correctness counts and separate profile identities; then regenerates
`summary.json`, `RESULTS.md` and `evidence-index.json`. Gzip timestamps are fixed.
Harness copies are provenance, not portable executable benchmark entry points.

Frozen package bytes were independently hashed from local copies against both
reports. This is not a new remote physical rehash or reproducible-build proof.
No accepted extraction, regression-free result, cross-host speed arithmetic,
release, or new0.3 claim is made.
