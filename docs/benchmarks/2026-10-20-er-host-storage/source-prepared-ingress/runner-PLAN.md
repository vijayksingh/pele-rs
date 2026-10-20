# Prepared ingress acceptance runner

Prepared only. Candidate pins remain unset until parent supplies the completed frozen build. No transfer, browser or timing authorization implied.

Primary comparison remains original e148 control plus pinned PeleJS. Exact warm1429f773 and first-use5469078 harnesses/cases copied unchanged. Each separately authorized phase retains all24 conditions,21 paired trials,1000ms warmup and all8+16+12 serial activation checks. First-use includes parse/edit setup/render/disposal. All probes close before timing; package hashes checked between probes and after timing.

Pin only the supplied frozen candidate:

```sh
python3 dist/perf-patches/parse-architecture/source-prepared-ingress-runner/pin-candidate.py --manifest <parent-manifest> --package <frozen-package> --source <parent-commit>
```

The pinning command validates every packed file and rejects handwritten/static differences from ef9: this assigned candidate is native-only. Then run the existing `files/scripts/perf/run-source-layout.mjs` with `--preflight-only`, explicit package paths, both WASM hashes, candidate source/manifest and output directory. Local preflight launches no browser. VM execution requires a separate parent GO with `--go --phase warm` or `--go --phase first-use`.

Immutable `references.json` retains complete e148113-file and ef9116-file identities plus previous ef9 raw24-row report hashes. Never derive isolated improvement by subtracting results from separate windows.

Minimal supplemental comparison, only if separately authorized: one additional unchanged first-use5469078 run with ef9 as control, new candidate and the same JS lane, all24 rows retained. This isolates the acceptance batch without replacing the primary e148 lane. Existing activation probes still run against original e148/new candidate (they deliberately assert old control lacks the source capability); additionally hash-check ef9 immediately before/after the supplemental timed harness. Do not relabel ef9 as e148 inside probe expectations. No second warm run needed to answer the edited-workflow question. Supplemental runner/GO can be added after primary results warrant it.

Execution priority: **first-use first**, warm only after parent reviews first-use and authorizes it. Current source acceptance revision is e518012745a2eaf12c8bb048b91074566bf2000a; this is context only, not an installed candidate pin. Build manifest remains required.

`vm-artifacts.py` prepares a Python-tarfile archive of the116-file candidate, parent package manifest and exact runner sources. It excludes AppleDouble metadata, validates every frozen input, and refuses remote overwrites with different bytes. Remote root remains `/home/admin/pele-perf.jL9wzP`. It never launches a browser/harness/build.

After candidate pinning and separate parent GO:

```sh
python3 dist/perf-patches/parse-architecture/source-prepared-ingress-runner/vm-artifacts.py pack
python3 dist/perf-patches/parse-architecture/source-prepared-ingress-runner/vm-artifacts.py transfer --parent-go
```

After the authorized first-use run completes, fetch with `vm-artifacts.py fetch --phase first-use --parent-go`. This preserves raw evidence and actual executed scripts under `runtime/fetch-first-use/`, plus independent file hashes. Later warm evidence uses `runtime/fetch-warm/`. Fetching is not semantic verification: require runner exit0, all three probes, full24 rows and before/after package equality before reporting success.
