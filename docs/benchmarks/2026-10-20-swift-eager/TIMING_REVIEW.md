# Independent Swift timing review

The three completed reports support a large immutable `Engine.parse` improvement for Sequence small and Gantt medium in this local Swift consumer. Both parse cases are faster in all 15 pairs. Source and retained render medians are near flat. This is evidence for the measured combined binaries, not isolated attribution to the eager bridge, a memory result, or JS parity.

Reviewed saved reports and source on 2026-10-20. No build, consumer execution, new timing, or production edits were performed. Independent arithmetic and profile-tree reconstruction are saved in `audit-timing.py` and `timing-audit.json`.

## Provenance

All three reports are `completed: true` and identify the same executed public `scripts/perf/swift/compare.mjs` SHA:

`a3b3d82f8a36e2d411851efec25537f60ec83d7f5f862ef7db49c43ef5b1ed3f`

That SHA matched the current source during this audit. Preserve that exact harness before removing its redundant statistics assignments; a subsequently cleaned source file will correctly have a different hash.

| Item | Control | Candidate |
| --- | --- | --- |
| Build revision | `02ee639d398deaabbaa923b7948c9329a1bcf4ce` | `364c509c4d2b2dcbeb7e63438a658c5dd6403d09` |
| Binary SHA256 | `a79a05ffc6302608f26a412ca6903cb44d2d10b7ddbe1d74465ac2dea3242f1e` | `1b94f3e492374f6f0c6028c59a1020da2ed47c91f5fcdeeb5572197e428b9df6` |
| Manifest SHA256 | `a751a4c0dc387af67f40112a0898d6ce6e1b49e886e7c118c72a0dcfd775003e` | `198268b4f4e7e7e60690212a1248e2aeab0bda998f7d4eb9e63485e704c4a7df` |

Both binaries and manifest files on disk match the recorded hashes, and embedded manifests equal the files. The candidate's complete current source/SDK input aggregate, computed by the imported build module without invoking its build entry point, is `77391725bcdf1d564af92b2496764aead70ccc020dcd7c92f380101356afec91`; it matches all three reports with zero input differences. Swift compiler, architecture, SDK, consumer input hashes, and case-generator hash match between the frozen builds. The currently installed oracle hash matches the pinned `d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d`.

All 14 changed-input records recompute from the two manifests. They include direct Gantt construction, core producer/consumer changes, the eager Swift bridge, FFI/header changes, and rebuilt CPele. The historical control tree was not reconstructed; its provenance rests on the frozen executable and original manifest. These are combined changes, not an eager-bridge-only A/B experiment.

Report SHA256s:

- Parse: `9981160d54aa4d6f6169e459a87d269fb0b2b5ad636710d875f72919b07ecaf5`.
- Source render: `0c6a2790e8195f0ae9375282d771d41771cb3865a8d54143b099c6294f9673aa`.
- Retained render: `055247ad4f0f166fb1d6dbba189511d68b8ce2910650daec2511bfd472a9d506`.

## Recomputed observations

Milliseconds per operation; median change is the ratio of independent medians, while paired change is the median of candidate/control ratios. Fifteen matched pairs per row.

| Public API | Case | Batch | Control ms | Candidate ms | Median change | Paired change | Faster pairs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| parse | Sequence small | 64 | 0.308373 | 0.058730 | −80.95% | −81.05% | 15/15 |
| parse | Gantt medium | 4 | 5.308823 | 0.845084 | −84.08% | −84.02% | 15/15 |
| render source | Sequence small | 128 | 0.082231 | 0.082172 | −0.07% | −0.07% | 8/15 |
| render source | Gantt medium | 32 | 0.435927 | 0.431550 | −1.00% | −0.90% | 12/15 |
| render retained | Sequence small | 128 | 0.068515 | 0.068635 | +0.18% | −0.31% | 8/15 |
| render retained | Gantt medium | 32 | 0.368180 | 0.368146 | −0.01% | +0.40% | 7/15 |

Every stored per-operation array, ratio, paired percentage, median, MAD, minimum, maximum, sample p95, and faster-pair count agrees with independent calculations from integer elapsed nanoseconds. Parse paired ranges are −81.76…−79.64% for Sequence and −84.47…−83.16% for Gantt. Source Sequence has a +94.77% individual paired outlier: near-flat medians must not be described as a narrow host-noise bound. No confidence interval or significance claim is made.

