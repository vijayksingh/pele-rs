# Ordered constructor checkpoint — 2026-10-21

All 22 class-root families and nested ClassMember now have scoped constructor
proof through Rust, Swift and the packed browser library. This completes the
factory inventory; it does not certify full Pele runtime/API parity.

- Native: 24 public feature integrations, including ordered getters, virtual
  member dispatch, escaped partial receivers, live options, Block warnings,
  helpers, raw SVG, shared Flow/Swimlane ownership and existing Block regressions.
- Swift: eight constructor features cover all 22 roots and ClassMember; one
  additional Block feature compares four pinned callback scenarios. A separate
  regression checks 99 cases across all 33 raw entries.
- C: strict C11 C17/C18 clients compile and run against the fresh ARM64 SDK.
- Web: 51 feature/regression checks pass across Chromium, Firefox and WebKit.
  New body comparisons are exact; previous explicitly named semantic checks
  retain their recorded browser-generated error-message exceptions.

Native runtime/build inputs (366 paths) match commit `9746e75`; all 13 generated
SDK files stay unchanged through execution. The later test-only `48a8d6f` adds
Block coverage and corrects the C18 fixture. Web runtime/package inputs (442
paths) match `b2222224`; the 118-file package contains WASM SHA-256
`39801cab676c1ed541b09d1bc0e27047e34f3613c2de3993f2d71fed6e4cffc8`.
The original build and final package differ only in the two documented adapter
files. No second WASM build was required for that correction.

## Failures retained

The first browser body run exposed Block's warning field masked before the
constructor assigned it. The generic adapter now binds receiver masks after
completion. Original failed report and successful rerun are included.

The first C18 client requested a constructor from plain-object Pie. The corrected
client uses authentic ignored-argument Sankey. This was a test error; no runtime
or SDK change occurred. The first Swift Block test failed to compile due to two
missing `try` keywords; its corrected feature passes. Failed logs/source remain.

The previous 45-check constructor run had 37 passes and eight failures: five
browser diagnostic wording differences, plus three older Sankey checks involving
a stale Flow unsupported assertion or known Firefox intrinsic key order. Its log
and recorded message differences are preserved. The final 51-check batch excludes
the known strict diagnostic tests by name; it is not a passing claim for those
exact diagnostics. Raw reports retain both candidate and oracle observations.

## Remaining work

The resource-callback mutation audit still has unresolved cases in 31 families.
Active malformed UTF-16 drawing, sparse/coercion domains, advanced overrides,
constructor reflection and cross-registry identity remain separate gaps.
Configured Agentflow graph-build diagnostic callbacks and Gantt scheduler
warnings are not covered by the explicit helper continuations. Provisional
constructor jobs expose ordinary helpers, not nested detached rendering.

The native SDK is ARM64 host-only. No physical Intel/macOS-14, VM performance,
release or publication claim follows. Version 0.3 remains unreleased.

`manifest.json` binds compressed and original bytes; `source-binding.json` names
exact source checkpoints. Reports are compressed losslessly, without duplicate
Playwright attachments or generated SDK/WASM binaries. Verify offline integrity
and recorded assertions, optionally checking source hashes against Git:

```sh
python3 docs/parity/2026-10-21-ordered-constructors/verify.py
python3 docs/parity/2026-10-21-ordered-constructors/verify.py --git
```

The verifier does not rerun browsers, Rust, C or Swift.
