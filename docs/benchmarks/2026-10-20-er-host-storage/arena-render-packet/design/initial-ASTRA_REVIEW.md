# Independent frozen source review

Reviewed patch SHA256:
`916ab23920ddbf70764dd09ed22b8f5161c3a32a36f4318c791331589bb22d7c`.
Base: `b19f38d6368d1ea8183004d24c58af5828b3f577`.

Result: **no concrete source blocker found**. Approved for parent application and execution gates, not a claim of compilation, browser parity or performance.

## Identity

Independently recomputed the patch hash. All 12 manifest candidate hashes match their files; all original hashes match both the saved originals and current production files. The two new test paths are absent from production, matching null original hashes.

Reviewed runtime mapping:

- `compat/public/engine.rs`: additive view outcomes; shared dispatch with explicit owned/borrowed export policy.
- `compat/public/mod.rs`: exports the two new view types.
- `pele-ffi/src/{public,prepared,presentation,native_resources}.rs`: corresponding binary consumers select views.
- `pele-wasm/src/{host_functions,prepared_entries}.rs`: additive normal/raw packet exports.
- `pele-wasm/src/render_packet.rs`: exact combined admission and full-graph/direct-result packet.
- `bindings/web/full.js`: capability selection, validation and publication.
- Two public feature-slice tests: native extension behavior and browser retained/raw behavior.

## Runtime invariants

`GraphExport::Owned` forces `Cow::into_owned` at the original post-render snapshot stage, before deferred presentation publication. The old API wrapper subsequently moves that owned graph. `Borrowed` retains the immutable post-state reference; the return lifetime is tied to the document, not temporary resource wrappers. Export-only extension fallback stays owned.

Both paths retain pre-state validation, dispatch, native revision advancement on dispatched errors, post-state schema validation and presentation ordering. There is no unchanged-graph shortcut, proof token, replay or postponed first-use work.

The C operation guard outlives the borrowed outcome and binary encoding. `with_resources` uses `FnOnce`, so returning a document borrow does not retain its temporary resource borrow. All `render_outcome` helper callsites use the new view type. C signatures/metadata serializers and v4 JSON are unchanged.

Successful WASM metadata counts the same legacy Result envelope, including special-number metadata. The count is bounded before subtracting it from the binary budget. Empty metadata leaves header width unchanged, so reduced-budget graph encoding admits precisely the old combined frame size. `encode_binary_exchange` still validates the graph before writes. Error outcomes keep original full binary metadata and no operation is replayed.

The facade decodes the complete graph and validates packet/count/revision/result before publishing. Every retained object still reconciles. Revision and acknowledgment happen only through the existing complete publication path. Original resource throws preserve their existing priority.

Status 1 explicitly handles a new result-factory failure after native commit: graph/revision publish first, then the original JS value is thrown. This is not a claim of matching arbitrary monkey-patched intrinsic behavior; the LLD states that scope.

## Feature test review

The fixture catalogue contains exactly 32 non-ER baseline entries. Adding one nonempty-link case and testing both normal/raw lanes gives the asserted 66 comparisons. The forced legacy endpoint and packet endpoint use the same compiled package, comparing full returned graph/revision, exact metadata count, SVG/result key order, dimension bits and resource traces. The broader pinned-oracle suite remains necessary because this differential shares core dispatch code.

The native extension tests cover borrowed and export-only providers, normal/raw behavior, exact graph bytes and numeric bits, partial mutation followed by error, native callback error, post-render invalid graph, revision rejection/repair and presentation ordering. Public view lifetime/type integration remains a compile gate; it was not executed here.

The browser callback slice now first publishes the detached array before removing its root reference, so it exercises retained history. It compares cycles, aliases, descriptor restoration, arbitrary object/undefined/symbol throws, malformed accessor edits, output limits, nonfinite dimensions and repaired ordinary operations.

The combined-budget fixture's algebra is correct. With third calibration frame `F3`, metadata growth `s` per quote and new string length `n`, the expected legacy frame is:

`F3 + 4 + 2*n + (n - 3)*s`.

The count selects the first integer `n` strictly exceeding 64 MiB while metadata alone remains below it. Prior calibration strings remain in canonical history and are already included in `F3`. Bounds prevent an uncalibrated large graph; this still intentionally creates a near-limit SVG/string workload and needs the parent's controlled execution lane.

The attempted width-call count is captured before retry/recovery, so recovery cannot supply the positive callback-stage assertion. A failed final encoding advances native revision without delivering a frame. The test now expects the **existing stale-revision failure on same-document retry**, then separately proves fresh-document recovery. This transport-failure recovery gap is preserved, not repaired.

All browser reports write artifacts before assertions. No private protocol/unit tests were added.

## Remaining gates

No builds, tests, browser activity or timing were performed during this review. Parent must compile native/WASM/C consumers, run the public slices and consolidated parity coverage, refresh affected SDK artifacts, and measure matched controls before adopting a performance claim. Validation-token optimization remains parked separately.