The JSON `statistics.controlMs` and `candidateMs` fields are arrays: later shorthand keys overwrite earlier summary objects in the executed object literal. Raw samples remain intact, and the reported ratios and paired summary are correct. This is a schema/readability defect, not a timing-arithmetic defect. Audit output supplies the independently reconstructed summaries. Preserve the original reports and executed harness rather than rewriting historical evidence.

## Protocol and correctness checks

- The three separate windows each contain exactly two rows and 30 paired trials, with 15 control-first and 15 candidate-first trials overall. Each row necessarily has an 8/7 split; the next row reverses it. The source's global imbalance rule also correctly permits a difference of one for an odd number of rows.
- Every trial has equal control/candidate operation counts, alternating order, both guard records, positive safe-integer elapsed time, and a batch at most 128. Saved calibration counts double from one, stop on the slower side reaching 12 ms or the common cap, and agree with the final batch.
- Warmup counts match exactly between binaries and reconstruct from the saved pair count and cap. Each side accumulates at least 1,000 ms, rather than sharing a one-second budget. This intentionally makes slower control warmup longer: parse Sequence takes about 5.05 s control / 1.00 s candidate; Gantt 6.72 s / 1.06 s.
- Source bytes and source SHA256 match each saved case. All five prepare/verify output records before and after each case match the recorded oracle SVG hash/byte count and exact IEEE dimension bits. Every calibration/trial/before/after pair matches all observable fields, including checksum and any arena fields, excluding only elapsed time. This audits saved correctness evidence; the oracle was not re-executed here.
- `guardQuiet()` rejects any recorded build process or Pele-owned xctest before every timed request, including warmup and calibration. Saved calibration/trial guard records contain no rejected process. Four rejected-attempt logs show the guard stopping on Swift compiler/build or cargo processes; those partial runs are excluded. Warmup retains aggregate durations rather than each guard snapshot, so its per-request guard use is established by the hashed source, not a complete independent process log.
- The guard is a pre-request snapshot, not continuous exclusion. Other host load remains visible, including WindowServer, Claude, and Chrome. Shorter completed windows preserve the guard without establishing a dedicated idle machine. Process names in the rejection logs alone do not prove which external project owned the compiler.

Source inspection confirms public immutable parse, eager construction, document destruction, and per-operation autorelease-pool drain lie inside the parse timer. Process startup, NDJSON, output verification/hashing, and retained document setup/warm render are outside it. Parse's timed checksum consumes the returned type; its eager public model work is supplied by the implementation, not forced by a lazy getter. These three stages do not time explicit model access or `decodeModel`, and render controls do not measure parse-plus-first-render latency.

## Profile attribution

The post-profile manifest exactly matches the candidate build manifest. Both saved sample-file hashes and verification outputs match. Independent traversal of the sample trees reproduces every attribution count and percentage. The attribution field named `sourceSHA256` hashes the sample text, not the Mermaid input.

| Inclusive frame category | Gantt medium (4,369 exercise samples) | Sequence small (1,146 exercise samples) |
| --- | ---: | ---: |
| Foundation JSONDecoder | 38 / 0.87% | 103 / 8.99% |
| ConfigValue decoder | 0 / 0.00% | 0 / 0.00% |
| `pele_parse_eager_v1` | 4,130 / 94.53% | 856 / 74.69% |
| Swift eager callback | 1,723 / 39.44% | 324 / 28.27% |
| model projection | 1,642 / 37.58% | 275 / 24.00% |
| exact envelope byte counting | 184 / 4.21% | 34 / 2.97% |

Counts use outermost matching frames inside `Prepared.exercise`, suppressing repeated recursive/witness matches. Categories overlap. In particular, `nativeParse` includes synchronous Swift callbacks and model projection; it is not pure Rust time and must not be added to those categories. The samples support removal of the former dominant ConfigValue decoder path and identify projection/callback construction as remaining work. Different sample totals and visible external Python activity make them diagnostic attribution, not a second latency experiment or an absolute-time decomposition.

Only two workloads and three APIs have completed timing evidence here. Full public API, Unicode, model-key, error, budget, provider, and mutation parity still rests on separate integration evidence. No allocation/live-memory, browser/VM, JS speed, 0.3 release, or all-family performance claim follows from these reports.
