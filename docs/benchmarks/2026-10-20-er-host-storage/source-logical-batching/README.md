# Bounded logical record batching: still rejected

Source `ee114c0`, default OptS WASM `7cc2aaed10097da5f89e50646c5194f5b7e8f5d154cfffe152cd231ef7f1b2f1`. One native source-store change uses 8 scratch slots for small records, 128 for larger batches, resolves the base/patch once, and borrows complete replacement Map slices. All validators and admission stages remain. This is **not a passing performance candidate**.

Matched VM-agent control remains frozen e148 plus pinned Pele JS. The timed harness/cases are unchanged:24 conditions, 21 balanced trials, 1000 ms warmup. All eight public-path activation checks and complete 115-file package maps pass. Only WASM differs from the rejected d35 package; handwritten JS and generated glue remain identical.

Sankey parse is 18.78–28.91% slower paired than e148 and 7.03–17.82×JS. Retained rendering is 36.39–46.87% slower and 2.50–3.98×JS, with zero lower candidate trials in every row. Source changes range −0.18% to+1.45%. All Flowchart parse/retained controls are adverse;22/24 total rows are adverse. [All rows](RESULTS.md), [raw samples](warm.json) and [provenance](warm-provenance.json) are retained. Separate d35/ee114 windows do not establish a direct batching speedup. First-use was not run after rejection.

Correctness from this exact source: 24 native integrations including the complete 11,616-case Sankey source corpus; 69 focused browser checks across Chromium/Firefox/WebKit; fresh local arm64 SDK Swift 16, strict C 3, API 168. Swift exercised 12,481 drawings and 16,572 expected errors, with zero explicit budget differences. The broader 426 enabled browser checks belong to the preceding d35 runtime/test-only6d302d5, not this rebuild. All 366 Swift inputs and 597 web build/test inputs were verified unchanged through their checks. No new Swift timing or allocation measurement.

Generation-bound validation receipts are being developed separately; they are absent from these measured artifacts. No 0.3 release or all 33-family performance parity claim.
