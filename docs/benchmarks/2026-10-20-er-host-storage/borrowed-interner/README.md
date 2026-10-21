# Borrowed ER arena string interning — unreleased

Runtime **`d25bb26089e597b9de8985edbfced5a3955a6bbd`**. This layer uses a borrowed short-string lookup while constructing an ER native public arena. Repeated strings can reuse their canonical ID without first allocating an owned UTF16 vector. The existing 64-byte UTF8 eligibility boundary and long-string fallback remain; generic conversion, canonical order, raw UTF16, limits and provider behavior are preserved. No persistent index, lazy conversion or new ABI is introduced.

**Measured scope is native public construction and Swift mutable parse.** Ordinary host-owned ER web parsing uses a different interner and does not execute this helper. All 18 VM conditions are negative controls, not evidence of a web improvement. No 0.3 release or JS parity claim; no new full performance release gate was run.

## Results and controls

| Affected measurement | Control | Candidate | Paired change | Faster pairs |
|---|---:|---:|---:|---:|
| Native ER medium public parse | 0.182865 ms | 0.161945 ms | **−10.59%** | 15/15 |
| Swift ER medium mutable parse | 0.201787 ms | 0.178163 ms | **−12.00%** | 15/15 |

Paired changes are medians of trial ratios, not ratios of separate medians. Native Node-JS parse was 0.208289 ms for this one medium synthetic source: candidate/native-to-JS ratio **0.7775×**. That is a local native consumer result, not browser or universal-family parity.

Native comparison retained all eight rows: two medium families × public parse, immutable parse, source render and retained render. Fifteen trials, 1000 ms warmup, fixed batch 16, separate runtime timers and balanced three-way cyclic/reverse order. Native negative controls range −1.28% to +0.31%; Flowchart immutable parse and retained rendering are adverse. Full samples and raw nanosecond responses remain in [native/native-comparison-attempt1.json](native/native-comparison-attempt1.json).

Swift uses one complete strict-guard window, six conditions × 15 pairs, **90 pairs / 45–45 first order**, 1000 ms matched warmup and batch cap128. Frozen record-layout control and candidate differ in exactly **two Rust input files plus the SDK archive**; unlike the earlier record comparison against 0b, this isolates this layer. ER lifetime is −2.44% (10/15), immutable parse −0.26% (8/15). Sequence controls are +0.48% mutable parse, −0.63% lifetime and +0.79% immutable parse. No retries, rejected windows or pooled samples.

The short VM diagnostic has 18 rows ×9 trials and 500 ms warmup. ER medium parse is **+2.80% fixed /+2.74% Canvas**, both only 2/9 faster pairs. Across all VM controls, paired changes range −6.21% to +2.80%; six rows are adverse. These results are retained without dismissing them as noise or crediting unaffected paths to the helper. One browser/page shares heap, GC and cache pressure across independent WASM instances and pinned JS. This diagnostic has no per-row process quiet guard and is not a replacement release gate.

[RESULTS.md](RESULTS.md) and [summary.json](summary.json) retain **every native, Swift, VM and allocation row**, including controls and counts. [web-comparison.json](web-comparison.json) preserves all raw VM samples and package hashes.

## Allocation evidence

Separate instrumented consumers measure allocation requests; elapsed timing uses the ordinary binaries. ER medium public parse at count1:

| Counter | Control | Candidate |
|---|---:|---:|
| Allocations | 2501 | 1987 |
| Reallocations | 584 | 109 |
| Total requested bytes | 278808 | 263576 |
| Peak additional requested bytes | 119594 | 118710 |
| Live-byte delta after operation | 0 | 0 |

Requested traffic decreases 15,232 bytes; peak decreases **884 bytes**, not 15,232. These are allocator requests and high-water accounting, **not RSS**. Each operation drops its parsed document, so zero retained live delta does not mean the model itself has no storage. Counts 1 and 16 are both preserved. All eight negative-control allocation rows are exactly identical. One repeated-string medium fixture is not a general memory or latency distribution; unique-heavy/threshold cases are correctness coverage, not separately timed workloads.

[allocation-comparison.json](native/allocation-comparison.json) retains request commands, complete replies, exact output checks and counter differences. [allocation-review.json](native/allocation-review.json) and the archived design manifest were written before later validation; their “next gate/not run” wording is preserved as historical, not the final status below.

## Fresh validation and provenance

