# Native Sankey callback reentry

Source commit [`7b25d99`](https://github.com/vijayksingh/pele-rs/commit/7b25d99335d638fd6f5a0963d046413485d1dd24) adds owned C v15 render jobs and Swift normal/prepared rendering through Rust's detached lifecycle. Document locks and host-binding borrows end before resource callbacks. Nested helpers, edits and rendering retain their state; finish acknowledges the latest revision before returning an original callback error. Close remains forbidden while a detached render is active.

The old Swift implementation failed all three initial public callback tests. With the freshly built SDK, 22 Swift integration tests passed, followed by nine additional distinct tests: **31 tests total**. Five new callback features each exercise normal and prepared/raw rendering (ten lane cases): helper append, mutation followed by an original throw and recovery, nested render with close rejection, caught partial helper failure and repair, and a caught inner resource error that must not poison the outer render. Fixtures come from actual pinned Pele; comparisons include SVG, dimensions, measurement order and retained state. Existing host-function, mutable-document and prepared-document tests provide scoped regression coverage.

The new strict C lifecycle client passed, including callback snapshot/helper/nested render, revision acknowledgement, repeated job-use errors, cancellation and unsupported-family fallback. Five existing strict C clients also compiled and passed. The fresh Swift consumer ran in **correctness-only** mode: 168 comparisons across 21 cases. Timing-related settings present in that tool's generic report are not measurements or performance evidence.

## Source and SDK binding

Original before/after SDK records are preserved unchanged. Their 515 input hashes match exactly; their recorded HEAD predates the uncommitted implementation and is **not** a runtime source pin. `source-binding.json` binds the final paths to commit `7b25d99`: 514 input files remain identical, while `NativeSankeyCallbackTests.swift` was extended after the SDK build. The second fixture and its oracle generator were added afterward and are recorded separately. These changes do not imply a second native build. The API consumer's recorded build revision is likewise retained as historical metadata; exact input and artifact hashes carry the binding.

- Native SDK archive SHA-256: `697781a9aec1bde887ab2944a48deaf9876036a157c4ad2d1fe97aabfdfb361c`.
- Swift consumer SHA-256: `6d1a19a5436830f634fec457692e75c559dfcfc7b19d2f8b441c473aa6b2cd6a`.

The separate browser baseline completed **469 passed, 59 skipped, zero failed** against the unchanged 117-file package from `d7caf8c`. Native source was edited during that run; this is package-level browser evidence, not a whole-worktree freeze or browser verification of the new C/Swift adapter. The original mid-run identity record and completion record explicitly retain that distinction.

## Bounds of this proof

Native Ref IDs remain graph coordinates. Arbitrary table replacement, truncation or ID reuse is not proven equivalent to JavaScript retained-object lifetime. No new apply restriction conceals that boundary. Object coercion, icon-specific native reentry, native presentation/image routes and the broader matrix in the [LLD](../../../LLD_NATIVE_SANKEY_CALLBACK_REENTRY.md) remain separate work. This evidence does not establish full API parity, release readiness, SwiftUI behavior or performance.

## Recheck the evidence

Run from any checkout or copied bundle:

```sh
python3 docs/parity/2026-10-21-native-sankey-callbacks/verify.py
```

The verifier uses only bundled data and Python's standard library. It checks stored and decompressed hashes, build-source equality, the explicit test delta, test counts, API case/stage uniqueness and fixed browser-package identity. It does not rebuild or rerun clients. `raw-manifest.json` maps each compressed artifact to its original path, byte count and SHA-256; gzip timestamps are zero. `SHA256SUMS.json` binds every bundle file except itself. Raw logs retain the original RED failures. Binaries and SDK archives are identified by hash rather than duplicated here.
