# Native canonical storage: rejected first runtime

Source `3166b3dd18569ad0bc20f173a8941dd628609ddc`, experimental selector only. Moving the sole canonical store to native memory removed the rejected JS-column import architecture, but this first implementation remains slower than the matched arena control. It is not activated for release.

The control and candidate contain identical **113-file packages**, WASM `2310934dd43dc52c20723c3e0217377f9819fef8e146d03ef3a33845e06bdcce`. Only the internal selector differs. All 18 latency rows retain their 21-trial samples, warmups, batch counts and negative controls in `comparison.json`. Both package maps match those of the six CPU profiles exactly. The data-only verifier checks 48 benchmark probes and 24 profile probes: candidate Sankey models actually use native storage; controls do not.

| Stage (six size/resource conditions) | Candidate / arena median range | Paired change range | Candidate / pinned JS |
| --- | ---: | ---: | ---: |
| Parse | 1.853–2.255× | +83.61% to +127.63% | 21.11–28.42× |
| Retained render | 2.174–2.253× | +117.32% to +126.97% | 4.65–7.33× |
| Source render control | 0.984–1.032× | −1.23% to +3.65% | 1.38–1.88× |

The positive source-control row remains visible; no cross-window comparison to the previous JS-column backend is used as acceptance evidence. No first-use benchmark was run for this rejected baseline.

## Actual CPU attribution

One authorized VM run produced six raw profiles: medium/fixed, parse/retained/source × arena/native, 2 seconds each at 250µs sampling, with untimed native-storage and exact-output checks before/after. All raw files are retained losslessly as `.cpuprofile.gz`. No forced GC or production instrumentation. The run exited 0; no failed launch preceded it. Profiling throughput is not a latency benchmark.

| Native candidate | JS self | WASM self | GC self | Refresh + buffer getter disjoint self | Full publication inclusive |
| --- | ---: | ---: | ---: | ---: | ---: |
| Parse | 49.68% | 30.65% | 4.72% | **41.86%** | 57.72% |
| Retained render | 52.44% | 26.83% | 0.12% | **53.73%** | 34.08% |

Retained matching adds an overlapping 31.46% inclusive group. Native admission is 6.54% inclusive for parse and 5.49% for retained render. The Sankey parser entry group is 3.77% of candidate parse samples. Native flat storage is 23.50% inclusive for parse and overlaps admission/construction. These percentages are weighted sampled attribution, not predicted removable cost; inclusive groups must not be added.

The concrete hot path is `bindings/web/native-storage.js:32`: `#word` calls `#refresh` itself and again through each of two `#range` checks. Header/property/item queries repeat those checks for each scalar; UTF16 decoding repeats them per code unit. Native `memory.buffer` getter self accounts for 12.13% parse and 15.69% retained, entirely below this refresh path. The scalar `#refresh` function itself accounts for 29.73%/38.04%. These disjoint leaves exactly equal the deduplicated refresh ancestor group. The source-render negative controls have zero native storage/refresh/publication attribution.

The next coherent layer is an immutable per-epoch POD row cursor: copy a complete validated header/property/item/string descriptor into fixed scalar registers during one synchronous refreshed read window. Recheck the requested row on each accessor; copied values survive callbacks and memory growth. Decode UTF16 using page-bounded copied scratch before any string-construction callback. Keep the shared traversal, full restoration, native admission and native canonical store unchanged. A newly read native row must refresh; a cache hit reads only immutable copied scalars. This targets the evidenced access boundary without introducing a graph mirror, delaying work or treating callbacks as pure. It still requires fresh E2E and matched old-native/new-native plus arena benchmarks.

The source-render profiles remain predominantly native rendering/formatting; that separate path is not bundled into this storage-read experiment.

## Reproduction and provenance

Run `python3 summarize.py` locally for data-only verification and regenerated `paired-summary.json` / `ATTRIBUTION.json`. The original executed `profile-summary-original.json` stays unchanged. Supplemental attribution normalizes Rust crate hashes so native wrapper symbols are recognized; the original native-wrapper regex missed hashed crate names. Its language categorization already correctly recognizes both `wasm://` and HTTP `.wasm` URLs. V8 may use the control module URL for shared compiled code; runtime identity comes from the sampled operation and validated probes.

`PROVENANCE.json` verifies all **586** frozen source hashes against Git `3166b3d`. The earlier build-source record named pre-integration HEAD `39338ff5`; its exact source blobs, including then-uncommitted files, match the final commit. Package maps, source/case/module/harness hashes, browser/host metadata and run IDs are retained. No full browser/Swift result is inferred from this profile bundle; those are separate validation evidence. `index.json` hashes every included file and records raw uncompressed profile hashes.
