# Numeric live-read ABI runner

Prepared only. Candidate source/WASM/package pins are unset. No transfer, browser, timing, or profiling until parent GO.

Comparison remains original e148 default control plus Pele JS, 24 rows (Sankey three sizes + Flowchart medium; fixed/canvas; three stages), 21 balanced pairs and 1000ms warmup. Warm and first-use each require separate GO. Timing harnesses, cases, and all three untimed probes are byte-identical to sankey-live-runner. Every probe closes its browser before timing. No experimental selector.

Candidate and previous8175006 each have117 files; e148 control113. Relative to8175006 only `live-render.js` may differ outside generated `pkg/`; no additions/removals. Generated glue/declarations/WASM recorded completely. Parent-reviewed manifest must contain both `bindings/web/live-render.js` and `crates/pele-wasm/src/detached_render.rs`; pinning verifies their committed Git blobs and packed JS hash. This does not claim the candidate WASM was built from those blobs; the parent's frozen source/build/package provenance supplies that proof.

Review manifest:
```json
{"schemaVersion":1,"sourceCommit":"<40hex>","files":[{"path":"bindings/web/live-render.js","sha256":"<64hex>"},{"path":"crates/pele-wasm/src/detached_render.rs","sha256":"<64hex>"}]}
```

Data-only local pin/pack after candidate freeze:
```sh
python3 dist/perf-patches/parse-architecture/live-read-numeric-runner/pin-candidate.py \
  --manifest dist/perf-patches/parse-architecture/live-read-numeric-runtime/package.json \
  --package dist/perf-patches/parse-architecture/live-read-numeric-runtime/candidate-package \
  --source "$CANDIDATE_SOURCE" --wasm "$CANDIDATE_WASM" \
  --reviewed-source-manifest "$REVIEW_MANIFEST"
python3 dist/perf-patches/parse-architecture/live-read-numeric-runner/vm-artifacts.py pack
```

Only after explicit parent transfer/phase GO:
```sh
python3 dist/perf-patches/parse-architecture/live-read-numeric-runner/vm-artifacts.py transfer --parent-go
```

VM `/home/admin/pele-perf.jL9wzP`, inventory must show no other browser/compare/probe job. Preflight-only:
```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node parse-architecture/live-read-numeric-runner/files/scripts/perf/run-source-layout.mjs \
  --preflight-only --phase warm \
  --control parse-architecture/svg-append-e148/candidate-package \
  --candidate parse-architecture/live-read-numeric-runtime/candidate-package \
  --candidate-manifest parse-architecture/live-read-numeric-runtime/package.json \
  --candidate-source "$CANDIDATE_SOURCE" --candidate-wasm "$CANDIDATE_WASM" \
  --control-wasm ec9c0d09226f24eaa08318094032728f2f686de892a19aefc1e4508bf4277d48 \
  --output parse-architecture/live-read-numeric-runner/evidence
```
Replace `--preflight-only` with `--go` only for an authorized phase. First-use replaces `--phase warm` with `--phase first-use` under its separate GO. Preserve raw preflight/probe failures; stop before timing on invalid probe. No retry without review.

After successful completion:
```sh
python3 dist/perf-patches/parse-architecture/live-read-numeric-runner/vm-artifacts.py fetch --phase warm --parent-go
python3 dist/perf-patches/parse-architecture/live-read-numeric-runner/verify-results.py warm
```

Immutable transfer tar includes hashes of every input and refuses different existing remote bytes. Historical817 runner/reports and full previous116/113 maps remain preserved. Report all rows/adverse controls and paired versus ratio-of-medians distinctions. This comparison is not an isolated numeric-ABI measurement versus817; no cross-window compounding.
