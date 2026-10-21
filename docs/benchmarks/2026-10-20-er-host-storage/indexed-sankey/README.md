# Indexed Sankey source ownership

Runtime source `31dab0b`; fresh-render recovery test `88b87ec`. Default arena backend, no experimental selector. Indexed Rust source stores each node once and links by index. The public Rust DTO/downcast, JavaScript aliases and helper behavior remain unchanged; edited/raw documents retain the existing importer.

## Matched VM results

Control `e559745`, candidate `31dab0b`; only WASM differs among 113 packaged files. Same Chromium 156 browser on vm-agent, pinned Pele 1.6.0, fixed/canvas resources, Sankey small/medium/large and Flowchart medium control. Each phase retains all 24 rows and 21 balanced pairs per row, including adverse results. These are diagnostics, not the complete 126-condition release gate.

- Warm parse: six Sankey rows lower **3.64–6.69%**, still **7.93–14.31× JS**.
- Retained rendering: paired changes −1.26% to −0.12%; lane medians can disagree. No general speedup claimed.
- First unchanged render: five Sankey rows lower, but **large/canvas regresses 6.50%**, 7/21 lower pairs.
- First edited render: six Sankey rows lower 0.67–4.08%, still **3.52–8.14× JS**. The edit adds a cyclic extension property; it is not a diagram-content edit.
- Source and Flowchart controls are mixed. Every sample remains in [warm.json](warm.json), [first-use.json](first-use.json), [warm table](WARM.md) and [first-use table](FIRST_USE.md). Provenance records the exact package maps and runner inputs. Percentages are not compounded across windows.

## Local verification

22 distinct native feature integrations pass, including the complete 11,616-scenario public Sankey corpus. One new fixture initially expected the wrong error text; its initial failure and corrected run are retained. Runtime behavior already matched the oracle before correcting that fixture.

45 browser checks pass across Chromium, Firefox and WebKit: pinned Sankey corpus (3), public models/binary graph/comparison/diagram entries (39), and first-render callback failure/recovery (3). This is focused verification, not a new full-suite claim.

Fresh arm64 SDK passes **16 Swift integrations, strict C3 and 168 API conditions**, covering 12,481 drawings and 16,572 expected errors, with zero explicit budget differences. All 363 native/SDK inputs stay unchanged through correctness. [Source and artifact bindings](swift-correctness.json). No new Swift elapsed timings were collected.

Separate default/release3 native parse-public allocation counters:

| Case | Allocations, control / candidate | Requested bytes, control / candidate |
| --- | ---: | ---: |
| Sankey small | 189 / 171 | 19,971 / 18,291 |
| Sankey medium | 834 / 734 | 99,772 / 89,297 |
| Sankey large | 3,272 / 2,866 | 404,323 / 361,617 |
| Flowchart medium control | 3,431 / 3,431 | 493,299 / 493,299 |

Reallocations unchanged; all live deltas zero. [Full counter results](native-allocations.json). Allocation counts are neither latency nor RSS. The strict build/test guard passed; no retry or guard relaxation.

**JS performance parity remains unmet. No 0.3 release.** Shared SVG formatting is the next rendering slice; the larger mutable-parse publication cost remains separate.
