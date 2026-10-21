# Bounded successful ER result handoff

Status: private candidate now implemented in `files/`, with originals, patch and hash manifest. Static syntax/format/apply checks only; no build, test execution, profile capture or timing. Existing `HostErDocument::render_host` and `render_raw_host` remain JSON; additive packet methods are unexecuted.

## Recommendation

Prefer one additive successful-result packet over a typed ER draw snapshot. It removes a real representation round trip without changing any model reads or callback phases: borrowed Rust Rendered/RenderResult, full escaped JSON Vec/String, JS JSON string, parsed JS object, then another UTF8 allocation just to recover JSON byte length. Keep the existing serializer as byte-budget authority, but send successful SVG/dimensions directly and carry its exact metadata byte count. Errors retain the existing JSON transport. Methods and low-level public exports remain unchanged.

This is a bounded allocation/copying layer, not a claim that output transport explains the whole retained gap. Do not combine it with admission-frame or draw-reader changes in the first measurement.

## Evidence and ceiling

Frozen input: `../er-inline-publication/runtime/profile/er-medium-render-model-wasm.cpuprofile`, SHA256 `59a75f605b9b7073d92407aede9ffc87427349a7ef4ddc527f82b0c45c2f7954`. This predates the later admission-count/raw/prepared edits; no current-after-those-changes attribution is inferred. Weight each sampled node by its matching CDP timeDelta and count an ancestor at most once.

Of 1,545,088 sampled microseconds:

- Complete bounded JSON serialization subtree: 64,919 us, **4.202%**.
- Host-operation metadata `TextEncoder.encode`: 53,023 us, **3.432%** (including its sampled descendants).
- These two subtrees are disjoint: **7.633% total**, an upper envelope, not the removable fraction. Counting with the same serializer retains escaping/scanning/number formatting. Approximately **1.240%** self in output Vec extension is attributable to serialization byte copying; removing that and the final TextEncoder is a concrete smaller target, before replacement costs.
- `hostOperation` self is 3.030%; JSON.parse is not separately identifiable in this artifact. Do not call all of it parsing or add it as promised savings. Raw SVG still needs one UTF8-to-JS string conversion.
- Draw-side HostRead/HostStore union is 15.701%; actual typed-import union within it is 6.295%. `field` alone is 7.114% inclusive, containing real own-property reads, classification and conversion. These overlap and cannot be added. `capture` is inlined into draw, so no measured capture-only ceiling exists.

The required entry/final audits remain 12.150% and admission 18.207% in this older profile. A result packet removes neither. It cannot close the current several-fold JS gap alone.

## Exact current stages

`full.js::hostOperation` begins the host transaction, then invokes `render_host` or `render_raw_host`:

1. Decode numeric options (raw also decodes prepared config).
2. Create operation-local HostStore; validate_render; execute existing Rust renderer/resources and output limit.
3. Call `finish` JS callback, which performs final structural audit. A failure here precedes result serialization.
4. Serialize `Result<RenderResultTransport, PublicError>` or raw equivalent into the 64 MiB `render_numbers::Bounded(Vec<u8>)`.
5. Transfer JSON text; JS JSON.parse; encode the entire JSON again with TextEncoder to pass its byte length into owner.finish.
6. Preserve original captured callback throw identity, then transport/operation errors, then return renderedValue.

Keep stages 1–3 and 6 unchanged. Only successful stage 4–5 representation changes. Existing semantic/output limit and final graph admission do not move.

## Proposed primitive

Add optional facade-only `render_host_packet` and `render_raw_host_packet`. Factor each old endpoint's orchestration into a private body parameterized only by the final result projection, so old exported methods retain their exact JSON signatures and no duplicate renderer path appears. Do not change source rendering or methods in this first layer.

At successful result projection:

1. Serialize the **same existing Result::Ok transport value** using serde_json::to_writer into a bounded **CountingWriter**. It retains only `usize bytes`, implements the same write admission as Bounded, and returns the same `Browser numeric transport exceeds byte budget` error. It stops on the same write that would exceed 64 MiB. No custom JSON length formula or alternate float serializer.
2. After counting succeeds, call one private JS result factory with borrowed SVG, normal type string or a raw tag, width/height as two u32 halves each, serialized links only, and the exact count. All large output remains borrowed until wasm-bindgen copies it; no full JSON Vec or second SVG string is built in Rust.
3. Factory decodes the four numeric halves using DataView, parses only links, creates the existing public object in the existing property order, and returns an internal tagged packet carrying `{metadataBytes, value}`. A single factory import avoids per-field Reflect.set crossings. ER algorithms do not move into JS.
4. For errors, keep `metadata(&Err(...))` and the existing string return. For successful packets, hostOperation uses the supplied exact byte count rather than TextEncoder. It validates the private packet tag/count/shape before acknowledging publication. Old-WASM feature detection leaves the existing branch unchanged.

