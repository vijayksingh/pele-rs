# Rejected captured backend: measured storage boundary

The backend stays disabled. Parent's ordinary 18-row matched benchmark rejects it; this diagnostic explains the mechanism rather than replacing those timings. Six medium/fixed profiles completed using Chromium 156.0.8078.4, sampling every 250us for two seconds per lane. Control and candidate package trees were identical at all 112 files before/after. WASM SHA256: `fbaa003fc95224a27cd8f7a49964a4a271b85f9a1dfaac4c44a3ecc4bed3dee2`. Retained/fresh backend probes and whole SVG/dimension checks passed before/after every condition.

| Weighted attribution | Arena parse | Captured parse | Arena retained | Captured retained |
|---|---:|---:|---:|---:|
| JS self | 40.26% | 79.63% | 30.95% | 72.59% |
| WASM self | 41.74% | 13.65% | 59.07% | 21.30% |
| GC self | 14.09% | 2.41% | 0.47% | 0.23% |
| Admission/adapter/import ancestor union | 3.23% | 75.99% | 2.64% | 57.02% |
| Raw Sankey parser inclusive | 2.51% | 0.40% | 0% | 0% |

The union counts each sample once, even if admission, the native captured adapter and JS imported storage all occur on the same stack. It is a description of where execution occurs, not an avoidable percentage or speed prediction. Remaining self samples are engine/profiler frames. A GC percentage across two-second windows with different completed-operation counts does not measure allocation per operation.

Captured parse spends 57.77% self in captured-graph.js. Largest frames: requireValue 9.08%, #page 8.93%, get 8.71%, set 8.14%, propertyAddress 6.76%, #checkNode 5.10%. Native admission is 51.21% inclusive; JS imported storage is 63.84% inclusive. Initial native construction is 29.08% inclusive and shared initial publication 8.48%; these groups overlap.

Captured retained spends 51.40% self in captured-graph.js. requireValue alone is 28.74%, #page 8.26%, propertyAddress 7.77%; capturedProperties/glue and capturedNodeHeaders are additional visible work. Native admission is 56.35% inclusive; imported JS storage 42.93%; shared matches 10.60%, full publication 10.25%. The profile does not establish COW allocation counts: #page also performs ordinary reads and guards. GC is not the dominant observed captured cost.

Source-render control has no admission/adapter/import union in either backend, and its parser/render/formatting stacks remain the ordinary source pipeline. This supports the storage placement explanation; it does not excuse negative control variation or establish unprofiled latency.

## Concrete source path, not sampled call counts

Current source creation performs structural validation at the InitialWriter/GraphWriter-equivalent finish checkpoint, then `document.snapshot_admission` (structure plus schema IDs), then `admit_outbound` (structure plus exact canonical frame count). Successful retained render runs snapshot admission both before and after rendering, then outbound structure/count; the JS owner additionally checks packet/frame sizing before publication. These are explicit source checkpoints. CPU samples do not count calls and the report records phaseCallCounts=null.

In the rejected design, each native pass repeatedly asks JS for canonical properties, headers, collections and strings. captured-store.js reads guarded paged columns and creates typed replies; wasm-bindgen copies these into Rust. Native admission and projection then consume the same records. The shared generic validators are not themselves the problem: arena validators directly borrow native data and cost much less in these samples.

## Next coherent primitive

Adopt the private `native-canonical-storage/LLD.md` direction for a separately reviewed prototype: one Rust-owned paged canonical table, direct native CapturedRead/Write/AdmissionRead, InitialWriter directly constructing that table, and a pinned scalar view plus bounded JS observation batches. JS retains real public objects/identity/string ledgers, not a duplicate canonical header/property/item graph. Keep the ready typed Sankey render projection and every admission/failure checkpoint. Native local validation must replace foreign query traffic, not delete validation to make a benchmark pass.

Physical-storage source review found no blocker to the first layer: immutable pinned pages and per-access memory.buffer refresh address nested memory growth; staged accepted/observed/working/history roots preserve failed captures and partial helper errors. Exact visitor and constructor interfaces remain the implementation gate. Public integration consumers must verify full source/raw/apply/helper behavior; internal page unit tests are outside scope.

No grammar rewrite is justified by these profiles. No packet optimization or smaller packed reply removes the dominant repeated storage boundary. No default selector change is authorized by this finding.

## Evidence correction and execution history

First launch exited 1 before creating a browser because the default Playwright cache was absent. It installed nothing and produced zero profiles. Configured retry used the existing `PLAYWRIGHT_BROWSERS_PATH=/home/admin/pele-perf.jL9wzP/browsers` and exited 0. RUN_ATTEMPTS.json records both.

Original summary's JS/WASM partition recognized only wasm:// URLs. Chromium supplies HTTP .wasm URLs here, and shared compiled code can retain the control URL while candidate code runs. `ATTRIBUTION.json`, generated by `analyze-profiles.py`, corrects that classification from unchanged raw profiles. Runtime lane identity comes from the actual exercised operation, never the frame URL prefix. Original summary and executed harness are preserved. The future harness revision fixes classification without rerunning the benchmark or profiler.
