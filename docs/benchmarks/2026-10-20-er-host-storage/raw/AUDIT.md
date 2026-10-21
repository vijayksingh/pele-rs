# Independent raw-adoption VM evidence audit

Read-only recomputation from completed reports. No builds, consumer/browser execution, VM access or production edits. `audit.py` reproduces the checks and writes `audit.json` with every row's recalculated statistics.

**Outcome:** the measured raw host layer is slower on every first-inclusive and retained row. This evidence does not establish a raw performance improvement or ordinary-path regression absence. Its callback correctness motivation is separate: these timed unchanged-model fixtures pass on both implementations; the old control's callback bugs are not exercised or hidden in these timings.

## Provenance and protocol

- Raw report succeeded, Chromium 156.0.8078.4, 2026-10-20 06:55:18–06:56:33 UTC. Eighteen rows, 15 triples each: 270 control/candidate pairs, 810 measured lane batches. Warmup is at least 1000 ms and six operations per lane/row.
- All 106 candidate package file hashes independently match the current local packed package. All 106 control hashes agree across raw report, immutable input manifest and preceding ordinary report. The control's actual VM bytes are not locally mirrored at this review, so that part is report-to-report provenance, not an independent reread of the frozen control. Six package files differ: `full.js`, `host-er-store.js`, and four `pkg/pele*` artifacts.
- All 124 ESM oracle file hashes independently match the local distribution and frozen oracle pins. The monolithic bundle matches the established pinned SHA. Both executed harness modules, oracle pins, cases and shared guard match local files. Raw input-manifest and control-proof SHA checks pass. README dependency instructions changed after execution; the executed modules/pins did not.
- Every raw batch duration divided by its recorded count reproduces the sample. Counts are shared across all three lanes, bounded to 1–128, and equal the minimum calibrated count. Calibration probes and stopping condition, medians, nearest-rank p95 batch means, ratio-of-medians, paired ratios and wins all recompute.
- Every recorded trial order matches the cyclic/reverse schedule. Each lane is first exactly five times per row. Control precedes candidate 8 times and candidate precedes control 7 times: aggregate 144/126 across 18 rows. This is balanced first position, not perfectly balanced pair precedence; 15 is odd and every row restarts the schedule.
- All six case/profile correctness groups agree across the three stages. The executed harness compares complete SVG, `Object.is` dimensions, links, caller identities and complete data-descriptor/alias graphs before/after. Fixture and both post-render state hashes match; first/repeat SVG hashes match. Reports retain hashes and successful assertion flags, not the full values: this audit checks the hash-bound assertion code and records, without independently rerendering the outputs.
- The initial missing-guard attempt stopped before freezing/timing and remains excluded. No interrupted attempt is pooled with the completed raw run.

## Raw results

Percentages below are **median paired changes**. Ratio-of-medians remains separately available in `audit.json`; do not interchange them.

| Case/profile | First-inclusive | Wins / 15 | Retained raw | Wins / 15 |
|---|---:|---:|---:|---:|
| small / fixed | +8.135% | 4 | +73.106% | 0 |
| small / Canvas | +6.399% | 3 | +77.143% | 1 |
| medium / fixed | +18.721% | 0 | +77.059% | 0 |
| medium / Canvas | +17.219% | 0 | +71.185% | 0 |
| large / fixed | +21.898% | 0 | +84.813% | 0 |
| large / Canvas | +27.154% | 0 | +87.582% | 0 |

First-inclusive ratio-of-medians changes span **+6.515% to +28.408%**, compared with paired **+6.399% to +27.154%**. Retained ratio-of-medians changes span **+72.358% to +88.721%**, compared with paired **+71.185% to +87.582%**.

Candidate versus pinned JS is **4.078×–6.090× for first-inclusive**, and **5.252×–6.996× for retained rendering**, using ratios of medians. A blanket “5–7×” would misstate the first-inclusive rows.

Clone-only paired medians range −1.006% to +0.517%; that does not justify subtracting clone medians from first-inclusive time. The first-inclusive timer contains clone, raw setup/admission, complete render and native disposal. The retained timer excludes its fresh clone/initial adoption/disposal. Renderer/schema/font caches are warm. These stages answer different questions.

## Ordinary 12-row controls

The separately completed ordinary report uses the same 106-file control/candidate packages and browser, but **nine triples and 500 ms warmup**, not the raw run's 15/1000. Its recorded medians, p95 values, ratios and matched WASM counts recompute. Its executed `compare-web-vm.mjs` hash matches current source; deterministic order can be reconstructed, but individual trial-order/duration records were not emitted. It does not contain the raw harness's per-row quiet-guard evidence.

Eleven of twelve paired medians increased. The sole decrease is medium Canvas source rendering (−2.031%, 8/9 candidate wins). Paired changes:

| Case/profile | Parse | Retained model | Source render |
|---|---:|---:|---:|
| medium / fixed | +2.374% | +0.734% | +1.502% |
| medium / Canvas | +1.679% | +2.841% | −2.031% |
| large / fixed | +3.626% | +0.787% | +0.489% |
| large / Canvas | +3.620% | +2.252% | +0.082% |

Separate medians are mixed (−3.014% to +4.487%), including signs opposite some paired medians. Neither representation supports calling the ordinary path uniformly faster or regression-free. This short diagnostic does not establish the cause or statistical certainty of the smaller differences.

## Limits of the evidence

The raw guard rejects active builds/Pele tests and a pre-existing browser; it is not continuous CPU exclusivity enforcement. Snapshots record node/chrome-headless and transient shell/ps processes, and high-CPU observations alone cannot identify every process's role. Timer isolation/resolution checks passed. One page shares JavaScript heap/GC and caches across all lanes; warmup is serial. Large raw rows use batch count 1; p95 is a nearest-rank batch-mean statistic (with 15 samples it selects the largest sample), not a separately sampled per-operation tail estimate. No confidence interval, memory/RSS conclusion, causal renderer-stage attribution, callback-performance claim, complete-release-gate result or JS parity claim follows from these rows.
