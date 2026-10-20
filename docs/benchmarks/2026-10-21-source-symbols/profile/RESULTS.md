# Source-symbols parse profile

Four valid5s CDP windows, medium/large fixed, candidate11f27516 versus e148. Public parse + type checksum + dispose only inside profile.500ms warmup;250us sampling. Fresh identity/descriptor/IEEE model digest, exact Pele SVG/dimensions/width trace passed before and after both cases. All117/113 package maps, source, oracle, cases and executed harness verified before/after. Browser/engines closed; VM inventory empty. No first-use or extra warm run.

Weighted self language partitions (%):

| Window | JavaScript | WASM | GC | Engine/profiler |
|---|---:|---:|---:|---:|
| sankey-medium-fixed-parse-control | 42.42 | 40.52 | 14.92 | 2.14 |
| sankey-medium-fixed-parse-candidate | 43.73 | 40.61 | 12.09 | 3.58 |
| sankey-large-fixed-parse-candidate | 43.59 | 19.84 | 32.27 | 4.30 |
| sankey-large-fixed-parse-control | 41.78 | 40.57 | 13.98 | 3.67 |

Candidate medium / large inclusive shares (overlap; not additive):

| Actual callable phase | Medium | Large |
|---|---:|---:|
| native-grammar | 4.67% | 2.79% |
| native-symbol-builder | 6.03% | 3.40% |
| native-source-entry | 26.83% | 14.71% |
| native-admission | 4.83% | 1.94% |
| native-layout-export | 7.54% | 3.29% |
| native-logical-read | 3.93% | 2.07% |
| js-layout-decode | 8.61% | 2.79% |
| js-publication | 25.36% | 38.18% |
| wasm-output-copy | 2.31% | 0.92% |
| deduplicated union: native-grammar-symbols | 10.70% | 6.19% |
| deduplicated union: native-admission-export | 12.38% | 5.22% |
| deduplicated union: js-decode-publication | 33.97% | 40.98% |
| deduplicated union: disposal | 1.62% | 0.75% |

Largest specific self frame is `publish` in `bindings/web/source-initializer.js:57`:18.43% medium /34.04% large. Its body appends the object and inserts the object→ID and object→owner entries. Record/array/Map construction is in the caller. Samples therefore do not establish that record construction, catalog decoding or native grammar explains this self cost. Inlining remains a limitation. GC self12.09%/32.27% is disjoint from ordinary function self samples; no allocation count or object-retention conclusion follows.

Source mapping: `indexed.rs::SourceSymbols::{intern,finish}` owns consumed catalog construction; `parser.rs::Reader`/prepare/tokenize are grammar. `source_store.rs::SourceCanonical::layout_packet` and `pele-wasm/src/source_layout.rs::prefix` form native export; `binary.rs::Writer` runs underneath export. `captured_admission::InitialObserver`, `captured_transport::Counter` and existing schema validation are observed admission work. Some fused work is inlined into Codec::try_parse_public, whose inclusive share already includes subphases. JS `layout-storage.js` readText/readColumns and decodeSourceLayout are decoding; `source-initializer.js` eligible/tryInitialize/publish are initial publication. Disposal is only1.62%/0.75% deduplicated ancestry.

The current source still has one fused initial admission traversal; this profile does not justify treating receipts as repeated validation passes. Large candidate GC changes the denominator substantially. Do not infer absolute gain from cross-window percentages, compare profile throughput as a benchmark, sum overlapping groups, or claim a speed ceiling. Current warm benchmark remains authoritative for latency. Parser classifier anchors actual callable owners: generic type mentions of SourceSymbols/IndexedSankey in hash/iterator/render frames do not classify them as grammar.

Raw profiles and exact executed inputs: `runtime/fetched/profiles/` and `runtime/fetched/files/`. Recompute offline with `python3 analyze.py`; output `runtime/attribution.json` includes all raw frame names, disjoint leaf roles and deduplicated ancestry. Strict finite/nonnegative time-delta validation passed on first attempt; no dropped/clamped samples or retry.
