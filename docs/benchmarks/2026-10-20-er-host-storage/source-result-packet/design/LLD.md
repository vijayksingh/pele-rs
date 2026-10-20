# Additive source render result packet

Private candidate against `6dfa9cac0ed7cf92ee4e5e7cacec0c1e1aa9268e`, whose runtime is the accepted dense-array layer. No build, browser run, benchmark, adoption or release claim. No native core, Swift, C, parser, layout, model-storage or public TypeScript signature changes.

## Problem and precise lane

Ordinary `renderer.render(source, options)` renders in Rust, serializes the whole v2 success envelope into a bounded JSON buffer, converts that buffer to a JS string, parses it, restores exceptional numeric dimensions, and drops the intermediate string. The native `RenderResult` already contains the finished SVG, dimensions and links.

The actual exports are `NativeRegistry.render_numbers_with_resources` and the registry-free top-level `render_numbers_with_resources`. There is no exported `NativeEngine` class. Both routes are retained with the same signatures and returned strings. Add `render_numbers_packet_with_resources` alongside each route. Both old/new transports call the same private render body; neither parses, selects a provider, renders or calls resources twice.

`full.js` chooses the new method only when it is a function. The registry-free route probes a module namespace import, so an older generated module without the optional export remains loadable. Missing capability selects legacy once. Errors and invalid packets never trigger legacy replay.

Registered foreign providers retain the existing earlier `foreignParse` / `foreignRender` path, object identity, callbacks and cleanup. Ordinary retained models, prepared/raw entries, `MutableDocument` binary exchange and immutable parsed-document render exports are unchanged. Existing ER packets share the extracted encoder only; their operation/audit lifecycle stays in place.

## Primitive: count exact envelope, emit borrowed result

`render_packet.rs` accepts a borrowed `Rendered` and a caller-supplied `Serialize` envelope. It:

1. Calls existing `render_numbers::encoded_length` on that exact envelope. The existing serde serializer and per-write 64 MiB `Counted` gate are unchanged.
2. Encodes borrowed `rendered.links` with existing `encode`, after the whole-envelope gate. No `Rendered`, SVG, `LinkInfo` or link-vector clone is introduced.
3. Calls the existing generic `hostErRenderResult` factory with borrowed type/SVG/link JSON and width/height split into low/high `u32` bits.

The factory already creates an ordinary object with ordered `type, svg, width, height, links` keys (raw host calls omit `type`). Null placeholders precede scalar dimension assignment; a numeric array never carries NaN payloads. Links still use their canonical serde JSON schema, decoded once. No diagram algorithm moves to JS.

For source success, the counted envelope is **`{version:2,ok:true,result:render_result_transport(&value)}`**, including exceptional `$numbers` metadata even though the packet does not expose it. The old JSON encoder and new packet use one `RenderEnvelope` definition. For retained host operations, it remains the original `Result` / `{Ok:...}` serializer. The two formats are not interchangeable for budgeting.

The packet remains `[1, exactLegacyEnvelopeBytes, publicResult]`. JS validates version, safe integer count in `[0,64 MiB]`, exact ordered result and link keys, ordinary object prototypes and scalar/array types. Source permits any returned diagram type string, rather than imposing the ER-only retained-host condition. Successful results contain no `$numbers` field. Native Rust is authoritative for the type value; this is an internal capability, not an untrusted wire protocol.

## Ordering and failures

