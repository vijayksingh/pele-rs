# Native canonical backend: frozen browser validation

The frozen backend bound to commit `3166b3dd18569ad0bc20f173a8941dd628609ddc` completed **468/468 browser tests: 156 each in Chromium, Firefox and WebKit, zero failures or skips**. The single-worker run reported 13.4 minutes. Each engine includes six native-memory lifecycle cases and three captured-Sankey feature tests, alongside the existing suite. Tests explicitly opt into the experimental owner where required; this result does not change default backend selection or establish performance/release readiness.

Native integration logs contain an initial 26 feature-enabled checks and a six-check follow-up. Five repeat, giving **27 distinct target/test identities**, including the complete **11,616-source** native flat-construction corpus. A separate default-feature run passed **11 checks**. Default and feature names also overlap; these counts must not be added into a claim of unique tests.

The original source and package freezes predate the final source commit. Their **586 source hashes** are identical to the final after-browser record and were independently matched to immutable Git blobs at `3166b3d`. All **113 frozen package files** match both package and after-browser records; the preserved `native-canonical-runtime/candidate-package` was independently rehashed during packaging. Current working sources and the later cursor experiment were not used as the baseline.

Frozen WASM SHA-256:

`2310934dd43dc52c20723c3e0217377f9819fef8e146d03ef3a33845e06bdcce`

`logs/` preserves the full browser output, all three native windows, WASM check/build output and original runtime-proof output. `proof/` preserves source/package/before-and-after hashes, immutable Git mapping and the independent frozen-package rehash. `scope/` contains the three relevant public integration/E2E sources from the immutable commit. `execution/freeze.mjs` archives the freezing mechanism. The separate [native/Swift/C correctness bundle](../native-canonical-correctness/README.md) records the fresh SDK and 16 Swift/C3/API168 proof for this source commit.

Run the portable data-only verifier from this folder:

```bash
node verify.mjs
```

It recounts every browser row, requires the same 156 tests in each engine, checks all row ordinals and feature-test counts, recomputes native window totals and their union, and verifies all source/package/hash-record and evidence-index bindings. It requires only Node and this folder. Full source trees, package binaries and browser attachments are omitted; the verifier checks captured evidence, not a fresh browser run, reproducible build or live package rehash.

Passing correctness does not supersede separate performance evidence. No latency improvement, complete JavaScript parity, release or adoption claim is made here. Reported build/test durations are operational metadata only.
