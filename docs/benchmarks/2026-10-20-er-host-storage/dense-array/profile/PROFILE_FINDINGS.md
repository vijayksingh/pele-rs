# Dense770 six-profile attribution

Data-only independent reanalysis of existing 250µs/1500ms CDP profiles. Full runtime hash map matches the dense comparison candidate, including WASM `6567d1a9ca51488928fd5940c7dc53afb85038c181d6278b84f55fcb0d4a1f8e`. No new profile, browser, build or timing run. Source-level record-layout results are archived separately.

Self time weighted by timeDeltas; categories are mutually exclusive. WASM includes Rust, standard-library allocator and generated glue, not only grammar. JS includes harness/facade. Native V8/idle/unattributed samples remain separate. Percentages describe different timed workloads, not relative execution speed.

| Profile | JS | WASM Rust/runtime/glue | GC | Boundary shim | Other |
| --- | ---: | ---: | ---: | ---: | ---: |
| parse-js | 72.60% | 0.00% | 21.50% | 0.00% | 5.90% |
| parse-wasm | 53.79% | 31.55% | 11.60% | 0.50% | 2.57% |
| render-model-js | 93.92% | 0.00% | 2.16% | 0.00% | 3.93% |
| render-model-wasm | 31.89% | 59.38% | 0.47% | 2.47% | 5.80% |
| render-source-js | 93.11% | 0.00% | 3.94% | 0.00% | 2.95% |
| render-source-wasm | 15.18% | 73.25% | 0.24% | 1.75% | 9.58% |

The parse profile has **31.55% WASM self time**, despite JS allocate alone drawing attention at 26.62%. Largest native parse families by whole-profile self time: hash tables/SipHash 6.55%; HostConstruction methods 6.07%; ER lexer 4.48%; Rust allocator 2.78%; compact atom encoding 1.59%; formatting 1.26%; ER parser 0.97%. These self categories identify separate function bodies. Their inclusive variants overlap and must not be added.

Dominant individual native bodies: HostConstruction::string 3.74%; lexer::tokenize 2.66%; RandomState::hash_one<&str> 1.95%; SipHasher::write 1.70%; construction_atom 1.59%; dlmalloc::malloc 1.48%; hash_one<&String> 1.20%; Scanner::kw 0.96%; string pattern search 0.87%; dlmalloc::free 0.85%. Exact symbols and all six top-25 lists are in six-profile-shares.json.

Ancestry classification of hash **self samples only** attributes 4.68% overall to HostConstruction::string, 1.05% to Direct::ensure_entity outside the string interner, and 0.83% to other callers. HostConstruction::string own self plus these hash descendants account for 8.42% whole-profile time without double counting. Its much larger inclusive time also contains frame flush/JS publication, and must not be mistaken for Rust string processing. This is stronger evidence for examining redundant interner work than the 2.44% JS text decoder self figure alone. It does not justify replacing collision-resistant hashing or moving budget/error order. No next implementation is bundled.

Ownership union is still small: adoption plus host_parsed plus any sampled native handle free is 0.69903% inclusive, ancestry-deduplicated. __wrap is inside host_parsed and ownRawModel inside adoption. Detached GC samples cannot be assigned to those handles; no sampled native-free frame does not prove free cost zero. Policy-sharing proposal stays parked.

JS oracle parse GC is 21.50%, versus 11.60% for WASM parse. This does not mean the WASM lane allocates less or spends less absolute GC time: profiles perform different operation counts over roughly equal durations. Retained/source rendering spends most self time in WASM; improvements to parse do not imply retained-render improvements.

Artifacts: profile-shares.py performs read-only JSON arithmetic; six-profile-shares.json retains metadata, exact profile hashes and top native functions; hash-self-callers.json isolates interner ancestry; profile-attribution.json binds ownership scopes. No performance improvement or memory claim follows from this attribution.
