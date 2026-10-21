# Parent execution plan: source-result packets

Prepared only. No commands below have been executed for this candidate. Private runtime/test patch stays frozen at `54bb4387ac4cc78e7a8e7fd0dfecbef66ae4a05e0c4db261f2bc4d75723d15f3` pending Boundary review. Execute from repository root. Never call `package-release.sh` for this slice: it unnecessarily rebuilds native products and performs unrelated packaging.

## 1. Apply once and build the actual default package

Finish review, record current commit/worktree and retain the frozen dense package before applying. Control package is `dist/perf-patches/parse-architecture/er-dense-array/package`; verify its bytes against the existing dense source/package freeze. Do not take control from `target/` or a package left by a rejected experiment.

```bash
set -euo pipefail
mkdir -p dist/evidence dist/perf-patches/parse-architecture/source-result-packet/runtime
shasum -a 256 dist/perf-patches/parse-architecture/source-result-packet/candidate.patch
git apply --check dist/perf-patches/parse-architecture/source-result-packet/candidate.patch
git apply dist/perf-patches/parse-architecture/source-result-packet/candidate.patch
git diff --check
CARGO_BUILD_JOBS=2 PELE_WASM_OPT_LEVEL=s bash scripts/build-wasm.sh >dist/evidence/source-result-packet-wasm-build-attempt1.log 2>&1
npm run test:types >dist/evidence/source-result-packet-types.log 2>&1
```

`build-wasm.sh` checks wasm-bindgen 0.2.128, compiles current Rust for wasm32, generates glue, copies the pinned development oracle, npm-packs the facade and extracts that exact archive into `dist/package-web/package`. It is the single required fresh build. Freeze the resulting complete package into a new private `source-result-packet/package` directory, with all file hashes, source/patch identities, build log and package archive hash. Do not change or overwrite either comparison package after freezing.

Use the normal exact package-freeze workflow already used for dense-array. Confirm both new capabilities exist in generated glue, and `render_packet.rs` belongs to the compiled source set. A package that merely contains new `full.js` with old WASM silently uses fallback and is not candidate proof.

## 2. Focused global-transport correctness, then one full browser run

Use all three configured browser projects, one worker. Commands deliberately omit `--project`, so Chromium, Firefox and WebKit all execute. Preserve both logs and explicit test output JSON. `scripts/serve.mjs` already serves `.json.gz` fixture fallbacks; do not regenerate oracle fixtures merely because uncompressed JSON is absent.

```bash
npx playwright test \
  tests/web/source-result-packet.spec.js \
  tests/web/all-render-numbers.spec.js \
  tests/web/render-numbers.spec.js \
  tests/web/foreign-providers.spec.js \
  tests/web/er-result-packet.spec.js \
  --workers=1 --output=dist/perf-patches/parse-architecture/source-result-packet/web-focused-attempt1 \
  >dist/evidence/source-result-packet-web-focused-attempt1.log 2>&1
```

Gate content:

- New source feature: all 33 normal source families plus actual nonempty links, full SVG/result shape/ordered keys, exact dimension bits/counts, original callbacks, nested source rendering, provider isolation, closed/empty registries, bounded envelope overflow and repair.
- Actual normal-loader capability absence uses the legacy registry method. The top-level optional export is separately masked through the existing exported internal factory; no new public loader mode is claimed.
- All-render-numbers: existing 3069 source/immutable/mutable comparisons, full corpus numeric normalization and resource sequence. Source packets must actually be selected in normal loaders.
- Foreign providers: complete existing live provider/preparation/config/identity controls; no packet selection for provider-owned rendering.
- Existing ER result packets: shared borrowed encoder preserves host finish/audit/count, raw/normal variants, exact scalar bits and combined graph-plus-metadata boundary.

If focused checks pass, run the complete browser suite **once** for this global transport layer:

```bash
npx playwright test --workers=1 \
  --output=dist/perf-patches/parse-architecture/source-result-packet/web-full-attempt1 \
  >dist/evidence/source-result-packet-web-full-attempt1.log 2>&1
```

Record discovered test count rather than copying an old 390 total. If a fixture fails, retain first-attempt evidence, diagnose exact lane/error and change only the justified slice; do not repeatedly rerun unrelated suites. Full logs plus explicit JSON are required because list-reporter attachments alone have previously been lost. No browser job overlaps local timing; VM timing uses a separate host but only one VM browser job.

## 3. Native/Swift negative-control identity and conditional reuse

This candidate changes only WASM implementation and web facade/test files. Native `inputs(root)` is expected to remain 351 entries with aggregate:

`439fe2e6a023eebcafb980508b447162cdd5cd9c07519200df47f5ab3fcaea28`

Frozen dense Swift executable:

`554ef37b648587962511c5e9eaa7ef639c469d87be601996817a1c9d762957fa`

Reference: `er-dense-array/candidate-swift/swift-build-manifest.json`, its sibling binary, and `er-dense-array/runtime/{correctness,c-consumers}.json`. The recorded build revision was precommit dense WIP; `er-dense-array/runtime/commit-binding.json` binds its unchanged inputs to committed 7701135. Do not relabel that build as a newly compiled source-packet binary.

On parent's request, use the safe imported `inputs` API and compare **all** entries plus aggregate, current SDK archive/host-slice bytes, frozen consumer bytes and active consumer manifest. Importing `build.mjs` does not build. If they match, retain a reuse-verification JSON and existing native build proof. Do not rebuild the SDK/consumer or run Swift latency comparisons for a WASM-only change. If they differ, stop reuse and explain the actual changed paths before deciding whether a rebuild is relevant.