Links remain `Vec<LinkInfo>` with String/String/bool only. Their small standalone JSON can reuse serde_json rather than adding another object codec. They are counted as part of the full original envelope before this transfer; if a future ER result contains many links, account for the retained small-JSON assumption in measurement and keep standalone encoding bounded. No speculative empty-links-only special case.

Normal packet public key order: `type`, `svg`, `width`, `height`, `links`; raw packet: `svg`, `width`, `height`, `links`. Private tag/count never escape. Public returned shape matches the object after existing renderedValue removes `$numbers`.

## Invariants

- Byte count includes the exact old `{"Ok":...}` envelope, escaped SVG, links and `$numbers` fields. Although the packet carries no `$numbers`, its **virtual JSON count still includes them**. This retains graphBytes + metadata admission exactly.
- Width/height u32 halves preserve finite values, infinities, signed zero and raw NaN payloads. Reconstruct through the same DataView mechanism as existing renderedValue; do not pass NaNs through f64-only JS ABI or normalize them.
- Same Rust strings reach JS. Embedded NUL, escaped quotes/backslashes, non-ASCII, leading BOM and astral text require differential coverage. No TextDecoder configuration change.
- Do not allocate packet/link data until original full-envelope counting passes. Counting is bounded and can abort early exactly as existing serialization.
- Final structural audit still precedes serialization/counting; owner.finish still performs its virtual graph-plus-metadata gate afterward. Original resource throws still dominate later sync/transport errors according to existing hostOperation logic.
- No model field cache, new read ordering, validation exemption, callback suppression, ownership migration or persistent second graph. Nested renders use independent local result values and counts.
- Arbitrary foreign/custom providers and low-level document APIs keep their current serializer/provenance. The initial packet path is captured host ER only.

## Why not typed ER draw extraction now

`HostRead::capture` already captures owned scalar Atoms and retained array/style handles according to pinned phases. Later `attribute_at`, `cell`, `keys`, title and accessibility reads deliberately remain live. In render.rs, type/name/keys/comment are read separately around layout calls that can invoke width/icons and mutate later values. A whole-row or complete typed ErModel snapshot would violate proven callback cases. Coercion can also invoke arbitrary user code before ordinary width callbacks, so a simple resource epoch is insufficient.

A fused field+string primitive could preserve a single field's read/conversion boundary, but it still must keep non-string failure timing, per-occurrence charges, UTF16 rejection, immutable-ID cache behavior and original exceptions. It removes only some imports inside the measured 7.114% field subtree. No profile separates those removable calls from real reads/conversion, and a typed-row bulk read cannot be justified by the 15.701% union. Revisit after a fresh profile if output packets are adopted; do not infer a 16% win from deleting the union.

## Acceptance and measurement

Public integration/E2E first: old JSON endpoint versus optional packet facade across normal/raw/ordinary/prepared/caller-root lanes; exact SVG, dimension bits, links, key order and original resource exceptions; nested same-model render; malformed and oversized output; graph budget plus output metadata near the frame boundary; post-error repair. Keep all existing live callback/alias/descriptors tests. Test old-WASM feature fallback.

Matched VM: fixed and canvas resources at small/medium/large ER, success and output-limit failures, first render and retained render. Source/parse and another family are unchanged controls. Freeze all runtime files and WASM hashes, preserve adverse rows, and compare paired raw trials. Record successful SVG byte size and original JSON metadata byte size outside timers. Allocation diagnostics distinguish removed full JSON bytes from live JS SVG ownership; do not call lower requested bytes lower RSS.

Expected measurable conditions: large successful SVG with many quotes/escapes makes the removed encoded buffer/copy and second UTF8 encoding largest; tiny diagrams may lose to packet/factory overhead. Failures intentionally retain legacy JSON and are controls. No latency prediction until measured.
