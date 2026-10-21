# Public raw ER first-adoption diagnostic

Private harness only. No benchmark, browser, VM, build or integration execution by its author. This is one bounded diagnostic; the full web gate remains separate.

The default matrix has 18 rows: ER small/medium/large × fixed/Canvas × three stages. Each row has 15 balanced three-way trials (`js`, frozen preceding host package, raw-adoption candidate), 1000 ms warmup per lane, median-of-three calibration targeting 12 ms and maximum 128 operations. All three lanes use the same batch count, chosen as the minimum calibrated count. Raw batch durations, per-operation samples, trial order, medians, p95 batch means, median paired ratio and pair wins remain in the report. Median-of-ratios and ratio-of-medians are separate fields.

## Timer boundary

- `raw-first-inclusive`: descriptor/alias-preserving fresh caller-root clone, `renderer.renderRaw('er', root, config, options)` (pinned JS uses actual ER entry.render), all adoption/structural admission/config handling, complete first render, checksum and native disposal. Every operation gets a fresh root never previously rendered, validated or adopted. Warmup and calibration use other fresh roots with the same boundary. Renderer instances and shared schema/font caches remain warm; this does not claim first-ever registry initialization cost.
- `clone-only`: the same fixture clone, measured separately for disclosure. Do not subtract its median from another row: GC/cache/JIT and owner lifecycle differ.
- `raw-repeat`: a fresh clone and one initial raw render occur outside each batch timer, then the same retained root is rendered `count` times. Disposal is outside this timer. This row deliberately measures retained rendering, not first adoption.

The oracle parses each deterministic source once per row to supply an untouched seed outside timing. Cloning copies data descriptors/collection membership through one identity map, strips the database class prototype, and rejects callable/symbol/accessor fixtures. Neither native parser runs. No root prepared for a measured first operation is pre-rendered by correctness/warmup. Full SVG, exact numeric dimensions, links, complete data descriptors/alias graph and key retained identities are compared with the actual pinned ER entry before/after every row. Results must match both control and candidate; this unchanged-model diagnostic deliberately does not benchmark the callback cases whose old control is known incorrect.

## Copy and run

Run from the Linux VM repository with its existing dependencies and Playwright browser. Copy this whole `benchmark/` directory unchanged, plus the independently frozen packages and the preceding accepted comparison report. Do not install/build during a coordinated timing window.

The runner dynamically imports two repository files that must also exist on the VM, including in `--freeze-inputs` mode:

- `scripts/perf/cases.mjs`
- `scripts/perf/swift/build.mjs` (shared `guardQuiet` and `systemProcesses`; importing it does not build Swift)

Upload these exact current files to the same paths under the VM repository root before freezing. For an already frozen attempt, never replace the harness modules or oracle pins. Example dependency-only upload from the local repository (set the actual SSH target and VM repository path):

```sh
export RAW_VM="admin@YOUR_VM"
export RAW_VM_ROOT="/absolute/path/to/VM/repository"
ssh "$RAW_VM" "mkdir -p '$RAW_VM_ROOT/scripts/perf/swift'"
scp scripts/perf/cases.mjs "$RAW_VM:$RAW_VM_ROOT/scripts/perf/cases.mjs"
scp scripts/perf/swift/build.mjs "$RAW_VM:$RAW_VM_ROOT/scripts/perf/swift/build.mjs"
```

Also required before the timing window: existing `@playwright/test` and its Chromium installation; `node_modules/pele/dist/pele.js` plus the complete `node_modules/pele/dist/esm/` tree matching `oracle-pins.json`; both package directories; and the control proof JSON. The guard requires the VM's `ps` command. Do not install missing dependencies during timing. Dependency hashes are frozen and rechecked by the runner.

Attempt 1 stopped before input freeze because `scripts/perf/swift/build.mjs` was absent; no timing samples were accepted from it. Its log is retained. The parent uploaded the exact guard for attempt 2; this documentation amendment does not change `run.mjs`, `browser.mjs` or `oracle-pins.json`.

