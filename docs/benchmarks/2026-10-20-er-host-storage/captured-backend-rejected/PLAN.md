# Captured Sankey architecture attribution

Prepared only. Parent must explicitly authorize execution after the current VM benchmark ends. This folder performs no run by itself. CPU sampling is diagnostic; matched ordinary benchmark timings decide adoption.

## Question and lanes

Use identical immutable copies of the feature package in separate control and candidate directories. The script verifies every file hash matches, not only WASM. Both module graphs/WASM instances load in one browser page. The control internal flag is false before its renderer is created; candidate flag is true before its renderer is created. Before and after each condition, retained and fresh public models must report undefined/control and source/candidate. Whole SVG bytes and exact dimensions must match pinned JS in all three retained/fresh/source lanes. Direct source rendering can be included as a negative control without selecting a captured model.

Primary bounded run: Sankey medium, fixed measurer, parse and retained render, one 1.5-second CPU window per backend (four profiles). If the VM result has resource-dependent regressions, repeat the same conditions with canvas or the affected scale rather than profiling every family. Optional repetition changes only order: control/candidate then candidate/control. All diagnostic settings are explicit. No helper-stage invention.

After GO, integrate the new script beside existing cases.mjs, then use:

```sh
node scripts/perf/profile-captured-sankey.mjs \
  --control /absolute/frozen-arena-copy \
  --candidate /absolute/frozen-captured-copy \
  --cases sankey/medium --stages parse,render-model \
  --profiles fixed --duration-ms 1500 --warmup-ms 300 \
  --repeats 1 --sampling-interval-us 250 \
  --output /absolute/captured-profile
```

An affected fixed/canvas comparison can use `--profiles fixed,canvas`; source negative control can add `render-source`. Total requested CPU windows plus warmup are capped at 120 seconds and a 240-second browser deadline bounds the whole run. No forced GC, heap snapshot, production monkey-patching, or allocation instrumentation changes the executed implementation.

## Source-backed architecture questions

1. **Native admission and transport.** Inspect `pele::compat::public::captured_admission`, `captured_transport::{admit_observed,admit_outbound,canonical_frame_bytes}`, `pele_wasm::captured_store`, and JS captured-store imports. Distinguish repeated native checks from the JS arrays and WASM copies they trigger. Inclusive parent and child costs overlap: do not sum native admission + storage adapter + imports as independent avoidable work.
2. **Projection versus actual parsing.** Separate Sankey `parser::{parse,prepare,tokenize}` from `public_model::{imported_from,string,array,index,normalized_source}`. Captured source should use its eager normalized projection; retained edits legitimately refresh projection. Sampling under importer ancestors can identify whether unchanged render accidentally reimports. A missing symbol may be inlined; zero named samples do not prove absence.
3. **Flat physical storage.** Examine captured-graph.js `#page`, `fork`, `copy`, `compact`, string indexes, header/property reads, and typed-array engine frames. The report provides a broad flat-storage group and a narrower named fork/page/copy/compact group. A page accessor includes read work as well as allocations; samples cannot count allocated pages. JS heap metrics around each sample window are shared-process occupancy, not allocation totals or retained-owner proof.
4. **Initial construction/publication.** Attribute native `initial_into` and `host_construction`, JS constructor decoder, then `reconcileGraph`. Current construction fills columns followed by one full initial publication walk. Compare it with arena snapshot/decode/reconcile. Do not describe the initial walk as removed, or infer an achievable speedup by adding overlapping groups.
5. **Retained observation/restoration.** Attribute `matchesGraph`, `snapshotGraph`, owner `sameGraph/capturedObservedFrame/outcomePacket`, and `reconcileGraph`. Full restoration after callback edits is mandatory. A profile can motivate a more direct shared primitive or avoidable duplicate scan; it cannot justify skipping ownership or restoration.

## Evidence produced

- Full `.cpuprofile` for every case/resource/stage/backend/repetition, SHA256 in summary.
- Full input package hashes before and after; WASM, owner module, harness, cases and pinned oracle hashes; source hash/bytes, browser, CPU, settings and host load/memory.
- Backend/output proofs before and after CPU windows. Probes and SVG hashing are outside profiling. Warmup occurs outside CPU windows and its operation count is recorded.
- Weighted self and inclusive rankings keyed by full frame identity (name/URL/line/column/script). Inclusive recursion is deduplicated once per sample. Architecture groups are likewise unioned over each sample's ancestor chain. Raw profiles retain all frames beyond the summary top 60.
- Disjoint self buckets for WASM/JavaScript/GC/engine-profiler, plus explicit overlapping architecture-group percentages and exact matcher patterns.
- Controlled loop operations/checksum/elapsed time, plus raw JS heap occupancy before/after. These are not ordinary latency results.

Exact phase invocation counts are unavailable from current public APIs without modifying execution. `phaseCallCounts:null` explicitly records that gap. CDP sample counts and hit counts must never be presented as function-call counts. A repeated-validation hypothesis needs source-call-path evidence alongside inclusive samples. No production wrapping or fake count estimate is introduced.

## Decision after profiling

Identify the dominant repeated representation/traversal boundary using weighted inclusive stacks and raw ancestor inspection. Preserve actual parser/renderer work and mandatory observation/admission/ownership contracts. Propose one coherent primitive change with exact checks that remain and one measurable removed allocation/traversal path. Then rerun ordinary matched VM timings before any new adoption claim. This profiler itself provides no speedup forecast.
