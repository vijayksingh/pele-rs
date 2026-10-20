# Source-backed canonical models: rejected checkpoint

Source `d35ca5b`, default OptS WASM `707da8ce4764afdcc1210f27c8d637a16e70f7077ec0bc995671f2a59b5d3d1f`. The native source owner and sparse edits replace a retained generic arena and its nested JS decode. The full API, validation, publication and recovery behavior remain required. This implementation **regresses performance and is not a release candidate**.

The VM-agent comparison uses the frozen e148 SVG-append default and pinned Pele JS, 24 conditions, 21 balanced trials, 1000 ms warmup. All eight separate activation checks confirmed the intended public path and restored prototypes before closing the probe browser. Package maps remain exact: control 113 files, candidate 115. [Every row](RESULTS.md), [raw samples](warm.json) and [provenance](warm-provenance.json) remain available.

Sankey parse increases 49.04–57.33% paired and measures 16.02–22.63×JS. Retained rendering increases 63.47–70.44%, with zero lower candidate trials in every row. Five of six direct-source rows and all Flowchart controls are adverse; 23/24 rows are adverse overall. First-use timing was not run after rejection. Percentages are not compounded across windows.

Correctness: 37 native integrations; 69 focused plus 357 remaining enabled browser checks across Chromium/Firefox/WebKit; 57 disabled-backend checks skipped; fresh local arm64 SDK Swift 16, strict C 3 and API 168. Swift exercised 12,481 drawings and 16,572 expected errors, with zero explicit budget differences. These are correctness checks, not Swift timing or complete release proof.

The initial browser run had one failed comparison because independently created fixture NaNs already had different bits before transport. Each path preserved its own bits. Test-only commit `6d302d5` checks those original bits across ten checkpoints and four graph signatures, then normalizes that one input for cross-lane comparison. The original failure is preserved; all runtime/package bytes remain those of d35, verified in [test continuity](test-continuity.json). Boxing/JIT causation was not established.

A separate nine-window [CPU diagnostic](profile/FINDINGS.md) retains raw profiles and exact inputs. Derived native batch synthesis occupies 25.51% of candidate parse self samples; `properties` alone 16.91%. Native validation overlaps that synthesis and must not be added to it. JS layout reads/decoding add cost; retained GC is 14.50% versus control 0.70%, without a causal allocation attribution. Sampling percentages do not establish latency improvements.

The physical canonical-fallback decoder is exercised. The actual native 64 MiB recipe-overflow branch is not yet triggered by a test. No 0.3 release, Intel SDK or AppKit activation claim.

Some raw verification logs are gzip-compressed to preserve their exact bytes, including tool-emitted trailing whitespace.
