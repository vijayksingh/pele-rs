# Fresh-record source/package and arithmetic review

Local frozen-data audit passes. One authorized read-only VM rehash after first-use/profiling found all 106 files in each lane exactly equal to the frozen comparison report. Saved in remote-package-hashes.json. No builds, browsers, timings or production edits by this reviewer.

Runtime source is **bd7df0d609778a33ef261e065017c9e171653d22**. HEAD 31edd85 differs only in documentation. All 448 tracked native/browser/WASM/consumer/test/design source hashes match the bd7 Git blobs and current files. The complete 351 native/SDK input map matches the fresh Swift build manifest. All 13 SDK files were checked, and the SDK host archive equals the explicit aarch64 target archive (`4bfd4da8…`). The actual Swift consumer binary matches its manifest (`f85a25d8…`). Native aggregate is **ee5e9dfc671379d3a6065e8633bc8c6488cee7107343a74380ce73aca0bd5f2b**. This establishes source/artifact linkage, not independently reproducible builds.

The complete 106-file local candidate package matches the VM candidate map. The complete 106-file control map equals the preceding error-brand report's candidate map. Only `host-er-store.js` and `pkg/pele_bg.wasm` differ. Harness/cases hashes match frozen source. This is the default optimized build; the separate O3 experiment is not mixed into these results.

All 24 expected conditions have 21 samples per lane. Medians, paired ratios, faster-trial counts, equal control/candidate batch counts and 1,000 ms warmup minima recompute correctly. The ordinary VM harness records whole SVG/exact-dimension checks before/after timing and complete package hashes. All rows remain in paired-summary.json.

| ER parse | Fixed paired | Canvas paired | Faster trials, fixed/canvas | Candidate/JS, fixed/canvas |
| --- | ---: | ---: | ---: | ---: |
| Small | -7.67% | -8.12% | 16/21, 17/21 | 9.92×, 10.53× |
| Medium | -10.19% | -13.68% | 17/21, 16/21 | 7.40×, 7.55× |
| Large | -17.15% | -12.31% | 20/21, 15/21 | 5.07×, 5.18× |

Parse improves 7.67–17.15% in paired medians, but remains 5.07–10.53× JS. Negative controls are material: **all six ER source-render rows are slower**, +0.86%…+4.05%. Retained ER controls range −3.40%…+1.27%; Sequence controls range −2.51%…+2.67%. The change targets construction; these controls cannot be dropped or attributed wholesale to it. Shared-page GC/cache pressure and serial warmup remain; no per-row quiet guard or recorded individual trial order. No JS parity, release, memory or operation-tail claim follows.

Verified existing evidence: 24 native integrations; 12 initial browser checks; 15 Swift integrations, including 17,437 ER corpus outcomes (7,535 drawings and 9,902 errors, zero explicit budget differences) ; 168 Swift API conditions over 21 workloads;three C consumers compiled and ran successfully. The Swift API report embeds the same fresh build manifest. Correctness log hashes match. Current full 390-browser suite remains running and is not counted as passed. Parent reports first-use 36 rows complete; those timings are not audited here. No new Swift timing is claimed.

Artifacts: source-package-freeze.json carries exact source and full package maps; audit.json carries counts/provenance and every negative control; paired-summary.json carries all 24 rows. verify.py reruns only local data/Git/hash arithmetic and deliberately does not touch the VM. Comparison SHA256: `6e2fd7c158f730156a3fbe269b915e3e7b87c313847c52c9ad1cd37f40b4f36b`.
