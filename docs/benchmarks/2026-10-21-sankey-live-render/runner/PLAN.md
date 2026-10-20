# Detached Sankey callback-parity runner

Prepared only. No package transfer, browser, timing or profiling is authorized by this preparation.

The timing programs and cases remain byte-identical to typed03: warm `1429f77380082b13424f8942824a86437dccf988c9200d228fc487b2437f8272`, first-use `5469078edefad399f109ad29f4c3f25a56ce3834bfdbe4a635c4f3b7ea463bb0`. Each authorized phase uses 24 rows, 21 balanced pairs and 1000 ms warmup. Control remains original default e148 (`ec9c0d...`); all comparison rows include Pele JS. No experimental selector.

`references.json` retains the full 113-file e148 control, full 116-file typed03 predecessor, older e518 map and original report hashes. The new candidate must have 117 files. Relative to typed03, exactly `errors.js`, `full.js`, new `live-render.js`, and `package.json` may change outside generated `pkg/`; all four require a parent-reviewed source hash. Generated files are recorded in full and the WASM hash is separately required. Relative to e148 the additional file count is four.

The untimed continuity probe observes actual public parse, detached prepare/run/finish/Ack materialization and a later helper update, with exact prototype restoration. It verifies original full03 layout identity, no render recipe publication, bounded graph-plus-metadata ACK, and actual helper update identity/revision. Initializer16 and ingress12 probes remain unchanged. Every probe closes its browser before the unchanged timing program starts; package maps are reverified between probes and after timing.

## Parent review manifest

```json
{
  "schemaVersion": 1,
  "sourceCommit": "<exact 40-hex committed candidate>",
  "files": [
    {"path": "bindings/web/errors.js", "sha256": "<64 hex>"},
    {"path": "bindings/web/full.js", "sha256": "<64 hex>"},
    {"path": "bindings/web/live-render.js", "sha256": "<64 hex>"},
    {"path": "bindings/web/package.json", "sha256": "<64 hex>"}
  ]
}
```

Additional reviewed source entries may be included. Duplicate paths are rejected. Candidate pinning remains blocked until the final continuity probe is installed and its manifest is marked ready.

## CLI (not executed)

```sh
python3 dist/perf-patches/parse-architecture/sankey-live-runner/pin-candidate.py \
  --manifest dist/perf-patches/parse-architecture/sankey-live-runtime/package.json \
  --package dist/perf-patches/parse-architecture/sankey-live-runtime/candidate-package \
  --source "$CANDIDATE_SOURCE" --wasm "$CANDIDATE_WASM" \
  --reviewed-source-manifest "$REVIEW_MANIFEST"
python3 dist/perf-patches/parse-architecture/sankey-live-runner/vm-artifacts.py pack
# Transfer only after parent GO; immutable tar SHA and every member recorded.
python3 dist/perf-patches/parse-architecture/sankey-live-runner/vm-artifacts.py transfer --parent-go
```

On the VM under `/home/admin/pele-perf.jL9wzP`, preflight only:

```sh
node parse-architecture/sankey-live-runner/files/scripts/perf/run-source-layout.mjs \
  --preflight-only --phase warm \
  --control parse-architecture/svg-append-e148/candidate-package \
  --candidate parse-architecture/sankey-live-runtime/candidate-package \
  --candidate-manifest parse-architecture/sankey-live-runtime/package.json \
  --candidate-source "$CANDIDATE_SOURCE" --candidate-wasm "$CANDIDATE_WASM" \
  --control-wasm ec9c0d09226f24eaa08318094032728f2f686de892a19aefc1e4508bf4277d48 \
  --output parse-architecture/sankey-live-runner/evidence
```

Replace `--preflight-only` with `--go` only after explicit phase approval, and set `PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers"`. First-use needs its own approval and `--phase first-use`. No concurrent VM job. Fetch with `vm-artifacts.py fetch --phase warm --parent-go` (or first-use) only after the authorized phase exits, then run `verify-results.py warm`. Preserve every raw/adverse row; no cross-window subtraction.

The additional `errors.js` change implements the generic typed JavaScriptFailure adapter for engine-native readonly-helper errors. It adds no package file; package counts remain 117 candidate, 116 typed03, and 113 e148. The expected static delta is four files; generated `pkg/` changes remain fully pinned separately.
