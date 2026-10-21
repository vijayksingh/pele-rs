# Independent successful-result packet review

Initial review covered private patch `97e0a4f63440e2daf984c54be6c912c981a9d8267c3cba664b657db089df1458`. The later applied patch is `edd4c8ef5bfb42a47dde0a52fccf891e5bda09e154af6719527acc6333c7cb68`; all four runtime files remain byte-identical and independently match production. Parent subsequently corrected only the public test fixtures. This follow-up reviews the current production test file, identified separately by SHA in `review-manifest.json`; it does not pretend the applied patch contains those later test corrections. The initial review manifest is preserved as `review-manifest-97e0a4f.json`. No production changes, compilation, test/browser execution or timing by this reviewer.

**Disposition:** no concrete runtime source blocker. Ready for parent compilation and focused public validation. This is not proof of runtime equivalence or performance improvement.

## Exact serialization and admission

`render_host` and `render_raw_host` retain their String-returning ABI. Their extracted private bodies reproduce the original numeric/config decoding, HostStore creation, validation, rendering and `finish` callback. Packet exports call those same bodies; no alternate renderer, callback replay or read cache is introduced.

The counted value is exactly the old `result.as_ref().map(render_result_transport)` for normal output and `result.as_ref().map(prepared_entries::transport)` for raw output. Consequently the count includes `Result::Ok`, the correct normal/raw key shape, escaped SVG and links, and the `$numbers` sidecar even though the successful packet does not expose it. `Counted` uses the same `serde_json::to_writer` and per-write `bytes.len() > JSON_BYTES - current` rejection as `Bounded`. The counter never exceeds 64 MiB, so its subtraction/addition cannot overflow. Serializer errors retain the same public budget error. No approximate JSON size formula replaces serialization.

Only after full-envelope counting succeeds does the code encode links and construct the JS packet. Link JSON is bounded by the same serializer; its bytes are a subset of a successful full envelope. The successful path no longer retains the full escaped SVG JSON buffer or reencodes that envelope with TextEncoder. Errors still use the existing JSON path. Standalone link serialization retains generic LinkInfo field order; ER's current empty links provide no reachable nonempty-link integration proof.

The final structural callback remains before serialization/counting. `owner.finish` still runs afterward with the complete virtual metadata length. A count/transport failure has no successful metadata packet, as before. Captured original resource exceptions still take priority over synchronization/transport failures after cleanup. Method results remain JSON, and packet input is explicitly rejected for a method operation.

## Public result and floating point

Normal key order remains `type, svg, width, height, links`; raw omits `type`. Object literals and parsed links have the same plain prototype and ordinary enumerable/writable/configurable own data properties as JSON.parse. No internal packet tag/count reaches the returned public object. Typed Rust strings preserve embedded NUL, Unicode and BOM through the existing wasm-bindgen UTF8 reader (`ignoreBOM: true`). SVG still requires its real UTF8-to-JS conversion.

Four u32 halves reconstruct IEEE dimension bits through DataView. Both properties begin as `null` and are then assigned numbers, matching the old sidecar restoration shape and avoiding temporary numeric arrays. Nonfinite values and negative zero retain the bits that were formerly carried in `$numbers`; finite serialization uses the workspace's enabled serde_json `float_roundtrip`, so direct transport does not bypass an intended finite-number normalization. Shared DataView writes have no user callback between writing and reading; object properties already exist, so ordinary inherited setters do not intercept assignment. Arbitrarily changed intrinsics are outside the existing contract.

Old-WASM feature fallback selects the unchanged JSON endpoint when the optional packet method is absent. Shared body ownership and operation-local results preserve nested render independence. The packet adds no document ownership, model references, disposal semantics or persistent graph.

## Test review finding resolved

The original NaN options case only compared the two outcomes; two matching early failures could pass. Requested a mandatory success assertion, observed successful packet, and an actually NaN returned dimension. The final test now requires all three for every lane and compares exact computed output bits against the legacy endpoint. It does not incorrectly require the renderer's arithmetic to preserve the input NaN payload unchanged. The runtime files did not change for this correction.

