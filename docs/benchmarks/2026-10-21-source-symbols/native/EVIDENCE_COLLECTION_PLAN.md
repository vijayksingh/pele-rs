# Fresh source-symbols correctness collection

Scope: source 11f27516d98c3d89799a7fe2065eec0bd4e147cf. Read-only collection after the existing driver finishes; no additional builds, tests, timing, package generation or retries.

1. Require actual `runtime/after-correctness.json` and a successful final driver command. Keep every command/log, including any failure; an absent final record is incomplete proof.
2. Compare before/after driver hashes, all source and correctness-fixture hashes, immutable Git mapping, and unchanged preexisting web-package map. Only the expected SDK archive input may differ across SDK regeneration. This old package is a continuity control, not a fresh 11f WASM proof.
3. Read the 11 native test-binary summaries (57 feature tests, including the full 11,616-source Sankey compatibility corpus), Swift 16 summary and 12,481 drawings / 16,572 expected errors / zero explicit budget differences, all three strict C compile/run pairs, and API 168 conditions / 21 cases. Assert actual report counts; do not infer success from compilation.
4. Bind generated SDK archive, consumer binary and build manifest to the final source-input map. Verify frozen candidate-swift files, prior control consumer/manifest and the relocated SDK tree. Relocated backup verification is already recorded in `runtime/prior-sdk-verification.json`.
5. Write a private evidence summary and manifest only, referencing exact raw records/log hashes. Separate source, SDK, native integration, Swift/C/API and later WASM/browser proof. No public documentation, Git index operations, performance claims or release claim.

Status: SDK build still running at preparation. No final correctness result claimed.
