# Private ER lexical dispatch candidate

Only `crates/pele/src/compat/diagrams/er/lexer.rs` changes. No production edits, compilation, browser run, benchmarks, or claimed speedup. Baseline lexer SHA256 is recorded in manifest.json; parent HEAD observed feb22d6. Candidate patch is independent of the raw ownership and output packet layers.

## Measured scope

`attribute.py` reads the existing raw51f1269 ER medium WASM parse CDP profile. It weights each sample by its corresponding time delta and deduplicates ancestors. Exclusive shares: constructor flush 43.317%, lexer 10.427%, constructor string excluding flush 9.615%, constructor node excluding flush 4.659%, detached GC 10.612%, other 21.371%. Lexer self is 6.121%. JS `allocate` self is 18.961%; this is creation of public JS objects and bookkeeping, not the Rust allocator. Rust dlmalloc ancestor union is 2.266%, including only 0.218% inside the lexer. GC lacks causal allocation stacks and cannot be assigned to a Rust temporary.

This change targets a measured native computation subtree. Removing even the entire observed lexer subtree would not close the 4–10x JS parse gap. These percentages are attribution, not predicted wall-time reductions. String/node inclusive shares overlap flush and must not be added to it. Fresh profile input SHA and reproducible weighted outputs are in attribution.json.

## Exact change

`Scanner::direction` no longer allocates `line.to_ascii_lowercase()` and builds four reverse substring searchers. It scans original-source d/D candidates right to left once, checks the same ASCII spelling, and records the highest-priority matching direction. First TB terminates the scan; lower priorities keep scanning because a higher-priority rule farther left wins. Initial keyword dispatch selects one of seven static slices before applying the unchanged `kw` predicate. Relation spelling dispatch selects one of five static slices before applying the unchanged predicate. No caches, retained representations, token ownership, parser behavior, or publication changes.

Ordinary non-keyword initial tokens previously attempted 19 keyword predicates and 11 relation predicates. Most identifiers now attempt none; the largest keyword group tries six spellings and the largest relation group tries three. Block attribute scanning is unaffected.

## Compatibility argument

- Rule order stays TB, BT, RL, LR; reverse traversal and exclusion of already matched priorities retain the rightmost valid occurrence per rule.
- `direction` has no left word boundary. d/D candidates are checked anywhere in the initial line, as before. The literal has no proper self-overlap, so reverse char-candidate enumeration preserves the original substring search occurrence set.
- The initial candidate line stops at LF, CR, U+2028, or U+2029. After the literal, `until(space)` deliberately still sees the whole source and can cross all those boundaries. After the direction label, the emitted span stops at LF only. No shared newline abstraction replaces these distinct policies.
- Existing code uses `to_ascii_lowercase`, not Unicode `to_lowercase`, in direction matching. Keyword and relation comparisons already use `eq_ignore_ascii_case`. The candidate uses exactly those ASCII semantics. Unicode initial characters cannot satisfy an ASCII first character under this predicate, so skipping those alternatives requires no fallback. No Unicode text, attribute, whitespace, or boundary predicate changes; Kelvin sign, dotted I, long s, supplementary characters, and combining text remain handled by the original paths.
- Slice accesses remain `get`, source positions come from UTF8-safe matches, and every successful direction still advances to a validated two-byte ASCII label before slicing. Input source limits remain untouched. At+9 and q+2 cannot overflow within admitted source lengths; existing code already uses those additions. No new allocations or length-derived reservations are introduced.
- Same first-character alternatives keep their exact order. Alternatives in distinct groups cannot match the same input under the existing ASCII case-insensitive predicate. `classdef` stays before `class`, multiword cardinalities stay before `one`, and unbounded `many(0)`/`many(1)` keep their original flags. MD_PARENT still overrides relation matching at its existing point.
- Errors, failed_dir cache lifetime, token offsets, resource charges, max-edge handling, and grammar fallback are untouched.

## Source investigation outside this patch

`direct_parse::parse_into` admits source/title up to 64 KiB. Direct<W>::ensure_entity seals the entity map at the first relationship; later new entities, duplicate aliases/attributes, styles/classes/subgraphs can fall back. A failed direct attempt can therefore tokenize/parse a prefix and then run the typed grammar again. This is real duplicated work, but ordinary benchmark fixtures declare entities before relationships and take the direct path. Removing fallback scans would not explain their current gap.

Fallback/prepared publication uses `host_initial::admit` -> `initial_into(Admission)` and then `initial_into(HostConstruction)`: it builds temporary node/property data twice, and each writer interns strings. Ordinary direct parse bypasses that admission traversal. `public_model::rec` constructs field/property vectors, `arr` formats index keys, `texts` constructs intermediate atoms, and relationship enum text is formatted. These are candidates for later writer interface deepening, but the current profile does not justify treating Rust allocation as the dominant 19% allocation category.

On the existing 48-entity/47-relationship medium fixture, publication makes 1,969 string requests for about 181 distinct strings: 48*29 entity/attribute requests, 47*11 relationship record requests, 47 array indices, and 13 finish/root requests. A later explicit interned schema-key writer interface could remove repeated hashing and temporary key formatting. It must preserve first-use ID order, UTF16/string/table budgets, and failure order across both writers; this patch deliberately does not mix that separate protocol change into lexer dispatch.

## Public feature proof prepared

`er-lexical-dispatch.spec.js` contains 70 bounded public source comparisons using the packaged renderer and pinned JS oracle. It compares exact parse failures or whole identity-aware model graphs (including property flags/order, Map/Set membership, prototypes, number bits), complete rendered results, measurement traces, and post-render graphs. It writes JSON evidence before assertions. Explicit success controls prevent passing solely because both lanes reject valid cases.

Coverage: direction priority, rightmost matches across LF/CRLF/U+2028/U+2029, missing boundaries, final LF-only span, mixed case, Unicode lookalikes and identifiers, all keyword/cardinality groups, punctuation relations, accessibility, subgraphs/styles, and malformed input. Tests are not lexer mirrors and do not call private tokenization.

Parent validation, after atomic integration and a fresh package build:

```
cargo test -p pele --test er_compat
npx playwright test --config dist/perf-patches/parse-architecture/er-lexical-dispatch/playwright.config.mjs
npx playwright test tests/web/additional-diagrams.spec.js --grep '^er pinned corpus'
```

The existing er_compat parser/SVG tests and ER browser case cover all 17,437 frozen ER vectors, rather than copying or reducing the frozen corpus here. Run applicable native/Swift public gates once with the integrated layer before timing. This agent ran only rustfmt parse/format, JS syntax checks, git apply --check, and offline attribution. No runtime parity result exists yet for this candidate.
