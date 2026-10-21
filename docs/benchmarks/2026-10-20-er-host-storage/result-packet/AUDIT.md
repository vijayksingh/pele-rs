# Independent frozen result-packet evidence audit

**Pass for the scoped claims. No production edits, builds, browsers, or new timings.** Audited `docs/benchmarks/2026-10-20-er-host-storage/result-packet` using its data-only `summarize.mjs` and an independent `audit_evidence.py`. Detailed recomputed results and hashes are in AUDIT.json.

## Identity and counts

All 41 evidence archives match both compressed and uncompressed hashes. Complete candidate package map contains 106 files and exactly matches source-freeze.json. Complete control map also contains 106 files and exactly matches the previous raw report's candidate map; no partial hash intersection was substituted. Changed package paths are full.js, host-er-store.js, and four generated pkg/pele files.

The freeze names runtime commit `7339f38ceb282f242bc851b2570c586bdea541ca`. All 444 Git-backed source/consumer/web input hashes match that commit's Git objects. The remaining 13 SDK files and the native/Swift source inputs are included in the complete 351-entry input subset, which exactly matches the earlier raw Swift build manifest. Native aggregate SHA matches too. This is unchanged-input proof, not a new native/Swift execution. The benchmark harness and cases file hashes also match their frozen commit contents. The current lexer/frame package was not used as the identity source.

All 24 unique expected conditions exist: four workloads × two resource profiles × three stages. Each lane has 21 samples, giving 1,512 timed batch-mean samples overall. Every recorded median recomputes exactly, every paired median change and faster-trial count agrees, control/candidate batch sizes match, and all recorded warmups reach 1,000 ms. The recorded harness checks whole SVG and exact Object.is dimensions on retained/source/fresh-parse results before and after timing, and hashes complete packages again after timing. `succeeded` and `completedAt` are present.

## Performance claim

| Retained ER | Fixed paired | Canvas paired | Faster trials, fixed/canvas | Candidate/JS, fixed/canvas |
| --- | ---: | ---: | ---: | ---: |
| Small | -12.75% | -10.82% | 20/21, 20/21 | 6.290×, 5.157× |
| Medium | -11.11% | -9.41% | 21/21, 20/21 | 5.224×, 4.814× |
| Large | -10.28% | -14.89% | 21/21, 19/21 | 5.112×, 4.413× |

The claimed retained range -9.41%…-14.89% and 19–21 wins is supported. Paired change means median per-trial candidate/control ratio; candidate/JS means ratio of separate medians. They are different statistics.

Adverse rows remain visible: five of six ER parse controls have positive paired changes (small Canvas +0.84%; medium fixed +3.06%, Canvas +0.40%; large fixed +1.87%, Canvas +1.75%). ER source small Canvas is +1.44% and medium Canvas +0.26%. Sequence source Canvas is +2.12%. Four rows reverse the sign between paired changes and separate-median ratios: Sequence source fixed, ER parse small Canvas, ER parse large fixed, and ER parse large Canvas. These cannot be selectively reported as universal improvements. This layer leaves retained ER 4.41–6.29× JS and does not satisfy JS parity or a release gate.

This is an ordinary VM diagnostic with shared-page heap/GC/cache pressure and serial warmup. It has balanced cyclic/reverse execution by harness code, but lacks individual trial-order records and the raw harness's per-row quiet guard. No independent p95-operation, RSS, allocation, full-library, startup, or GPU claim follows.

## Actual browser artifacts

Final log records nine successful checks, three per browser. Independently checked all nine final JSON artifacts, beyond the log:

- Each engine has 48 exact JSON/packet result pairs: 42 successful and six error pairs. All result fields, SVG, dimension bits, links, measurement traces, and observed legacy byte arrays match. All 96 transport count records per engine pair exactly; packet success is positively observed.
- All four lanes per engine—ordinary, prepared, raw, caller-raw—successfully transport actual NaN width and height. Recorded bits are `7ff8000000000000` in both transports. This proves actual NaN-result transport, not preservation of a custom input NaN payload through arithmetic.
- Each engine has 14 equal nested/throw/audit/repair pairs (42 total). Alias/cycle/name state, repairs, original arbitrary throw identity, and accessor-audit-before-limit precedence match. Object, undefined, null and Symbol throw cases are present.
- Each engine's normal envelope is **67,108,869 bytes**, five bytes above 64 MiB, and fails with `Browser numeric transport exceeds byte budget`. Each raw case records native success at **67,108,863 bytes**, one byte below the ceiling, followed by `Public binary graph exceeds transport budget` from the combined graph-plus-metadata gate. Old and packet results, callback count, and repair result agree. These are six byte-gate comparisons across three engines.

The initial 54-check run had 48 passes and six ineffective-fixture failures. Those failures were NaN-output and trusted-icon-payload assumptions; archives preserve them. The 45 existing checks passed, as did the three initial error/lifecycle checks. Corrected final packet checks subsequently passed nine of nine. This is 45 existing checks plus nine corrected packet checks, not a fresh complete 369-test suite. The complete 369 run belongs to the preceding raw runtime.

## Fixture provenance and remaining gaps

Source-freeze.json pins test hash `c61f3987745fe148c917766c32cc36af07b49dffdacc64579b64c8c350124d1c` at 7339f38. Commit4031a94 later changes only the strict-overflow formula/assertion; its test hash is `bda1071b1724c228b79b8affa62ac1b4dadfb5f9c50f06db35f3ce7f85cc53d3`. Final archived reports already prove strict overflow by five bytes, so that correction does not invalidate the observed boundary result. Do not label the frozen test as hash4031 or claim this audit re-executed that later test version.

The raw initial graph's zero-versus13-byte metadata threshold remains a separate unexecuted boundary. These result-envelope and final combined-gate tests do not close it. ER nonempty links are currently unreachable and remain untested. Review manifests describing tests as “running” are historical source-review status; the final logs and artifacts supply the execution evidence inspected here.
