# ER token ownership candidate review

Private candidate against `175daa8fd76fc1e25f488e4957b6970b4b6d4fe2`. The patch changes one expression in `Reader::take`: clone the current token String becomes `std::mem::take` of that String. No tracked runtime or integration files changed.

## Exact safety argument

The reader cursor only advances; the grammar has no rewind, replay, checkpoint or consumed-token lookup. Every take returns an owned String to the same consumer before the same cursor increment. Moving the String retains its exact bytes, including Unicode and empty values. The emptied slot retains its kind and source position. Consumers and target callbacks cannot access the private reader token vector.

Errors inspect the current unconsumed token. Source snippets use the unchanged original source and token position. Unexpected CHAR text therefore remains available. The terminal END is never taken; EOF is skipped directly, and an early lexical ERROR is rejected without taking it. The last-token clamp cannot expose a moved CHAR. A failed target drops the reader; the existing engine fallback creates a new token vector from the same prepared source. This patch neither reuses tokens nor changes provider selection, schema validation, admission, graph construction, callbacks or fallback routing.

Future backtracking or reuse of a partially consumed token vector must revisit this invariant. Ownership transfer is not compatible with merely rewinding this reader.

## Existing public coverage reused

No new test was added: existing public feature coverage exercises the affected grammar and its observable errors directly.

- `er_compat::pinned_er_parser_corpus` compares complete public typed models or complete serialized errors against the frozen 17,437-source oracle. Existing malformed sources include unexpected CHAR, incomplete style/block/accessibility description, backticks and nested subgraph EOF.
- `er_compat::pinned_er_complete_svg_corpus` exercises source parsing through complete SVG output.
- `public_er_direct_parse::er_direct_and_legacy_match_every_frozen_source_and_full_graph_table` checks complete graph binary tables/errors through direct and forced legacy public paths.
- Remaining `public_er_direct_parse` feature tests cover exact eligibility, first/edited rendering and helpers/aliases, captured providers, size/title limits, fallback and late syntax/error precedence.

These are coverage references, not fresh pass claims. No compilation, tests, browser execution or benchmarks were run for this private candidate. Private rustfmt completed and the formatted candidate was checked to differ by exactly the intended expression.

## Performance claim boundary

The change removes the explicit nonempty String clone at each take and transfers its allocation to the existing consumer. The source-derived counts in LLD.md/evidence.json are not allocator measurements or latency evidence. Lexer allocations, quoted substring conversion, target-owned copies, arena construction and repeated fallback tokenization remain. Runtime benefit requires the parent's matched allocation and timing checks.
