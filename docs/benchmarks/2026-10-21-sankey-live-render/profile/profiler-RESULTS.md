# Retained Sankey live-read profile

Frozen candidate8175006345f3b66a6c9fd942814ed0ba5ff3014d versus e1486055d596d4e0ac25e4c78f1d5d95a73f781a. Four serial5-second CPU windows,250µs samples, medium/large fixed measurer. Parsing and disposal outside windows;500ms warmup. One browser. Full SVG/dimensions and ordered callback traces match pinned Pele before/after each case. No prototype interception. All package/harness/oracle hashes unchanged; browser closed.

| Case/lane | text decode self | live-read decode self subset | prepare input inclusive | snapshot inclusive | charCodeAt self |
|---|---:|---:|---:|---:|---:|
| sankey-medium-fixed-render-model-control | 7.81% | 0.00% | 9.17% | 0.00% | 0.00% |
| sankey-medium-fixed-render-model-candidate | 36.14% | 33.61% | 24.44% | 11.02% | 0.06% |
| sankey-large-fixed-render-model-candidate | 37.95% | 35.06% | 23.97% | 11.15% | 0.11% |
| sankey-large-fixed-render-model-control | 7.33% | 0.00% | 10.37% | 0.00% | 0.00% |

The candidate’s dominant added work is the string-command boundary. `crates/pele-wasm/src/detached_render.rs:30` creates a new JS operation string in every Reader::call; `get` also transfers a key, and `require_string` transfers method and expression receiver. Generated `pkg/pele.js:2296` decodes each through TextDecoder. Raw stacks tie33.61%medium/35.05%large of all weighted leaf samples specifically to these live Reader/HostPhase paths. This is not recipe decoding. Result-factory decode and width-callback decode are separate smaller subsets.

Recommend one generic live-read ABI layer: numeric operation codes plus job-bound immutable key/method handles, optionally ordered GET/index segments whose execution preserves exactly the Rust-selected operations and stopping/error order. Keep original receiver identities and every callback boundary; do not cache field values across callbacks or move conditionally-read fields earlier. Rust remains responsible for geometry, coercion choices and phase ordering. This removes repeated control-string transport without suppressing live reads. Bulk UTF16 transfer is not the primary sample-supported target here (explicit charCodeAt leaf0.06–0.11%); do not predict gains from that cost alone.

A second cost remains: `bindings/web/full.js:214` deliberately clears confirmedGraph after readonly completion; `prepareMutableInput` at221 consequently snapshots/applies on the next retained call. Candidate prepare inclusive24.44/23.97% overlaps snapshot11.02/11.15%, native admission and source-layout evaluation. It cannot be removed merely because rendering is readonly: callbacks and result factories can mutate the public graph. Any separate change needs a proof of exact observation/accepted-state semantics.

Self categories partition weighted samples. Ancestor categories deduplicate each sample within that category, but overlap one another and must not be added. Inlining hides some exact methods. The inherited automatic summary `native-parser` regex matches IndexedSankey type names in control render symbols; ignore it. Corrected data-only attribution is `runtime/attribution.json`; there is no parser in these windows. No allocation-count or speedup ceiling inferred.

Untimed resource callback counts:
- sankey/medium before: {'js': 48, 'control': 48, 'candidate': 48}
- sankey/medium after: {'js': 48, 'control': 48, 'candidate': 48}
- sankey/large before: {'js': 200, 'control': 200, 'candidate': 200}
- sankey/large after: {'js': 200, 'control': 200, 'candidate': 200}

Internal operation invocation counts are unavailable without new instrumentation; CPU sample counts are not call counts. No extra counting experiment was run. Raw four profiles, full SVGs/traces, before/after hashes and executed source are retained. No first-use run.
