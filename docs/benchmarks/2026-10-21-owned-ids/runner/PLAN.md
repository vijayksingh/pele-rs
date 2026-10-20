# Owned IDs runner — prepared only

No pin, transfer or execution before parent correctness GO. Expected source6c400a83,117 candidate files /117 immediate11f timed control /113 e148 probe-only control. Exactly full.js and graph.js may differ from11f; all other statics and ALL pkg/ WASM/glue/declarations must be identical. Required WASM c0ddd411aebbcd3a2293d66cf7de50f8b0c10fafdf335d7554962922ee42f07c. Review manifest schema1/sourceCommit/files[{path,sha256}] requires both bindings/web/full.js and bindings/web/graph.js; committed Git blobs and packed hashes checked.

Timed1429/5469, casesb16, all36 untimed probes including70851 zero-apply are byte-identical. 11f + candidate + PeleJS;24rows,21 balanced pairs,1000ms warmup. Warm and first-use require separate GO. No experimental selector. Probe failure stops before timing. Source558 Git freeze and370 native-input reuse proof are parent evidence, not new native execution. All generated initializer bytes remain exact11f.

Data-only local pin/pack after candidate freeze:
```sh
python3 dist/perf-patches/parse-architecture/owned-ids-runner/pin-candidate.py \
  --manifest dist/perf-patches/parse-architecture/owned-ids-runtime/package.json \
  --package dist/perf-patches/parse-architecture/owned-ids-runtime/candidate-package \
  --source "$CANDIDATE_SOURCE" --wasm "$CANDIDATE_WASM" \
  --reviewed-source-manifest "$REVIEW_MANIFEST"
python3 dist/perf-patches/parse-architecture/owned-ids-runner/vm-artifacts.py pack
```

Only after explicit parent transfer/phase GO:
```sh
python3 dist/perf-patches/parse-architecture/owned-ids-runner/vm-artifacts.py transfer --parent-go
```

VM `/home/admin/pele-perf.jL9wzP`, inventory must show no other browser/compare/probe job. Preflight-only:
```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node parse-architecture/owned-ids-runner/files/scripts/perf/run-source-layout.mjs \
  --preflight-only --phase warm \
  --control parse-architecture/source-symbols-runtime/candidate-package \
  --probe-control parse-architecture/svg-append-e148/candidate-package \
  --candidate parse-architecture/owned-ids-runtime/candidate-package \
  --candidate-manifest parse-architecture/owned-ids-runtime/package.json \
  --candidate-source "$CANDIDATE_SOURCE" --candidate-wasm "$CANDIDATE_WASM" \
  --control-wasm c0ddd411aebbcd3a2293d66cf7de50f8b0c10fafdf335d7554962922ee42f07c \
  --output parse-architecture/owned-ids-runner/evidence
```
Replace `--preflight-only` with `--go` only for an authorized phase. First-use replaces `--phase warm` with `--phase first-use` under its separate GO. Preserve raw preflight/probe failures; stop before timing on invalid probe. No retry without review.

After successful completion:
```sh
python3 dist/perf-patches/parse-architecture/owned-ids-runner/vm-artifacts.py fetch --phase warm --parent-go
python3 dist/perf-patches/parse-architecture/owned-ids-runner/verify-results.py warm
```

Immutable archive verifies every transferred file, refuses conflicting remote bytes, and preserves all historical evidence. Report all adverse controls and every raw row. No isolated effect inferred from comparisons in different windows. No new profile or fourth lane.

## Same-run isolation

The timing control is the immediate frozen11f package; third lane is pinned PeleJS. Both native packages share all pkg bytes/WASM; only full.js+graph.js differ. This measures the Map representation layer relative to its parent, not original e148 or release parity. All24 rows/21 balanced pairs and negative controls retained.

Unchanged continuity/initializer probes still require a legacy e148 control, so they run against a SEPARATE probe-only e148 package; all three packages rehashed before/after probes and timing. These probe browsers close before timed11f/candidate/JS comparison. The immediate11f control's full prior36-check proof is reused by exact raw/provenance/verification hashes; timed output/oracle checks remain freshly executed for BOTH11f and candidate. No false claim that legacy-only probes selected11f. No e148 timed run.
