# Source symbols runner

Prepared only. Candidate pins unset; no transfer or execution authorized. Expected source11f27516; final package/build/source manifest must be supplied after correctness GO.

Unchanged e148 default control + Pele JS,24 rows,21 balanced pairs,1000ms warmup. Warm and first-use require separate GO. Timed harnesses1429/5469, casesb16 and all36 untimed probes are byte-identical to accepted-baseline. Continuity70851 requires zero post-ACK helper applies and preserves actual jobs/ACK/alias/revision checks. Every probe browser closes before timing; no experimental selectors.

Candidate117 files, a4 prior117, e148 control113. NO static bytes may differ from a4 (including generated source-initializer.js). Only generated pkg/ files may change; new WASM hash required; all generated differences recorded. Review manifest schema1 has sourceCommit and files[{path,sha256}] for all six Rust Sankey sources: indexed,layout,parser,public_model,render,source_store. Pin verifies committed Git blobs and exact frozen package map. Source/build correctness comes from parent evidence, not these hashes alone.

Data-only local pin/pack after candidate freeze:
```sh
python3 dist/perf-patches/parse-architecture/source-symbols-runner/pin-candidate.py \
  --manifest dist/perf-patches/parse-architecture/source-symbols-runtime/package.json \
  --package dist/perf-patches/parse-architecture/source-symbols-runtime/candidate-package \
  --source "$CANDIDATE_SOURCE" --wasm "$CANDIDATE_WASM" \
  --reviewed-source-manifest "$REVIEW_MANIFEST"
python3 dist/perf-patches/parse-architecture/source-symbols-runner/vm-artifacts.py pack
```

Only after explicit parent transfer/phase GO:
```sh
python3 dist/perf-patches/parse-architecture/source-symbols-runner/vm-artifacts.py transfer --parent-go
```

VM `/home/admin/pele-perf.jL9wzP`, inventory must show no other browser/compare/probe job. Preflight-only:
```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node parse-architecture/source-symbols-runner/files/scripts/perf/run-source-layout.mjs \
  --preflight-only --phase warm \
  --control parse-architecture/svg-append-e148/candidate-package \
  --candidate parse-architecture/source-symbols-runtime/candidate-package \
  --candidate-manifest parse-architecture/source-symbols-runtime/package.json \
  --candidate-source "$CANDIDATE_SOURCE" --candidate-wasm "$CANDIDATE_WASM" \
  --control-wasm ec9c0d09226f24eaa08318094032728f2f686de892a19aefc1e4508bf4277d48 \
  --output parse-architecture/source-symbols-runner/evidence
```
Replace `--preflight-only` with `--go` only for an authorized phase. First-use replaces `--phase warm` with `--phase first-use` under its separate GO. Preserve raw preflight/probe failures; stop before timing on invalid probe. No retry without review.

After successful completion:
```sh
python3 dist/perf-patches/parse-architecture/source-symbols-runner/vm-artifacts.py fetch --phase warm --parent-go
python3 dist/perf-patches/parse-architecture/source-symbols-runner/verify-results.py warm
```

Immutable transfer tar retains every input hash and refuses overwriting different remote bytes. Preserve all historical runner/data; no cross-window isolated-gain claim. No old native-reuse proof applies: fresh default WASM/native correctness belongs to the parent build. Report all adverse controls and all raw samples.
