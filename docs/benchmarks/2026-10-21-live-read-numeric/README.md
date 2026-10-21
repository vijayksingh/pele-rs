# Numeric live-read ABI: performance remains rejected

Source `7417fed53e34f482f726dd07a12fec4dd32b9425` replaces repeated string command/context operands with numeric operation IDs and job-owned property-key handles. Rust still selects read phases and owns layout/SVG. The change retains corrected callback mutation, nested helper/render and original-throw semantics. It does not establish JS performance parity or authorize a 0.3 release.

## Matched VM diagnostic

The original Rust control is `e1486055d596d4e0ac25e4c78f1d5d95a73f781a`; the third lane is pinned Pele 1.6.0. The run includes Sankey small/medium/large and Flowchart medium, fixed/Canvas measurers, parse/retained/source stages: **24 rows, 21 balanced paired trials per row, 1000 ms warmup**. Matched Rust batch counts, complete sample arrays, before/after output checks and all adverse rows are retained. The 36 activation/continuity/ingress probes run outside timers.

| Sankey stage | Paired change versus e148 | Candidate / Pele JS |
| --- | ---: | ---: |
| Parse, six conditions | -39.86% to -6.91% | 6.04-14.19x |
| Warm retained render, six conditions | +86.72% to +125.23% | 3.42-6.26x |
| Source render, six controls | +0.15% to +2.37% | 1.02-1.59x |

Every retained condition has **0/21 lower candidate trials**. All six Sankey source controls are adverse. Flowchart parse is +0.73%/+7.19%; retained is -0.17%/+3.40%; source is -0.44%/-0.13%. Across all rows, 15 paired medians and 17 ratios of lane medians are adverse. The [complete results](RESULTS.md) preserve both metrics and all absolute times; these are not dismissed as noise.

The preceding [8175006 run](../2026-10-21-sankey-live-render/) is preserved separately. These independent windows do not measure an isolated numeric-ABI gain versus 817, and their changes must not be compounded. The e148 control also lacks the corrected mutating-callback behavior. No first-use run or new complete 126-condition release gate was performed. The historical complete gate remains 126/126 failing.

## Correctness and provenance

Fresh numeric-ABI checks pass **Node 122**, **60 browser tests across Chromium/Firefox/WebKit** (including 366 callback comparisons), type/generator checks, and **API 168 across 21 cases**. Raw Node/browser reports and exact logs are included. Generic Error stack coordinates remain diagnostic-only; semantic messages, original arbitrary thrown values, object graphs and PeleError source diagnostics remain strict. Exotic coercion/Proxy and lone-surrogate behavior remain outside the demonstrated scope.

The native core and SDK inputs are unchanged from `8175006`: 370 inputs, SDK `4ecfbbfb33ce6a0fba349ed6e9340074ba0fce37d28d2e35603b6e6a027327bd`, consumer `a5f4427f7b8005b949f9d2c49053692ed5b0ef13421c5fda60e44b20ada218e3`. Its native 57, fresh Swift 16 (12,481 drawings / 16,572 expected errors / zero explicit budget differences) and C 3 proof is reused, **not presented as a fresh 7417 SDK/corpus run**. API 168 was rechecked using that identical consumer.

Source manifests bind immutable 7417 Git inputs. Local package before/after and executed VM candidate maps agree on 117 files; control has 113 files. The exact executed harness SHA is `1429f77380082b13424f8942824a86437dccf988c9200d228fc487b2437f8272`, cases SHA `b16a28135adb4206a10af40f12b3a1508c2648e6dc1be3b1a4cff5a62f537a58`, raw report SHA `0f55ccaeacef892094c9993d2185ef08699dfb267cfef0da5cb3ca5cdb8ee53c`. These recorded hash maps prove the measured package identities; this bundle contains no binary or independent rebuild claim. Runner preparation metadata is archived verbatim, including its pre-execution status; completed execution and verification records establish what actually ran.

## Retained CPU profiles

Four serial five-second windows compare medium/large fixed retained renders against e148; parsing occurs outside the windows. Full SVG, dimensions and ordered callback traces match before/after. The [profile report](profile/RESULTS.md), raw profiles and exact executed harness are retained.

Candidate `prepareMutableInput` occupies 39.41%/39.05% inclusive. Snapshot (17.58%/18.80%), sparse encoding (7.87%/7.13%) and native apply (12.82%/12.33%) overlap within that preparation path; these shares must not be added. The live-reader ancestor union is 12.83%/12.98%. Live-phase text-decode leaf samples are 0.48%/0.21%; total text-decode leaves are 5.07%/4.53%, including result and resource text. Sampling and inlining limit attribution; these percentages establish neither an isolated latency gain nor a removable-cost ceiling.

The inherited automatic parser category falsely matches `IndexedSankey` type names in render symbols. No parsing occurs inside these retained windows. The corrected weighted attribution is recomputed from all raw samples. One local analyzer attempt read an incomplete transfer and failed; analysis succeeded after the same transfer completed, without another profile run.

## Portable verification

```sh
python3 docs/benchmarks/2026-10-21-live-read-numeric/verify.py
python3 docs/benchmarks/2026-10-21-live-read-numeric/verify-profile.py
```

The verifier reads data only: it verifies every stored/original SHA256, decompresses exact original bytes, recomputes all paired statistics, checks package/harness bindings, probe counts and correctness records. Gzip output is deterministic (`mtime=0`). [Artifact manifest](artifact-manifest.json) records original paths, original/stored sizes and hashes. No browser, build or benchmark is launched.
