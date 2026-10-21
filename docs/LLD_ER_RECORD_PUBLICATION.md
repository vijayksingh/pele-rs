# ER fresh record publication layouts

Unreleased construction sink architecture. Parsing, methods, rendering, callbacks,
imported/live graphs and native/Swift ownership retain their existing contracts.
Performance requires the independent VM comparisons described below.

## Evidence and target

Current frame profile (`er-entry-read-frame-first/profile`, WASM `1036593e94ae9113a848aaf1a0cbb118e05991d6abdc3c6e977fec17bb4d66f2`) assigns ER medium parse 19.90% self to JS `allocate`, 7.55% self to its generic property publisher and 10.16% to GC. Rust `HostConstruction::string` is 19.76% inclusive, but contains flush/publication work; it is not an independent interning percentage. Do not add these overlapping scopes or predict a gain.

The 48-entity workload has 437 required public nodes. This slice handles 238 ordinary records: 96 attributes, 48 entities, 47 relationship specifications and 47 relationships, with 1001 ordered properties. These are source-derived counts, not measured allocation results. All 437 final objects and their immediate registrations in both ownership WeakMaps remain required and unchanged.

Current fixed records travel through a field Vec, a Property Vec and Node, then repeated key/flag encoding, a temporary JS constructionNode, and per-property previous-descriptor reads. A repeated immutable field layout allows those intermediate representations and repeated layout work to disappear while retaining the real final objects. No typed ErModel is added to the direct parse lane; bulk publication from a second whole typed model would reintroduce the conversion already removed.

## Primitive

`InitialWriter::record<const N>(keys: &'static [&'static str; N], values: [Atom; N])` is a defaulted construction-only sink method with `Self: Sized`, preserving existing dynamic writer interfaces. Its default calls the existing `property` method in order and then `node(Object)`. GraphWriter's overridden property method therefore retains every existing interner/work charge. The counting fallback sink retains its table, reference and final binary-byte error ordering. External implementations of this doc-hidden trait receive the default; no provider opts into a new route.

Five Rust layouts cover entity, the two attribute comment/key orders, relationship specification and relationship. Values are evaluated into stack arrays at precisely the previous point/order, then keys are processed. Entity ID, name, array children, attribute keys/comment/type/name and specification Debug formatting are unchanged. Dynamic class/group records, instance root, Maps/Sets/arrays and arbitrary Node publication keep the existing generic path.

`HostConstruction` remembers at most 32 layout identities. Identity is a private static pointer plus length, never serialized; immutable storage cannot be reused during construction, and linker merging identical arrays preserves the same content/order. Keys must number at most 64, be unique, and exclude `__proto__` and `length`. A new or unsuitable future layout uses the original generic path when the optional bounds fail; there is no new public rejection. No fields, strings or layouts persist in Rust after construction.

The first occurrence interns all keys at the original rec() stage and declares their canonical string IDs. Subsequent occurrences are proven interner hits: HostConstruction accounts only new text/table storage, so skipping those lookups changes no counter. GraphWriter/default sinks still visit every key. Node admission follows key interning as before; node IDs are allocated at the same call site/order.

## Bounded storage events

Existing stream event limits remain 128 events and 65,536 bytes, checked before allocating a transfer buffer. Existing compact atoms remain 1/5/9 bytes.

* Tag10: layout ID u8, field count u8, then canonical key IDs u32. Total `3 + 4*N`, at most259 bytes. JS validates count, consecutive ID, actual string keys, uniqueness and reserved-key exclusions once before storing this bounded metadata.
* Tag11: node ID u32, layout ID u8, then N compact atoms. Total `6 + sum(atom lengths)`, at most582 bytes. JS checks the minimum payload/count and next node ID before allocation, creates the final ordinary object, registers both WeakMaps immediately, and writes each unique flags7 property in layout order. It keeps only local object/count variables, not constructionNode or Property objects.

Raw numeric Atom bits use the existing decoder and NaN values retain the descriptor publication path. There is no conversion of numbers through JSON or a new numeric array. UTF16 string transport is untouched, including surrogate chunks and first/last string history. Unknown/duplicate/nondefault descriptors, `__proto__`, explicit length rewrites, holes, Map duplicate-key and Set SameValueZero accounting remain on generic events.

For each optimized record, exact canonical bytes are `5 + 5*N + sum(atom wire lengths)` and references are N. Store these aggregates in the existing nodeBytes/nodeRefs arrays. Existing constructed admission still includes all table/string/node bytes and the same 13 metadata bytes; no graph limit moves or disappears. Release JS layout metadata after constructed admission, and reset it on unpublished fallback/abort. No caller code runs during supported pure construction; changed intrinsics remain outside the established contract. First method, render, snapshot/history audit and disposal retain their complete existing work.

## Files and validation

Runtime files: ER `public_model.rs`, `direct_parse.rs`; WASM `host_er.rs`; JS `host-er-store.js`. No schema field names or diagram algorithms are duplicated in JS; it consumes generic ordered keys from Rust.

New public E2E `er-record-layout.spec.js` compares complete own-property order/flags, values/aliases and first retained helper/render/edit behavior against pinned JS for eight frozen sources in ordinary and standalone prepared lanes. Includes both comment/key orders, Unicode, aliases, repeated declarations forcing fallback, reserved-looking names, classes/subgraphs and cyclic readonly NaN/-0 extras. NaN extras prove subsequent full API behavior; current specialized source record fields are strings/refs, so this does **not** claim direct numeric-record construction was executed.

Required reused public gates: `public_er_direct_parse` (17,437 complete native graph/error comparisons), `public_er_host_fallback`, `public_er_prepared_host`, `public_model_construction` (frozen externally authored graphs/descriptor semantics), existing ER corpus/oversized/chunked construction and callback E2E. Existing large-source, table, late-grammar and result-envelope/combined-byte-gate checks remain mandatory. The exact initial caller-root zero-versus-13-metadata-byte boundary remains source-audited rather than executed; the result-envelope tests do not close that gap. Duplicate own fields/nondefault descriptors and imported NaN payloads deliberately stay generic; do not fabricate a private protocol/unit test to claim a new caller capability. Low-level and native/Swift APIs retain the default publisher, so their frozen expected graph tables are stronger than rendered SVG alone.

Measure same-source default-build package versus this candidate across all ER sizes/resources and first parse+invoke/edit/first callback; source and non-ER negative controls. Record native allocation counters separately if desired. O3 is a separate experiment and must not be combined into this attribution. Possible regressions include layout lookup/declaration overhead on tiny models and extra specialized code size. No latency or memory improvement is asserted before measurement.