Fresh d25 validation: **27 native integration tests**, including the 17,437-source ER fallback corpus and borrowed-string public slice; **27 browser checks** across Chromium, Firefox and WebKit; **15 Swift integration cases**, including 7535 ER mutable drawings and 9902 errors with zero explicit budget differences; **168 Swift API conditions** over 21 workloads; **three strict C consumers**. Browser finished 27/27 in 19 seconds. The earlier 390/390 suite belongs to **bd7df0d**, not this runtime; it is not presented as a new full-suite result here.

SDK build, one standalone consumer build, focused checks and timing logs are compressed under [logs](logs). Three record-publication browser reports are retained under `tests/`; repeated attachment copies and large unrelated reports are excluded. `swift/correctness.json` records the earlier point at which timing was still held. Completed timing and post-run freeze supersede that field without rewriting its history.

[source-package-proof.json](source-package-proof.json) checks native consumer inputs against immutable Git blobs and rehashes both frozen timing/allocation binaries. Native builds occurred before the commit, so their original manifests honestly retain `31edd859` plus WIP; full candidate source aggregate matches d25, control matches 31edd859. Their native input aggregates use the native harness's byte-stream algorithm and differ from the Swift aggregate, which also includes FFI, SDK and consumer inputs.

The 338 tracked Swift inputs match d25 Git. All 351 Swift input hashes, including 13 SDK files, were frozen before timing and checked again afterward; candidate aggregate **`e9be1b27738482738b6c9f08fa98d4e637da73f70f31ee2ed4026b0636fefc73`**, binary **`2131d6fd1090d375e71d938d7b1dd799a03b319b84f9c45ea50651916593de3c`**. The frozen local package's 106 files match the VM report; its control map matches the preceding record-layout candidate. No independent physical remote-directory rehash or reproducible rebuild was performed for this packaging step. The later working tree and SDK are not used as d25 proof.

Native harness source hashes identify the strict compiler checks before warmup and every trial, plus source/binary/output checks before/after; individual native guard snapshots are not retained. Swift stores 180 measured-request guard snapshots, all checked for compiler/Pele-test activity. VM saves hash/output checks and host snapshots but no per-row process guard. Successful assertions provide the indicated scope, not proof of absence of all environmental contention.

## Recompute or reproduce

Recompute packaged arithmetic and deterministic index only; no source, build, browser, network or measured workload:

```sh
python3 docs/benchmarks/2026-10-20-er-host-storage/borrowed-interner/summarize.py
```

Original focused native command shape, with frozen artifacts available locally:

```sh
node --expose-gc scripts/perf/compare-native.mjs --quiet-run \
  --control dist/perf-patches/parse-architecture/er-borrowed-interner/control-native/pele-perf \
  --candidate dist/perf-patches/parse-architecture/er-borrowed-interner/candidate-native/pele-perf \
  --cases flowchart/medium,er/medium --stages parse-public,parse,render-source,render-retained \
  --trials 15 --warmup-ms 1000 --batch-count 16 --output <new-native-report.json>
```

Swift comparator uses the original unchanged manifests; the control directory supplies its frozen binary override:

```sh
node scripts/perf/swift/compare.mjs --quiet-run \
  --control-dir dist/perf-patches/parse-architecture/er-record-layout/candidate-swift \
  --candidate-manifest dist/perf-patches/parse-architecture/er-borrowed-interner/candidate-swift/swift-build-manifest.json \
  --cases er/medium,sequence/small --stages parseMutable,lifetime,parse --order-offset 0 \
  --output <new-swift-report.json>
```

VM diagnostic command shape, on the VM with pinned dependencies and both frozen package directories:

```sh
node scripts/perf/compare-web-vm.mjs \
  --control parse-architecture/er-record-layout-first/package \
  --candidate parse-architecture/er-borrowed-interner-first/package \
  --cases sequence/small,flowchart/medium,er/medium \
  --profiles fixed,canvas --stages parse,render-model,render-source \
  --trials 9 --warmup-ms 500 --output <new-web-report.json>
```

These are execution commands requiring matching binaries, SDK, pinned oracle and coordinated quiet; they are not run by the data summarizer. Historical manifests preserve original paths. No binaries, runtime packages or duplicated runtime source are committed in this evidence folder. `evidence-index.json` binds every other bundled file, including decompressed hashes for deterministic gzip files.