- Registry route checks open state before option decoding; the private shared body preserves this.
- Source/options preflight, raw numeric-mask validation, source/edge policies, preparation, captured registration, config, grammar, layout and SVG output limits keep their existing order.
- Registry-free route retains its existing `Engine::with_builtins` options semantics; it does not acquire the registry route's extra parse-limit arguments.
- Width, icons and clock wrappers are unchanged. Default measurer construction stays lazy until the first Rust width request. Nested source rendering still owns independent local result state.
- Native render or envelope-count errors return the exact existing v2 JSON failure string. `failure` and its bounded fallback string are unchanged.
- Link encoding follows a successful complete envelope count. Factory exceptions propagate as host exceptions; they are not mistaken for a native envelope or retried.
- The facade restores any captured original callback throw before parsing/validating the response. Existing `renderError` handling of `RangeError` is unchanged.
- Existing host ER `finish` runs before packet preparation exactly as before. The shared encoder keeps its existing thrown native-failure behavior, distinct from source errors returned as strings. Source packets do not enter `owner.finish` and do not acquire graph-plus-metadata budgeting.
- No retained model, lazy result, first-use conversion, pending scratch owner, callback borrow, extra provider serialization or resource callback is introduced.

## Reference count remains deliberate

This slice does not specialize SVG escaping or change `Counted`. The immutable result fields are pure, so a future exact counting formatter may be feasible. That requires a separate proof of canonical escaping, number/$numbers/type/link encoding, checked arithmetic, bounded scanning and identical failure precedence. It is not included here, nor is a text-width string cache.

## Saved profile scope

`profile-attribution.json` was derived by `analyze-profile.py` from the saved dense-array ER-medium source-render WASM CPU profile. Self weights: bounded serde `serialize_str` 6.6554%; facade `result` 5.8656%; text-width resource string decoding 4.5038%; resource `decodeText` 1.0316%; final returned JSON `decode` 0.6370%. Those are one saved diagnostic profile, not paired timing or all-family percentages.

Most reported `decode` work is resource input, outside this layer. The reference counter still scans/escapes the SVG; do not claim the 6.6554% disappears. Direct SVG transfer still decodes UTF-8; do not claim the entire 0.6370% disappears either. The large JSON allocation/parse and wrapper work are plausible targets, not predicted speedups. New JS validation, link JSON and capability dispatch also cost work.

## Public proof slice and gates

`tests/web/source-result-packet.spec.js` contains five E2E cases (one definition is parameterized):

- 33 existing family baseline sources plus a nonempty flowchart link source, normal public `loadRenderer().render`; packet versus forced old capability and pinned JS; complete SVG/result, ordered keys, dimension bits, resource sequence and exact old UTF-8 envelope counts.
- Registry-free `createRenderer(undefined).render` through the existing exported internal factory and renderer interface across all 33 families, with and without the optional top-level export. A route shim only removes that export (or wraps it to count actual use); old exports, WASM and facade stay identical. Explicit measurers avoid unrelated registry-free default font detection. Standard public loaders all currently construct a NativeRegistry; this test reaches the existing fallback branch without adding a loader mode.
- Numeric limits, negative zero, successful NaN dimensions, infinities, Unicode, syntax/source/edge/config errors, closed/empty registries, registered provider isolation, original arbitrary throws, clock throw, nested source rendering and recovery. Equality is actual old/new behavior, not two accepted errors where a successful transport is required.
- A bounded accessibility-source fixture calibrates actual envelope growth, then crosses 64 MiB JSON with input below 12 MiB. It requires the exact numeric transport error and a subsequent successful source render. It does not claim the unrelated raw graph-plus-metadata gate.

All reports are explicitly written to `testInfo.outputPath` before attachments, so list reporters retain the evidence. Fixture syntax and outcomes are unexecuted obligations; syntax checking alone proves neither native compilation nor browser behavior.

Parent validation: apply/check/build once, run this source feature slice in Chromium/Firefox/WebKit plus existing host result-packet tests (shared helper), all-family render-numbers/source coverage and relevant foreign-provider tests. Rebuild/package the exact candidate before tests: current raw target artifacts are not source-fresh. Keep native/Swift as negative controls; their inputs are unchanged by design. If correctness passes, compare matched source-render conditions against the frozen dense package, including all-family or representative non-ER controls and allocation/peak-memory evidence if available. No new 0.3 or full parity claim follows from this layer.
