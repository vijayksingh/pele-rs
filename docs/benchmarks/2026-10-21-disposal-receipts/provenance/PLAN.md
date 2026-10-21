# Disposal receipt teardown: correctness evidence

Prepared only. No command below has run as part of preparation. Parent owns source/package/test execution after the candidate is committed.

1. `node dist/perf-patches/parse-architecture/disposal-receipt-runtime/freeze.mjs source`
2. Freeze the actual candidate package using the existing parent packaging flow, then `node dist/perf-patches/parse-architecture/disposal-receipt-runtime/freeze.mjs package`.
3. Preserve the Chromium RED retained-leaf GC report and the subsequent focused candidate report. Parsed/rendered/helper histories must each show terminal root collection while the retained leaf remains usable as an inert object. Retain actual CDP evidence and observer-lifetime safeguards; no inferred allocation/RSS claim.
4. Run the parent's chosen public browser/Node/type checks, preserving raw outputs and any failures. Existing semantic disposal/borrowed-helper tests remain separate from retained-root collection evidence.
5. `node dist/perf-patches/parse-architecture/disposal-receipt-runtime/verify-native-reuse.mjs before`
6. `node scripts/perf/swift/run.mjs --output dist/evidence/disposal-receipt-swift-api.json > dist/evidence/disposal-receipt-swift-api.log 2>&1`
7. `node dist/perf-patches/parse-architecture/disposal-receipt-runtime/verify-native-reuse.mjs after`
8. `node dist/perf-patches/parse-architecture/disposal-receipt-runtime/freeze.mjs after`

The native helper checks all 370 actual native/SDK inputs, the current consumer manifest and binary, the SDK archive and frozen11f consumer, then binds the fresh API168/21 report. No SDK/native/corpus/C rebuild or rerun is included. A mismatch is a hard stop, not permission to relabel old proof. Preserve HEAD/source/package during the checks. Final correctness.json is written only from returned completed results.

Source-symbols profile wording correction is documentation only: five-second target, not exact duration. Large candidate operation elapsed7580.785000801086ms; raw CDP span7876194us. Original profile/RESULTS and raw bytes stay unchanged.
