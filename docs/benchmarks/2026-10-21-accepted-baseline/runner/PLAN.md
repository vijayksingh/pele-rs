# Accepted-baseline runner

Prepared only. Candidate source/WASM/package pins are unset. No transfer, browser, timing, or profiling until parent GO.

Comparison remains original e148 default control plus Pele JS, 24 rows (Sankey three sizes + Flowchart medium; fixed/canvas; three stages), 21 balanced pairs and 1000ms warmup. Warm and first-use each require separate GO. Timing harnesses, cases, initializer and ingress probes remain byte-identical. The new continuity probe changes only the authorized post-ACK helper apply expectation1→0; all real job/ACK/publication/alias/revision checks remain. Every probe closes its browser before timing. No experimental selector.

Candidate and previous7417fed each have117 files; e148 control113. Relative to7417fed only `full.js`, `graph.js`, `graph-storage.js` may differ; all generated `pkg/` bytes MUST remain identical; no additions/removals. Generated glue/declarations/WASM recorded completely and equal prior7417. Required WASM96f4e39f326ad08866ec010bf53b3df97dcf19f98c9a7699b6594be3679507c1. Parent-reviewed manifest must contain all three `bindings/web/full.js`, `bindings/web/graph.js`, and `bindings/web/graph-storage.js`; pinning verifies their committed Git blobs and packed JS hash. This does not claim native input continuity; the parent's frozen source/build/package provenance supplies that proof.

Review manifest:
```json
{"schemaVersion":1,"sourceCommit":"<40hex>","files":[{"path":"bindings/web/full.js","sha256":"<64hex>"},{"path":"bindings/web/graph.js","sha256":"<64hex>"},{"path":"bindings/web/graph-storage.js","sha256":"<64hex>"}]}
```

Data-only local pin/pack after candidate freeze:
```sh
python3 dist/perf-patches/parse-architecture/accepted-baseline-runner/pin-candidate.py \
  --manifest dist/perf-patches/parse-architecture/accepted-baseline-runtime/package.json \
  --package dist/perf-patches/parse-architecture/accepted-baseline-runtime/candidate-package \
  --source "$CANDIDATE_SOURCE" --wasm "$CANDIDATE_WASM" \
  --reviewed-source-manifest "$REVIEW_MANIFEST"
python3 dist/perf-patches/parse-architecture/accepted-baseline-runner/vm-artifacts.py pack
```

Only after explicit parent transfer/phase GO:
```sh
python3 dist/perf-patches/parse-architecture/accepted-baseline-runner/vm-artifacts.py transfer --parent-go
```

VM `/home/admin/pele-perf.jL9wzP`, inventory must show no other browser/compare/probe job. Preflight-only:
```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node parse-architecture/accepted-baseline-runner/files/scripts/perf/run-source-layout.mjs \
  --preflight-only --phase warm \
  --control parse-architecture/svg-append-e148/candidate-package \
  --candidate parse-architecture/accepted-baseline-runtime/candidate-package \
  --candidate-manifest parse-architecture/accepted-baseline-runtime/package.json \
  --candidate-source "$CANDIDATE_SOURCE" --candidate-wasm "$CANDIDATE_WASM" \
  --control-wasm ec9c0d09226f24eaa08318094032728f2f686de892a19aefc1e4508bf4277d48 \
  --output parse-architecture/accepted-baseline-runner/evidence
```
Replace `--preflight-only` with `--go` only for an authorized phase. First-use replaces `--phase warm` with `--phase first-use` under its separate GO. Preserve raw preflight/probe failures; stop before timing on invalid probe. No retry without review.

After successful completion:
```sh
python3 dist/perf-patches/parse-architecture/accepted-baseline-runner/vm-artifacts.py fetch --phase warm --parent-go
python3 dist/perf-patches/parse-architecture/accepted-baseline-runner/verify-results.py warm
```

Immutable transfer tar includes hashes of every input and refuses different existing remote bytes. Historical7417/817 runner/reports and full previous116/113 maps remain preserved. Report all rows/adverse controls and paired versus ratio-of-medians distinctions. This comparison is not an isolated accepted-baseline measurement versus7417; no cross-window compounding.

The new public feature test is outside package files. Native370-input reuse proof lives at accepted-baseline-runtime/native-proof-reuse.json; retain it with the parent source/package freeze. No simultaneous extra7417 lane is added: the unchanged harness has e148/candidate/JS only.

## Authorized receipt probe delta

Previous probe SHA `ca4df47b5a31146f625a0e23aac3bed0aecccf1ad49fb9422627bf4ccf432de4`; new probe SHA `70851eb99e6f303ccf5ddbb1801e8cde0a11e6abcf467ddace21686553a822ee`. Parent-authorized accepted-baseline receipt: completed publication and unchanged helper-argument observation renew the accepted receipt. Three real jobs/finish and zero applies were proven in raw and high-level public feature checks. Existing helper alias/publication/update identity and ACK+acceptedApplyCalls+1 revision formula remain exact. Historical runner/probes remain unchanged. This private updated probe has not yet run.