The tests cover normal/prepared/raw/caller-root lanes, exact metadata count equality and public key order, Unicode/escapes, output-limit forms, nested renders, arbitrary thrown values, final-audit versus output-limit precedence, and post-error repair. The final accessibility-string fixture calibrates exact serialized growth using one and two quotes. It then uses a single bounded string of at most 12 MiB characters. Normal rendering targets an envelope just over 64 MiB; raw rendering targets a successful native envelope just below 64 MiB which the final graph-plus-metadata gate must reject. Both are comparatively large correctness fixtures and belong outside timing windows.

## Remaining proof obligations

- Compile generated WASM and run these public tests in Chromium, Firefox and WebKit. In particular, establish successful NaN/infinite dimension outputs and exact computed bits in each engine; source inspection cannot establish JIT boxing behavior.
- Run the existing live callback and ownership slices; the new packet changes response handling even though the renderer body is shared.
- The current raw boundary fixture requires a recorded native success, exact old/new byte-count equality, metadata within 16 bytes below 64 MiB, then the specific final graph-plus-metadata Limit error. This now covers the previously missing combined gate in the test design. Cross-engine results must be retained before calling that execution proof complete.
- Nonempty ER links are currently unreachable; generic serialization is inspected, not exercised through ER public rendering.
- Any speed claim requires the matched measurements. Serializer scanning/number formatting remains; older profile attribution is an upper bound on a broader subtree, not the removable fraction.

## Parent fixture corrections and follow-up source review

The first parent run reported 54 checks: 45 existing controls plus three nested-result checks passed, and six fixture assertions failed (two assumptions across all three engines). Old JSON and packet rows agreed; those failures do not demonstrate transport disagreement. Keep that attempt separate rather than rewriting it as a successful run.

1. ER fontSize/measurer NaN could still produce finite outer dimensions. The corrected test injects NaN padding (and retains NaN measuring output), requires success in all four lanes, requires an actually NaN width/height, and requires an observed successful packet. Exact computed bits still compare against the legacy endpoint, not against an assumed preserved input payload. This makes the fixture test the desired transport outcome rather than just an option's presence.
2. Standalone raw inline icons were unset, so the icon did not produce the intended large envelope. The corrected fixture uses `accDescr` quotes, which the SVG accessibility encoder expands to `&quot;`. It measures one/two-quote metadata lengths independently for each lane/mode, checks a safe integer growth step, and bounds the generated string to 12 MiB characters. With `first` as one-quote metadata and `step` as the measured delta, raw `count = 1 + floor((64MiB - 1 - first)/step)` yields metadata below the serializer ceiling; normal currently adds one quote, which crosses or exactly reaches the ceiling depending on the remainder. Source review flagged that narrow test-arithmetic edge: for guaranteed overflow, use `2 + floor((64MiB - first)/step)` for normal mode and assert `expectedBytes > 64MiB`. Parent has been notified; current successful Chromium observations did not hit that equality edge. The raw assertion additionally checks actual returned native metadata equals that expectation and is within 16 bytes of the ceiling. The existing nonempty graph guarantees a later combined-frame rejection. The normal assertion requires no completed endpoint response and the distinct numeric-transport error. Both require callback execution, exact old/new outcomes and successful repair/reuse afterward.

The calibration mutates only short accessibility strings before the large attempt. Their retained string-history cost is allowed and reinforces the final graph contribution; it does not substitute a metadata overflow for the raw combined-gate test, because successful native metadata is explicitly observed. The test does not retain the huge SVG/JSON in its evidence report.

Parent reports all three corrected Chromium tests passed; the cross-engine nine-check run was still underway when this source review was requested. This reviewer did not execute or independently reread browser result artifacts. Execution status above is parent-reported continuity, not a new browser proof claim.
