# Captured native core correctness

Source `25d87cb49bb451d3e7c1e70c2856ccb6fef4abd0` passed 61 feature-enabled native integrations across 15 targets and 29 default-feature integrations across six targets. Its freshly built arm64 host SDK passed 16 selected Swift integrations, the ER/Sankey mutable corpus (12,481 drawings, 16,572 expected errors, zero explicit budget differences), three strict C consumers, and 168 API correctness conditions across 21 cases. These are correctness checks, not latency samples.

The later combined-dispatch source `eb72419e237cdb11706096122fe65086a3416c15` passed 22 native integrations across four targets, including three new source-routing cases. This is a separate native checkpoint. The SDK/Swift proof below remains pinned to `25d87cb`; it does not claim a newer SDK or browser captured-owner validation. Counts overlap between runs and must not be summed as unique tests.

## Freshness and scope

The before-SDK record maps all 344 source/consumer inputs to immutable `25d87cb` Git blobs. Rebuilding the host SDK changed only `CPele/macos-arm64/libpele_ffi.a` among the 357 enumerated source/SDK inputs. The consumer build independently checked that the SDK archive equals the current Cargo target archive. All 357 inputs and the consumer binary then remained unchanged through the correctness run.

- Source/SDK aggregate: `0bffad4be2e139643262e136f1663487ac98c0a5f8cdbbe3edecd81d88132b0d`.
- Native archive: `869d20000dfd58de5f4cd2a3805a24df9731462b13151b29c25ac9db96b489ca`.
- Swift consumer: `9c1d267539a7eaa1b94243dc64cc1dabfafbd0c795a73dd265d5b3f843d56888`.

The API report records unrelated, uncommitted browser/WASM work. Those paths are outside the SDK input set; the native input map was checked against the pinned commit before the run. No current working-tree snapshot is substituted for this evidence. The feature-enabled native checks exercise captured storage through a Rust integration consumer; the default SDK checks exercise its existing Swift/C delivery path. Neither proves the optional captured WASM facade.

`proof/` contains the original before/after source/SDK records, consumer manifest, full 168-row API report, exit-code/command records and immutable commit mappings. `logs/` contains deterministic gzip copies of the complete native, SDK, Swift and C logs. `execution/` preserves the executed scripts for provenance; they are repository-dependent and are not the portable verification command. No binaries or packages are copied.

## Portable verification

Run with Node from any directory:

```sh
node path/to/captured-native-core/verify.mjs
```

The verifier reads bundled data only. It recomputes the input aggregates, exact API output comparisons, test totals, SDK/consumer hash bindings, immutable-map/manifest consistency, command exits and every indexed file hash. It does not build, execute consumers, access an SDK, or re-create omitted binaries. Stored physical-file hashes are historical proof captured during the original run, not a new rebuild claim. `summary.json` is derived by this verifier; `evidence-index.json` covers every other bundled file.

No timing result, browser result, Intel SDK validation, all-family performance win, full JS parity, or release readiness is claimed here. Existing release/performance gates remain separate.
