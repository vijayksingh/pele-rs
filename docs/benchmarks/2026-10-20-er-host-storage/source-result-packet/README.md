# Source-result packet evidence

The successful source-render result crosses WASM as SVG, dimension bits and a small links payload. It no longer builds/transfers/parses one complete result JSON string. The same canonical serde traversal still counts the exact v2 envelope and enforces its 64 MiB limit. Old exports, failure strings, callback priority and missing-capability fallback remain intact. This is the source-result boundary across all 33 families; parsing/layout remain Rust.

Current package was built from the frozen 657-file source map at base `6dfa9ca` plus patch `54bb4387...`. Every captured input matches immutable candidate commit `640deee`. The control is the accepted dense runtime `7701135`. The full browser suite passed **405/405 in 13.1 minutes**, after the focused checks; see [status.json](status.json).

## Measured scope

One independent VM diagnostic completed all 126 predefined conditions: seven families, three sizes, fixed and Canvas measurements, source render plus parse and retained-render controls. Each row has nine three-way trials, 500 ms warmup per runtime and matched control/candidate batches capped at 128. Complete SVG and dimensions were checked before and after. This is not the full JS performance release gate.

All 42 source rows had lower paired medians: -0.89% to -20.78%, median across rows -13.89%; candidate won 6 to 9 of 9 pairs per row. Candidate source medians were still 1.35x to 4.63x pinned JS. Family medians of the six row changes were:

| Family | Median source change |
|---|---:|
| Flowchart | -9.09% |
| Sequence | -15.32% |
| ER | -14.05% |
| Pie | -16.80% |
| Sankey | -18.54% |
| Gantt | -7.29% |
| Railroad | -14.58% |

Controls remain mixed. Parse: 21/42 rows lower, median -0.04%, range -7.08% to +6.83%. Retained render: 25/42 lower, median -0.30%, range -9.49% to +3.85%. Every row and all raw samples are retained in [comparison.json.gz](comparison.json.gz); [RESULTS.md](RESULTS.md) shows every adverse control. No control subtraction, noise dismissal or regression-free claim.

Paired change is `median(candidate[i] / control[i]) - 1`, not the ratio of the standalone medians. Stage/family summaries are medians of row changes, not pooled trials. Three-way positions balance 3/3/3; control precedes candidate in five of nine trials per row. The shared browser/page heap, GC and caches differ from release-gate scheduling. No broad causal or memory claim follows from this one diagnostic.

## Correctness and identity

- Full Chromium/Firefox/WebKit suite: **405/405 passed** on the same source/package, with its complete log retained.
- Focused Chromium/Firefox/WebKit checks: **33/33 passed**, including all-family numeric transport, foreign-provider controls, new source packet/fallback/limit/throw cases and existing retained ER packet controls. The 24 explicit source/ER reports retain full results and exact envelope counts, without duplicate attachment copies.
- Fresh local Swift: **15/15 integration cases**, ER corpus 7,535 drawings and 9,902 errors, zero explicit budget differences; **168/168 API conditions** across 21 workloads.
- All 351 native/SDK/consumer inputs matched before/after at `439fe2e6...`. Swift binary `554ef37b...` and native SDK/target archive `d87e97d3...` were reused unchanged. No native build or Swift timing was performed for this layer. Three strict C consumers are prior identical-native validation from dense `7701135`, explicitly not rerun.
- Default size-optimized WASM build and TypeScript checks passed. Package maps contain 106 files per side. Exactly five files changed: facade, generated JS/types, WASM and WASM types.
- An independent fresh Node reader hashed both actual VM directories after timing; all 106 hashes per side match the benchmark and local frozen-package maps. This proves file identity at recheck, not an independent rerun or reproducible build. The frozen source/build log binds the built package; no rebuild was performed solely to verify reproducibility.

WASM size: 8,149,880 to 8,153,968 bytes, **+4,088 bytes (+0.0502%)**. Gzip level9: +1,266 bytes (+0.0510%). Brotli quality11 generic: +617 bytes (+0.0360%). Exact sizes, hashes and compressor versions are in [package-size.json](package-size.json); these are file/transfer diagnostics, not peak live memory or latency.

## Reproduce the evidence summary

```sh
python3 summarize.py
```

The portable ASCII script reads only bundled data. It verifies raw sample arithmetic, all row identities, exact hashes, package/source links, Swift reuse, explicit E2E equality and any completed full-suite/commit proof before regenerating `summary.json`, `RESULTS.md` and `evidence-index.json`. It does not run repository code, builds, browsers, consumers, benchmarks or network requests.

[VALIDATION_PLAN.md](VALIDATION_PLAN.md) records exact parent build/test/VM commands and their scope. `harness/` preserves the executed comparison/case scripts. `design/` preserves the original reviewed candidate and diagnostic profile attribution; statements there about unexecuted tests describe its pre-application stage. Eighteen later candidate CPU profiles (Sankey/Sequence/Railroad medium; source/parse/retained; JS/WASM) are retained compressed, with the original summary and frozen profiler. They diagnose self-time, not additional paired latency evidence; they are not an isolated before/after profile comparison. No custom SVG counter is included. No new 0.3 or JS parity claim.
