# Borrowed ER arena string interning

Private candidate only. No production edits, execution, timing or speedup claim.

## One seam and its actual reach

`GraphWriter::string(impl Into<JsString>)` must perform the caller-supplied conversion before it can look up an interned value. ER's construction-only `InitialWriter::string(&str)` currently delegates to that generic API. It therefore allocates a temporary UTF16 vector for repeated nonempty short IDs, labels, aliases, attribute values, comments and relationship strings even when the value is already interned. Property keys already have a stack lookup in `GraphWriter::property`; that path is unchanged.

Add a crate-private borrowed `GraphWriter::string_str(&str)` and use it only from ER's existing `InitialWriter for GraphWriter`. It uses the existing 64-unit scratch helper, whose eligibility condition is **UTF8 byte length <=64**. A hit returns the existing ID without an owned UTF16 vector. A miss copies the already encoded units into `JsString` and calls a shared private insertion helper, avoiding a second UTF16 traversal and a second lookup. Longer strings retain the existing generic conversion path.

This affects ER arena construction in native public parse, Swift mutable parse, and low-level WASM arena parse. Ordinary host-owned ER web parse uses `HostConstruction` and its separate UTF8 interner: this patch does not remove work from that path. Immutable typed parse/source rendering also do not use this seam. Do not promote native allocation results into a general web parse claim.

## Preserved contracts

- The public generic `GraphWriter::string` still executes `Into<JsString>` exactly once on every call, including hits. It still accepts arbitrary raw UTF16 through owned `JsString`.
- Exact UTF16 equality and hash equivalence come from the existing `JsString: Borrow<[u16]>` implementation. There is no new hash function, collision policy, schema cache, or retained index.
- First IDs, insertion order, duplicate handling, graph nodes, property order/descriptors, aliases and IEEE numbers are unchanged. Both APIs use the same insertion body after an absent-key lookup.
- The ER trait method signature remains object-safe. `record` retains the existing `Self: Sized` restriction, and default receiver dispatch remains unchanged. Existing `public_er_host_fallback` deliberately invokes the graph sink through `dyn InitialWriter`.
- GraphWriter has no per-string work charge. Table/reference/UTF16 work admission still occurs in `finish` at the existing stage, with the same graph and error precedence. Direct eligibility, typed fallback, parser/source admission, schema validation and provider routing are unchanged.
- No callback runs in borrowed lookup, and no Rust borrow survives a call to an external sink. No WASM/Swift ABI, core model or JavaScript file changes.
- On hits, the 128-byte stack scratch replaces a temporary owned UTF16 vector. Misses still retain the graph string plus cloned HashMap key; this is not the rejected StringIndex representation change. Empty-string hits may already allocate nothing. Vec capacity on misses may differ from collect; measure rather than assume an exact byte count.

## Public proof slice

New `public_er_borrowed_strings` compares actual builtin public parse with a separate public host publication sink that explicitly calls the original inherent owned-conversion API. It checks complete binary frames, canonical string uniqueness, UTF8 thresholds (ASCII64/65, é32/33, emoji16/17), both direct and style-forced typed fallback, and exact late-syntax/source-admission errors. Intended valid inputs must succeed; equality of two errors is not accepted as success.

Reuse `public_er_host_fallback` (all17,437 frozen sources, dynamic sink dispatch, real graph overflow and later syntax precedence), `public_er_direct_parse`, and `public_short_keys` (raw surrogate strings, keys, cycles and number bits). Tests remain public integration; no private-helper unit test. No tests were run while preparing this patch.

## Allocation and timing evidence plan

Parent freezes the current production control consumer and manifest before applying. Build candidate independently with identical compiler/profile/settings; record source and binary hashes. Use the existing separate allocation consumer for ER medium `parse-public` at counts1 and16. Record allocations, reallocations, total requested bytes, peak live and retained live separately. Expected affected mechanism is temporary UTF16 allocation on repeated short values, not disappearance of the canonical graph or its retained storage.

Run unchanged typed `parse`, `render-source`, and typed `render-retained` as negative controls, plus another arena family (for example Flowchart medium) because the public generic method shares the extracted insertion helper. `render-mutable-publish` may create a fresh model in setup; separate measured body from setup rather than calling every observed change unaffected. Existing ordinary host web parse is a scope control, not evidence of a speedup from this helper.

Only after allocation results justify the layer, use the existing guarded15-trial matched native comparison with the same frozen artifacts. Do not time the allocation consumer. Preserve all raw samples and adverse controls; report paired distributions, not a predicted CPU benefit or subtraction of negative-control medians. First-time/unique-heavy input should be represented in the integration boundary slice and, if timing is warranted, a bounded unique-value workload alongside repeated ER medium input. No runtime instrumentation/cache is added by this patch.
