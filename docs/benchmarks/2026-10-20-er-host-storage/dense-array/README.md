# Dense ER array publication — unreleased

Frozen runtime **`770113546dd302a60b52707194bbda40bb9e070c`**, default optimized WASM build. A bounded dense-array publication event sends consecutive values directly to the host constructor, avoiding generic property records for eligible fresh arrays. Canonical decimal-key interning, node/string order, descriptors, identity indexes and admission accounting remain eager. Oversized arrays use the existing generic path. The native default sink still constructs the same graph through its existing property/node interface.

This follows the borrowed-interner layer `d25bb26`. Later single-index experiments are excluded. **No 0.3 release, JS parity or regression-free claim.** No new full performance release gate ran.

## Web results and adverse controls

Ordinary VM comparison: 24 rows, 21 trials, 1000 ms warmup, matched WASM batch counts, fixed and Canvas resources. All six ER parse rows improve **3.82–10.24%** in paired medians, with **15–20/21** faster pairs; candidate still costs **4.52–9.96× pinned JS**.

| ER parse | Fixed | Canvas |
|---|---:|---:|
| Small | −5.12% | −6.42% |
| Medium | −9.17% | −8.42% |
| Large | −3.82% | −10.24% |

The ordinary **ER medium Canvas retained-render control is +4.64%, only 6/21 faster pairs**. ER small Canvas source is +0.28%; Sequence fixed/Canvas source controls are +2.69%/+2.00%, and Sequence Canvas retained is +0.53%. All controls remain in the raw report and complete results; they are not subtracted or dismissed as noise.

Separate first-use comparison: 36 rows, 15 trials, 1000 ms warmup, same frozen packages. All 30 ER paired medians are lower in this run:

| Complete ER operation | Paired range | Faster pairs across rows |
|---|---:|---:|
| Source render control | −5.71% to −0.37% | 9–15/15 |
| Parse and first unchanged render | −3.64% to −0.57% | 8–13/15 |
| Parse and first edited render | −4.18% to −0.85% | 9–14/15 |
| Parse and first method invocation | −23.09% to −2.54% | 9–12/15 |
| Parse and first callback render | −3.64% to −0.36% | 9–13/15 |

Sequence Canvas first render is **+2.25% (4/15)** and source **+0.70% (5/15)**; other Sequence first-use rows are lower. Complete first-use costs include fresh parse, the first operation and disposal; this is not an isolated method/layout microbenchmark. Different runs' source controls need not agree. The table does not establish a universal causal gain.

[RESULTS.md](RESULTS.md) and [summary.json](summary.json) retain all 24 ordinary, 36 first-use, six Swift and ten allocation rows. Raw reports are compressed as [comparison.json.gz](comparison.json.gz) and [first-use.json.gz](first-use.json.gz). Changes are medians of paired trial ratios; JS comparisons are ratios of separate medians. Neither is a confidence interval. Both VM diagnostics share one browser/page across independent WASM instances and pinned JS, with shared heap/GC/cache pressure, serial warmup and no per-row process quiet guard. Exact outputs and package hashes were checked before and after by the hash-identified harnesses. These diagnostics do not replace the release gate.

The subsequent [single-string-index experiment](rejected-index/README.md) is
rejected. It passes 84 browser checks and removes duplicate metadata, but three
ER parse rows regress, including small Canvas +5.25%. Its isolated package,
24-row comparison and unchanged native-input proof remain archived. Production
retains the preceding two-index implementation; no first-use run was performed
for the rejected experiment.

The [computed-property record-literal experiment](rejected-literal/README.md)
also passes 84 checks but regresses every ER parse condition by 8.84–11.74%.
It is rejected and restored to this dense runtime. All 24 rows and package
proof remain archived; no first-use run followed that regression.

[Six saved dense-control profiles](profile/README.md) partition ER parse self
time into 53.79% JavaScript, 31.55% WASM/runtime, 11.60% GC and remaining glue or
unattributed samples. They identify separate host and native costs; they do not
measure improvement or directly explain the rejected candidate's regression.

## Native and Swift controls

One complete guarded Swift window compares the frozen d25 consumer with dense-array: six conditions ×15 pairs, **90 pairs with 45/45 first order**, matched 1000 ms warmup and batch cap 128. Native source inputs differ only in `er/public_model.rs` plus the rebuilt SDK archive. All results are retained:

| Case / API | Paired change | Faster pairs |
|---|---:|---:|
| ER mutable parse | **+0.65%** | 5/15 |
| ER lifetime | **+0.64%** | 5/15 |
| ER immutable parse | −0.49% | 8/15 |
| Sequence mutable parse | +0.10% | 7/15 |
| Sequence lifetime | −0.33% | 10/15 |
| Sequence immutable parse | +0.87% | 4/15 |

No native speedup claim. The complete attempt1, host snapshots, per-request guards and exact before/after responses are preserved. All 180 measured guard snapshots reject compiler/Pele-test activity, including idle compilers. No retries, pooled windows or guard bypasses.

Separate native allocation consumers tested counts 1 and 16. **All ten rows and every allocation metric are identical** to d25. ER medium public parse count 1 remains 1987 allocations, 109 reallocations, 263576 requested bytes, 118710 peak additional requested bytes and zero live-byte delta after dropping the result.

The caller's input Vec now lives until native node publication returns. Exact graphs and identical medium-fixture counters do **not** prove unchanged peak memory for every input. These are allocator-request/high-water counters, **not RSS**. Counts 1/16 share a synthetic medium fixture; no general heap or latency distribution is inferred. No native elapsed comparator was run for this layer.

## Validation and immutable evidence

Fresh checks: **27 native integration tests**, including exact default-arena graphs and the 17,437-source ER corpus; **84 browser checks across three engines (48.6 seconds)**; TypeScript consumer checks; **15 Swift integration cases**, including 7535 ER drawings and 9902 errors with zero explicit budget differences; **168 Swift API conditions** across 21 workloads; **three strict C consumers**. The earlier 390-check full browser suite belongs to bd7df0d, not this runtime. SDK and one standalone consumer were freshly built; three expanded oversized-publication browser reports are included, without duplicate attachments or the full unrelated report set.

Build/test logs are in `logs/`; all raw JSON with substantial size is gzip-compressed. Archived static review/manifest predate execution and keep their original “not run” statements. `swift/correctness.json` likewise predates timing and records the earlier hold. The later complete report and post-run freeze are the final timing evidence.

[freeze/commit-binding.json.gz](freeze/commit-binding.json.gz) binds all **658 frozen source-file hashes** to immutable Git 7701135. The source/package freeze and [source-commit-proof.json.gz](source-commit-proof.json.gz) also bind all 338 tracked Swift inputs, native input aggregates, frozen native timing/allocation binary hashes, 106 retained package files and executed harness hashes. The native build manifest and Swift report preserve their actual precommit `be7b206` plus WIP provenance; source hashes now bind to 7701135. No later worktree or next experiment is substituted.

Swift source/SDK aggregate **`439fe2e6a023eebcafb980508b447162cdd5cd9c07519200df47f5ab3fcaea28`**; candidate binary **`554ef37b648587962511c5e9eaa7ef639c469d87be601996817a1c9d762957fa`**. Full source/SDK/binary hashes were checked before and after timing. Frozen local package hashes match both VM reports; control hashes match the prior borrowed-interner package. Packaging does not claim an independent physical remote-directory rehash or reproducible rebuild. No runtime source, binaries or full packages are duplicated here.

## Recompute

```sh
python3 docs/benchmarks/2026-10-20-er-host-storage/dense-array/summarize.py
```

This portable standard-library script checks packaged data, recomputes paired statistics and writes the deterministic evidence index. It performs no build, browser, network or workload execution. `evidence-index.json` records compressed and uncompressed hashes. Historical binary paths in unchanged manifests require their original frozen artifacts for reruns; the publication folder is evidence, not executable packaging.

Reproduction uses the hash-bound `scripts/perf/compare-web-vm.mjs` (21 trials), `compare-first-use-web-vm.mjs` (15 trials), and `swift/compare.mjs` (15 pairs), with 1000 ms warmup. VM packages are `parse-architecture/er-borrowed-interner-first/package` and `parse-architecture/er-dense-array-first/package`; cases are Sequence small and ER small/medium/large, fixed/Canvas. Swift control is `er-borrowed-interner/candidate-swift`; candidate manifest is `er-dense-array/candidate-swift/swift-build-manifest.json`, cases `er/medium,sequence/small`, stages `parseMutable,lifetime,parse`, order offset 0. Any rerun requires a coordinated quiet window and fresh, separately named outputs.
