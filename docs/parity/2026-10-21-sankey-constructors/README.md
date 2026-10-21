# Captured Sankey constructors

[Source `2be5491`](https://github.com/vijayksingh/pele-rs/commit/2be54917938f880d09d2b34cafe1d32bb3633e8f) implements captured Sankey construction through Rust, C v16, Swift and the browser facade. Factories retain their registration across replacement or registry closure. Each construction creates independent native defaults without parsing a seed. Helpers, snapshots and raw rendering use the existing model contracts. Swift exposes constructor provenance explicitly; its low-level normal render remains a native API extension, while browser core render rejects an unparsed constructed model as Pele does.

## Verified scope

- Swift: 31 existing regression features passed in the consolidated run. The final targeted run passed both constructor features, covering normal and prepared/raw models, ordered defaults and populated graphs, helper aliases, exact SVG/dimension bits, independent siblings, foreign-provider rejection and factory/registry lifetime.
- Strict C: the new v16 constructor lifecycle client and the existing v15 callback client passed against the fresh SDK.
- Browser: two constructor features passed in Chromium, Firefox and WebKit (six tests). The five behavioral modes in each engine compare actual Pele against the candidate, including receiver/prototype behavior, ignored arguments, aliases, callbacks and raw rendering. A separate scoped regression run passed 73 tests with two explicit skips. TypeScript checking passed.
- Native Rust: the implementing agent reported both `public_sankey_constructor` integration tests passed. No standalone raw log was retained; `native-reported.json` records that limitation and the exact command. This result is not reconstructed from SDK build success.

## Failed attempts retained

The old browser package failed both constructor tests because its constructor stubs were unavailable. This is the behavioral RED evidence.

The initial Swift compilation failed because the test used `XCTAssertEqual` on an Atom type without `Equatable`. After correcting that observer, the 33-test run had 32 passing tests and one constructor test with six assertion failures: its graph observer omitted intrinsic array `length`. The observer was corrected, then both constructor tests passed. These were test-observer defects, not runtime RED failures or runtime fixes.

The first new-package browser run passed five tests and failed one Firefox assertion because the test hardcoded function own-key order. Candidate and both live oracle variants already agreed on Firefox's order. The final assertion compares the live same-engine oracle order; all six passed. Runtime, SDK and packed package stayed unchanged through these test-only repairs. Every original failure log and the intermediate browser report remain in `raw/`.

## Identity and limits

The 468 native build inputs and 547 web build inputs match their respective before/after records and immutable `2be5491` Git blobs. Original records retain the earlier checkout revision captured before the uncommitted source was committed; that revision is not substituted for the final runtime pin. SDK and 117-file package hashes are retained in the original after-build records. Final test hashes are separately bound in `source-binding.json`.

Constructor metadata is not completely equivalent: the candidate function name is `SankeyDb`, versus the pinned monolith's `Xc` and ESM's `B`; Chromium's no-`new` error text also reflects different names. Raw metadata is retained, not normalized away. Unsupported constructors remain explicit; this Sankey feature does not implement every family constructor, all JavaScript class semantics, full API parity or release readiness. No performance claim is made.

## Portable verification

```sh
python3 docs/parity/2026-10-21-sankey-constructors/verify.py
```

This checks bundled hashes, lossless gzip bytes, source-map continuity, final behavioral report equality and logged counts. It does not run clients or infer tests from builds. `raw-manifest.json` records original paths, original/stored hashes and sizes. `SHA256SUMS.json` lists every bundle file except itself. Native binaries and SDK archives are not duplicated.
