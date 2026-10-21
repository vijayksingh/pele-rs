# Borrowed ER token payloads

Private candidate against `cbd629049b413631c32f2460dcc312b6bf725f70`. Two runtime source copies only: ER lexer and reader. No production edits, builds, tests, parser execution or measurements. `manifest.json` binds originals, candidates and patch. The current clone-to-move layer remains the control.

## Representation

`Token<'a>.text` becomes `&'a str`. `Scanner<'a>` emits slices of its immutable original source into `Vec<Token<'a>>`; its explicit ERROR token holds `""`. `Reader` ties its source and token vector to the same source lifetime. `take()` creates the existing owned `String` with `to_owned()` and advances exactly once. All target APIs and returned models remain owned and unchanged.

No lexical decisions, scanner state, byte positions, grammar actions, parser bounds, provider dispatch or fallback paths change. No token cache, shared parser state, borrowed model fields or unsafe code is introduced. The lexer still scans the full source and builds its token vector before the grammar runs.

## Why the slices are exact

Every nonempty token originates at the single `emit` function as `src[p..end]`; the old conversion to String performs no decoding, normalization or mutation. Direction detection creates a lowercase scratch string for matching, but emits the original source span. Quoted names/comments, backtick attribute contents, accessibility values and multiline text likewise emit original contiguous spans. The explicit style ERROR token is empty. EOF/END and empty title/description sentinels are empty source slices at `src.len()`.

The same indexing expressions already require UTF8 character boundaries. Borrowing changes ownership, not index calculations or panic conditions. Source positions and unexpected CHAR text remain identical. `fail()` reads the same unconsumed token text directly instead of calling `String::as_str()`; error snippets still use the original source.

`parse_into` owns the Reader only for its synchronous call and already holds the source reference for the whole parse. The independent target borrow does not own or mutate that source. No token or slice escapes into ErModel/PublicState: `take`, trimming, unquoting and target calls retain their current owned results. Repeated reads are now harmless, although the cursor remains monotonic and no replay/backtracking is added.

## Source-derived opportunity

For the literal ER workload shape in `scripts/perf/cases.mjs`, n >= 2:

- Nonempty emitted tokens: header 1, eight per entity, seven per relationship, and `2n-1` newline tokens; total `17n-7`.
- `take()` materializations: `10n-3`, unchanged from the prior source analysis. These still allocate/copy once, now at consumption instead of lexing.
- Skipped nonempty tokens: `7n-4`. Newlines, braces and relationship operators/cardinalities previously owned payloads until token-vector destruction. They now never own text.
- Removed skipped payload bytes: `10n-7`. This is payload length, not allocator overhead or retained heap.

| Literal workload | Nonempty emitted | Owned take | Skipped payload materializations removed | Skipped payload bytes |
| --- | ---: | ---: | ---: | ---: |
| ER small, 8 entities | 129 | 77 | 52 | 73 |
| ER medium, 48 entities | 809 | 477 | 332 | 473 |
| ER large, 200 entities | 3,393 | 1,997 | 1,396 | 1,993 |

These are source-derived counts, not instrumented allocation counts. They describe the literal fixture before any source-preparation changes. EOF/END are empty and do not count as payload allocations. `evidence.json` records input hashes and arithmetic. A slice also removes the String capacity word from each token's payload representation; actual compiled Token layout, vector capacity and resulting bytes require target-specific measurement.

On early grammar/direct-target failure, tokens after the consumed prefix no longer have preallocated owned text. The later fallback still lexes and parses again; this candidate does not remove the repeated scan. Quoted substring allocation, style concatenation, accessibility normalization, entity/database copies, token-vector allocation and direction's lowercase scratch allocation remain unchanged.

## Review and verification boundary

Source review found no transformed/synthetic nonempty token, escaping lifetime, text mutation, public contract change or changed error-order branch. Rust compilation remains unverified by instruction. No new unit or integration tests are added for this internal representation-only candidate; parent should reuse the public ER full corpus, direct-eligibility/canonical graph, source/title/graph-boundary and late-error/fallback integration slices.

Before adoption, compare exact public models/graph bytes/errors with the frozen control, including Unicode, quoted names/comments, backticks, empty/unterminated accessibility states, invalid style CHAR/ERROR, and subgraph EOF. Then measure allocation counts and matched parse/complete first-use controls against cbd. The small payload totals and many tiny avoided materializations suggest a bounded allocator opportunity; they do not establish a latency, heap or parity claim.