Set these paths explicitly (examples are shell variables, not inferred artifact identities):

```sh
export RAW_BENCH="$PWD/dist/perf-patches/parse-architecture/er-raw-host/benchmark"
export RAW_CONTROL="/absolute/path/to/frozen-pre-raw/package"
export RAW_CANDIDATE="/absolute/path/to/frozen-raw-candidate/package"
export RAW_CONTROL_PROOF="$PWD/parse-architecture/er-raw-host-first/ordinary-comparison.json"
export RAW_INPUTS="$PWD/dist/evidence/er-raw-inputs.json"
export RAW_OUTPUT="$PWD/dist/evidence/er-raw-first-vm.json"
```

First, freeze inputs without launching a browser or taking timings. The control must match all package hashes under the proof report's `runtimeHashes.control`: the parent's `er-raw-host-first/ordinary-comparison.json` records the preceding prepared-host package on that side. If choosing another report, explicitly select and verify its correct side. Existing manifests are not overwritten.

```sh
node "$RAW_BENCH/run.mjs" --root "$PWD" \
  --control "$RAW_CONTROL" --candidate "$RAW_CANDIDATE" \
  --control-proof "$RAW_CONTROL_PROOF" --control-proof-side control \
  --inputs "$RAW_INPUTS" --freeze-inputs
```

Then, after coordinating a quiet VM and verifying no other browser/build/test job remains:

```sh
TZ=UTC node "$RAW_BENCH/run.mjs" --root "$PWD" \
  --control "$RAW_CONTROL" --candidate "$RAW_CANDIDATE" \
  --control-proof "$RAW_CONTROL_PROOF" --control-proof-side control \
  --inputs "$RAW_INPUTS" --output "$RAW_OUTPUT" --quiet-run \
  --trials 15 --warmup-ms 1000 \
  --cases er/small,er/medium,er/large --profiles fixed,canvas \
  --stages clone-only,raw-first-inclusive,raw-repeat
```

For bounded windows, explicitly select one case/profile, give every invocation a distinct output path, and retain every attempt. Do not pool selected successful timings while silently discarding slow completed rows. Interrupted/guard-rejected reports stay separate. A `.partial` or `.failed` report is not an accepted complete run.

## Provenance and guard limits

The manifest freezes every control/candidate/ESM oracle file, the pinned monolithic oracle bundle, both harness modules, shared cases, existing quiet guard and the control proof. The report rechecks all hashes before/after. Independent control/candidate real paths are required; identical package hashes are rejected. The existing oracle bundle SHA is mandatory. ESM files are separately pinned by `oracle-pins.json`, copied with this harness.

Uses existing `scripts/perf/swift/build.mjs` guardQuiet before browser launch, before/after every row and at completion. It rejects build/Pele-test processes; a separate startup check rejects any already running Chromium/Firefox/WebKit process. High CPU process snapshots are recorded but are not an exclusivity guarantee. One browser/context/page holds independent package module/WASM instances. Cross-origin isolation and observed clock resolution ≤0.01 ms are mandatory. Overall deadline is ten minutes. Browser version, machine, free memory/load and all guard snapshots are retained.

## Review pitfalls

- First-inclusive is an end-to-end caller fixture plus raw API cost; it is not isolated native adoption latency. Cloning/disposal are intentionally included and disclosed.
- Serial warmup, shared JS heap/GC, package caches and Canvas/font caches can influence rows. These are warm CPU API numbers, not startup, paint, GPU or memory results.
- Repeated rendering intentionally excludes first adoption. Never combine it with the first-inclusive result into an unlabelled headline.
- Fifteen triples balance first-position counts; the control/candidate relative order differs by one because 15 is odd. Keep actual order in analysis.
- A control proof checks exact package identity, not whether a human chose the intended preceding revision; review its provenance before freezing.
- No callback mutation speed claim, full-regression-absence claim or JS parity/release claim follows from these 18 rows.
