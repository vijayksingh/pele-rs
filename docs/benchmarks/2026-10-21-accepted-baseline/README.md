# Accepted-baseline authority: performance remains below gate

Source `a4f933d9af2cd3ccd5cda7eac46dc1048a296249` binds accepted graph/root authority to completed observation and publication. Three JavaScript files change relative to `7417fed`; the 117-file package retains WASM `96f4e39f326ad08866ec010bf53b3df97dcf19f98c9a7699b6594be3679507c1`. The matched diagnostic below records this checkpoint against the original Rust control and Pele JS. Complete API parity and release gates remain unmet.

## Matched VM diagnostic

The run compares `a4f933d` with original Rust control `e1486055d596d4e0ac25e4c78f1d5d95a73f781a` and pinned Pele JS: **24 conditions, 21 balanced paired trials each, 1000 ms warmup**, matched Rust batch counts. All samples, exact output checks, executed harness files and 36 untimed activation/continuity/ingress probes are retained.

All six Sankey retained conditions remain **11.56-14.55% slower than e148**, with 0-4 lower trials out of 21, and **2.05-3.09x JS**. Parse remains **7.18-13.84x JS**: medium/large paired changes are -33.29% to -25.63%, while small fixed is +2.18%. Small Canvas has a -1.08% paired median but an adverse ratio of lane medians; both statistics remain explicit.

All six Sankey source controls are adverse (+0.39% to +2.40%). Every Flowchart paired control is also adverse: parse +3.45%/+6.90%, retained +2.51%/+8.78%, source +0.19%/+0.48%. The [complete table](RESULTS.md) preserves all absolute times and lower-pair counts; the raw report and paired summary retain both comparison metrics. There are 19 adverse paired rows and 19 adverse ratios of lane medians, with differing membership.

This is one matched window versus e148, whose behavior predates the callback fixes. The preceding numeric-ABI window is separate: no isolated accepted-baseline gain versus `7417fed`, cross-window compounding or neutrality of negative controls is claimed. First-use and the complete 126-condition gate were not rerun. The historical complete release gate remains 126/126 failing. No 0.3 release.

The untimed continuity probe expectation changed from one redundant apply to zero only after the new public feature demonstrated three real jobs and finishes with zero applies. The executed probe still checks job dispatch, alias/update identity and revisions. Its preparatory review status is archived verbatim; completed probe reports prove execution. The first package-pin attempt stopped before transfer/browser launch due to the stale precommit revision record. The committed-package provenance correction and authorized second pin attempt are both preserved; no timed attempt was discarded.

## Correctness and source identity

Fresh checks pass: **122 Node oracle comparisons, 114 browser tests** (111 consolidated plus three focused tests across Chromium/Firefox/WebKit), TypeScript, and **168 Swift API conditions across 21 cases**. Two generated initializer outputs match exactly. Actual browser JSON reports, full Node/API output and logs are retained.

The focused feature exercises normal/raw rendering, live edits, arbitrary callback/factory throws, failed observations, partial helper writes, nested helpers, aliases and recovery. Three genuine unchanged render jobs finish with zero redundant apply calls. The prior package fails this assertion with two apply calls for those same three jobs. The first attempt failed before execution because the configured headless browser executable was absent; that launch failure and the subsequent genuine RED assertion are preserved separately. Neither is counted as a pass.

All 370 native/SDK inputs are unchanged. The existing `8175006` SDK (`4ecfbbfb33ce6a0fba349ed6e9340074ba0fce37d28d2e35603b6e6a027327bd`) and consumer (`a5f4427f7b8005b949f9d2c49053692ed5b0ef13421c5fda60e44b20ada218e3`) are reused. Native 57, Swift 16 corpus and C 3 results belong to that earlier fresh build; they were not rerun for this JavaScript-only checkpoint. The API 168 check is fresh.

The original source and package records correctly name the earlier HEAD plus working changes. The original package record is preserved separately as `package-before-commit.json.gz`. Before VM execution, its stale revision pin was rejected; `package.json.gz` now derives the committed identity from the final `after.json.gz` record, citing both predecessor hashes. Exact source and package maps are unchanged, and the final record binds all 558 source inputs to committed `a4f933d`. This metadata correction changes no candidate bytes and triggers no rebuild. The reviewed three-file map and unchanged-WASM package delta are included. This is recorded artifact provenance, not an independent rebuild.

## Retained CPU profiles

Only the second profile attempt is valid: four five-second medium/large fixed retained windows, with parsing outside the windows and exact SVG/dimensions/ordered resource traces checked before/after. Candidate preparation is 10.33%/10.16% inclusive, predominantly matching (9.97%/9.84%). No snapshot, sparse encoding or native apply frames appear in these samples. Zero samples are not invocation counts; the separate feature and warm probes establish zero redundant apply calls on their unchanged inputs.

The live-reader ancestor union is 20.63%/21.76%, overlapping geometry and other render work. Live-phase decode leaves are 0.82%/0.28%; total decode leaves are 8.43%/7.92%. Ancestor shares overlap and cannot be summed into a removable-cost ceiling. No parser runs in these retained windows; the inherited automatic parser-name category can misclassify `IndexedSankey` type names. Raw weighted samples and corrected attribution are retained in [the profile report](profile/RESULTS.md).

Attempt one stopped on the first control profile because CDP returned two negative time deltas (-224 and -409 microseconds). No candidate window ran. Its raw profile, failure manifest and cleanup/input-hash proof are preserved separately. One authorized fresh four-window attempt passed the same nonnegative-weight requirement. No clamping, dropped samples or pooled attempts are used; warm timing results are unaffected.

```sh
python3 docs/benchmarks/2026-10-21-accepted-baseline/verify.py
python3 docs/benchmarks/2026-10-21-accepted-baseline/verify-profile.py
```

Verification is data-only: hashes, exact decompression, recorded source/package continuity, correctness totals, focused observable outcomes and generator equality. Gzip uses deterministic `mtime=0`; the manifest binds both original and stored bytes. No browser or benchmark is launched. All 24 paired rows are recomputed from the raw samples; executed package/harness identities and the zero-apply untimed probe are checked. Historical runs remain separate.