Only after parent requests local correctness, run sequentially with new prefixes:

```bash
PELE_VERIFY_MUTABLE=1 PELE_FAMILIES=er swift test -c release --package-path bindings/swift \
  --filter 'AdditionalDiagramTests/testPinnedDiagramsThroughNativeDocuments|EagerProjectionIntegrationTests|PreparedDocumentTests' \
  >dist/evidence/source-result-packet-swift-corpus.log 2>&1

for consumer in ffi ffi-public ffi-parse-limits; do
  cc -Wall -Wextra -Werror -std=c11 -I include "tests/$consumer.c" \
    bindings/swift/Artifacts/CPele.xcframework/macos-arm64/libpele_ffi.a \
    -o "dist/evidence/source-result-packet-$consumer" \
    >"dist/evidence/source-result-packet-$consumer-compile.log" 2>&1
  "dist/evidence/source-result-packet-$consumer" \
    >"dist/evidence/source-result-packet-$consumer-run.log" 2>&1
done

node scripts/perf/swift/run.mjs \
  --output dist/evidence/source-result-packet-swift-api.json \
  >dist/evidence/source-result-packet-swift-api.log 2>&1
```

Expected 15 Swift cases (ER corpus + 7 eager + 7 prepared), 3 strict C consumers and 168 consumer API conditions. The normal Swift test command checks incremental freshness; it does not require an SDK rebuild. The API command has no `--quiet-run`, so it performs correctness only and verifies current input/binary hashes itself. It uses the active `.build/swift-build-manifest.json`; validate that manifest before execution. If active output differs but the frozen executable is valid, explicitly select/restore the matching consumer manifest/binary as a separate approved setup step, not a silent overwrite. No unchecked `--skip-build` against an unidentified XCTest bundle.

## 4. Independent VM diagnostic: preselect one matrix

Preferred first diagnostic: 7 families × 3 sizes × 2 resources × 3 stages = **126 conditions**, each 9 balanced three-way trials and 500 ms warmup per runtime. Source is 42 rows; parse and retained together are **84** controls. A proposed split of “42 source + 42 controls” would omit half the full control matrix. Do not select favorable rows after results.

The existing `compare-web-vm.mjs` enforces:

- pinned JS SHA and oracle revision;
- high-resolution isolated clock;
- exact complete SVG and `Object.is` dimensions before/after;
- matched control/candidate batch counts, max128, median-of-three calibration;
- balanced three-way cyclic/reverse order;
- complete package input hashes before/after;
- a 600000 ms whole-run watchdog and retained `.partial` progress.

It records host load but does **not** implement the local Swift process quiet guard. Ensure VM exclusivity separately; do not invent a stronger guard claim. Links, ordering, errors and raw NaN payload correctness come from E2E; the timing harness checks SVG/dimensions, not every public result field. Shared browser heaps/GC/cache and parse/disposal included in measured API costs remain explicit limitations.

Upload unchanged comparison harness, `cases.mjs`, pinned `node_modules/pele`, matching Node/Playwright dependency installation and both complete immutable packages into `/home/admin/pele-perf.jL9wzP`. Reuse the established VM environment; no install or browser download during the window. Control must be freshly physically hashed after transfer, not inferred from a second local copy. Suggested package paths:

- control: `parse-architecture/er-dense-array-first/package`
- candidate: `parse-architecture/source-result-packet-first/package`

On VM, after one-browser exclusivity is confirmed:

```bash
set -euo pipefail
cd /home/admin/pele-perf.jL9wzP
source_packet_cases="$(node --input-type=module -e 'import {cases} from "./scripts/perf/cases.mjs";console.log(cases().map(c=>c.id).join(","))')"
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/compare-web-vm.mjs \
  --control parse-architecture/er-dense-array-first/package \
  --candidate parse-architecture/source-result-packet-first/package \
  --cases "$source_packet_cases" --profiles fixed,canvas \
  --stages render-source,parse,render-model --trials 9 --warmup-ms 500 \
  --output parse-architecture/source-result-packet-first/comparison-attempt1.json \
  >parse-architecture/source-result-packet-first/comparison-attempt1.log 2>&1
```

Transport SSH flags remain `ssh -T -o RemoteCommand=none -o RequestTTY=no vm-agent`. No benchmark is launched by this plan. Confirm actual frozen control path before invoking; if it is absent, transfer the hash-bound dense control there rather than substituting another baseline.

Alternative, chosen **before any timing** if greater source precision is preferred: the same 21 cases/fixed+Canvas with `--stages render-source --trials 21 --warmup-ms 1000` (42 rows), followed by a separate report with `--stages parse,render-model --trials 9 --warmup-ms 500` (84 controls). These are two predefined conditions, not repeated attempts to obtain a win. Keep reports separate; no pooling their samples/settings.

On watchdog/failure/interruption, preserve entire attempt and partial report. Do not pool it with a replacement; diagnose and predeclare a new bounded run if needed. Rehash both physical VM package directories after timing with an independent Node reader. Fetch raw report/log and rehash evidence; bind local source/package freeze, immutable commit, harness, cases and oracle. Report all source and control rows, paired median ratios and wins plus standalone medians; retain adverse rows. No “noise” or regression-free claim without support.

This comparison is diagnostic, not the JS parity release gate. A subsequent explicit `web-vm.mjs` full gate can report current parity; neither this matrix nor prior failures authorize a 0.3 claim. Profile only after timing, using the same frozen package and no competing VM browser. No custom SVG counter or width-string cache enters this experiment.
