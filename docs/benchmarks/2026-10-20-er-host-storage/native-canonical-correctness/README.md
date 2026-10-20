# Native canonical core: fresh Swift and C correctness

Commit `3166b3dd18569ad0bc20f173a8941dd628609ddc` passed a fresh macOS ARM64 SDK build and the existing default-feature consumers: **16 Swift integrations, 12,481 mutable ER/Sankey drawings, 16,572 expected errors, zero explicit budget differences; 168 Swift API conditions across 21 cases; three strict C consumers**. These are correctness results, not timings or proof of the opt-in native-canonical browser backend.

The SDK and standalone consumer were rebuilt from the committed source. All 362 enumerated source/consumer/SDK inputs remained unchanged from the completed SDK build through correctness. Before building, 349 source/consumer inputs and 34 additional correctness inputs were matched to immutable Git blobs. Only the host SDK archive changed during the SDK build. Both driver files, Swift test fixtures, the bindings package manifest and C consumer sources were additionally bound before/after execution. The previous `25d87cb` consumer was preserved separately.

| Artifact | SHA-256 |
| --- | --- |
| Final source/consumer/SDK aggregate | `1f89890b534bf1fe2958ed9b03bbc91470cae260a615925e1b15df752cb157ee` |
| Fresh host SDK archive | `3e89155e3283ef2d77abf2b707e4a6831088f6339ddddf8dab17d5dd8b16c1dd` |
| Fresh standalone Swift consumer | `c9fb8ab770594cf4395d0f035f8f7cc0f56cf98641a6a5c74b2ee623ec66082a` |

`proof/` contains the four execution freezes, immutable commit mapping, build manifest, exact API outputs, command exit records and C consumer records. `logs/` retains every command log, compressed without changing its contents. `execution/` archives the exact driver and consumer build/run scripts. The Swift corpus covers the existing eager projection, prepared-document, mutable ER/Sankey and nonfinite Sankey feature slice. C consumers cover ownership/errors, mutable public models and parse-limit precedence with warnings treated as errors.

Run the portable data-only verifier from this folder:

```bash
node verify.mjs
```

It recomputes aggregate hashes, all 168 unique case/stage output checks, corpus totals, command outcomes, driver bindings, SDK/consumer record consistency, immutable-source mapping consistency and the complete evidence index. It needs only Node and these files; it does not access Git, an SDK, a binary or a browser. The archived execution scripts are historical reproducer inputs with their original repository-relative imports, not portable benchmark entrypoints.

Binaries and full source trees are deliberately omitted. Their stored hashes establish captured provenance; this verifier is neither a reproducible build nor a fresh physical binary rehash. The native SDK uses default features, so this proof must remain separate from the selected WASM owner/browser suite. No local latency, Intel SDK, release-readiness or complete JavaScript parity claim is made. Build/test durations in logs are operational metadata, not performance measurements.
