# Consumed source symbols: measured checkpoint

Source `11f27516d98c3d89799a7fe2065eec0bd4e147cf` makes the Sankey source parser borrow lexical fields and consume one owned symbol catalog into the indexed source representation. SourceCanonical shares that catalog; public DTO fields and the grammar remain unchanged. The [contract](architecture/CONTRACT.md) and independent source review describe the removed copies and retained admission stages. No allocation count or speedup is inferred from the source change.

Fresh local validation passes **57 native integrations in 11 binaries**, including the 11,616-source Sankey graph/error corpus; a newly rebuilt macOS arm64 SDK; **Swift 16** with 12,481 drawings, 16,572 expected errors and zero explicit budget differences; **three strict C consumers**; and **168 API conditions across 21 cases**. All 15 driver phases pass. Exact commands, logs, before/after records, immutable Git mappings and independent collection proof are retained. This is new native proof, not reuse of the preceding 817 SDK.

The final native map contains 370 inputs and 402 source/test Git mappings. Aggregate SHA256: `0bd18e5872e7c18e7457807b59ebe5e9ecd160c51f63ecb05213b6da16940408`. Fresh SDK: `c645d3e7e9b13970d16f5940a7a04feabc6bddc9910193f8e6b369a9b6f9ae50`. Consumer: `c8341a87477a455013fc43d04b8a84d69b58f476a2cb0f78ea7ab8917b34d83f`. The prior SDK's 13 files and both protected consumers remain byte-identical. SDK regeneration changes only the expected archive input; source and correctness fixtures remain fixed.

The default WASM package was rebuilt separately: 117 files, WASM `c0ddd411aebbcd3a2293d66cf7de50f8b0c10fafdf335d7554962922ee42f07c`. Node 122, **114 browser tests across Chromium/Firefox/WebKit**, TypeScript and two identical generated initializer outputs pass. The native driver intentionally pinned the preceding web package while it ran; that continuity record is not presented as the later fresh WASM package. All 558 source Git inputs and all 117 package files remain unchanged through completed browser validation. Only the WASM file differs from a4; JavaScript, generated glue and static assets are byte-identical. Actual browser reports and the complete log are retained. Matched warm VM results and bounded parse profiles follow.

## Matched warm VM diagnostic

The run compares `11f2751` against original Rust control `e1486055d596d4e0ac25e4c78f1d5d95a73f781a` and pinned Pele JS: **24 conditions, 21 balanced paired trials each, 1000 ms warmup**, matched Rust batch counts and 36 untimed activation/continuity/ingress probes. Exact package/harness hashes and outputs remain fixed before/after.

All six Sankey parse conditions have lower paired medians than e148 (-34.25% to -4.48%), but remain **6.459-14.268x JS**. All six retained conditions are adverse: **+10.21% to +14.64%**, 0-4 lower pairs out of 21, and **2.064-3.162x JS**. All six Sankey source controls have lower paired medians (-2.72% to -0.75%) but still take **1.004-1.513x JS**.

All six Flowchart controls are adverse: parse +4.36%/+5.95%, retained +6.92%/+3.42%, source +2.29%/+0.61%. The [complete table](RESULTS.md) and raw samples preserve every row, absolute lane medians, paired changes and ratios of independent medians. No adverse control is dismissed as noise.

This is one window versus e148, whose behavior predates the callback fixes. It does not isolate this layer against a4 across separate windows and does not establish allocation savings. No first-use or complete release-gate rerun occurred; the historical complete gate remains 126/126 failing. Full API and performance parity remain unproven. No 0.3 release.

## Parse profiles

Four fixed medium/large parse profiles with a five-second target compare `11f2751` and e148. Each sampled operation includes public parse, a type checksum and disposal. The large candidate loop elapsed 7580.785 ms and its raw CDP span was 7876.194 ms; the target is not an exact window duration. Exact model identity/descriptor/IEEE digests, SVG/dimensions and resource traces match pinned Pele before/after. All source/package/harness hashes remain fixed; the first attempt passes strict nonnegative-weight validation with no retry. The executed four-window program is explicitly adapted and pinned; it is not claimed byte-identical to the older nine-window harness.

The largest named self frame is `publish` in the source initializer: 18.43% medium / 34.04% large. Its body appends the object and inserts object-to-ID and object-to-owner entries; record construction is in its caller. GC self shares are 12.09% / 32.27%, separate from function self samples. These observations establish neither allocation counts nor an avoidable-cost ceiling. Inlining and the large candidate GC denominator limit comparisons.

Initializer/publication inclusive shares are 25.36% / 38.18%, overlapping the publish self frame. Deduplicated native grammar-plus-symbol shares are 10.70% / 6.19%; native admission/export 12.38% / 5.22%; JS layout decode 8.61% / 2.79%; disposal 1.62% / 0.75%. These groups overlap and must not be added. The data do not demonstrate redundant full admission walks. Classifiers anchor actual callable owners rather than matching `SourceSymbols` or `IndexedSankey` inside generic type names. See the [complete attribution](profile/RESULTS.md) and raw profiles; warm timing, not profile throughput, supplies latency evidence.

```sh
python3 docs/benchmarks/2026-10-21-source-symbols/verify.py
python3 docs/benchmarks/2026-10-21-source-symbols/verify-profile.py
```

The portable verifier reads data only. Original and stored SHA256 hashes bind exact deterministic gzip bytes. No binaries or SDK archives are distributed here. Full API parity, first-use performance and release gates remain unproven. No 0.3 release.
