# Performance

Current matched web benchmarks run on an independent Linux VM. Actual Swift APIs
run locally on macOS because the VM cannot supply the macOS/AppKit SDK.

**The Rust/WASM API is currently slower than Pele JS on the measured workloads.**
No release, language choice or GPU availability implies otherwise. The target of
matching or beating Pele JS remains open. Measured results, raw samples, artifact
hashes, reproduction steps and unshipped compiler/schema experiments are in the
[current direct ER report](benchmarks/2026-10-20-direct-er/README.md).
The latest complete gate still fails all 126 measured conditions. Source,
parse and retained medians across the matrix are 2.40×, 10.94× and 4.03× JS.
Earlier direct Gantt construction improves scoped parse costs; current normalized
projection improves Swift Gantt parse. VM diagnostics remain mixed, with control
slowdowns disclosed. No 0.3 release.

| Large workload | Pele JS | Rust/WASM | WASM / JS |
| --- | ---: | ---: | ---: |
| Flowchart | 1.358 ms | 2.404 ms | 1.77× |
| Sequence | 0.511 ms | 1.010 ms | 1.98× |
| Er | 2.811 ms | 6.272 ms | 2.23× |
| Pie | 0.277 ms | 0.628 ms | 2.27× |
| Sankey | 0.858 ms | 1.510 ms | 1.76× |
| Gantt | 0.689 ms | 2.506 ms | 3.64× |
| Railroad | 0.771 ms | 1.845 ms | 2.39× |

The [earlier eager Swift checkpoint](benchmarks/2026-10-20-swift-eager/README.md)
compares frozen Rust-backed Swift executables on this Mac. Sequence-small parse
falls from 0.3084 to 0.0587 ms (5.25×); Gantt-medium falls from 5.3088 to
0.8451 ms (6.28×), each winning 15/15 matched pairs. Source and retained render
medians stay near flat. This includes the combined source changes since
`02ee639`, not isolated bridge causality, all-family timing or a JS comparison.
The 33-family Swift model corpus and 168 consumer API conditions pass; native UI
activation/focus assertions and current Intel validation remain open.

The [direct normalization checkpoint](benchmarks/2026-10-20-normalized-projection/README.md)
removes the first JSON buffer/reparse for captured builtin Gantt. Against the
preceding eager binary, Gantt-medium Swift parse falls from 0.8582 to 0.6292 ms
(26.69% paired, 15/15 faster). Negative-control medians remain near flat. Six
separate windows have an aggregate 48/42 first-side order imbalance. Current
Gantt corpus, eager checks, native contracts and three browser engines pass;
current VM diagnostics remain mixed and do not establish a web improvement.

The [current static-policy correction](benchmarks/2026-10-20-projection-policy/README.md)
removes 33 handler-box allocation requests per registry construction (69 to 36).
Gantt/Sequence Swift parse paired medians fall 0.86%/2.16% against the preceding
normalized binary, each 13/15 faster; all six windows balance first-side order
45/45. Outliers remain disclosed. Full current 33-family Swift corpus passes,
including 167,473 mutable drawings and 270,358 error cases with five existing
explicit budget differences. VM source medians return near the earlier control;
parse remains mixed and the complete gate still fails all 126 conditions.

The [ER immutable normalization](benchmarks/2026-10-20-normalized-er/README.md)
reduces matched ER-medium Swift parse by 25.67% paired (0.8484 to 0.6219 ms),
faster in all 15 pairs. Controls are 0.2–2.1% slower by paired medians. Native,
ER Swift corpus, 168 API conditions and nine browser corpus checks pass.
Independent VM results stay mixed; mutable web parse does not use this path.
No new complete gate or release acceptance follows.

The [direct ER construction](benchmarks/2026-10-20-direct-er/README.md) avoids
its complete temporary typed model during eligible mutable parsing. ER-medium
Swift mutable parse improves 17.912% paired, lifecycle 3.228%, and native public
parse 18.503% (15/15 faster), repeated at 18.343% in a separate window. Native
public parse remains about 1.19 times JS in the original window. Some controls
regress; the separate identical-binary A/A window does not erase them. VM ER
parse paired improvements range 0.856–7.354%; first-use results remain mixed,
including +41.167% ER-medium Canvas first-invoke. All 126 complete gate conditions
still fail. The resource-callback model mutation mismatch is separately
reproduced and remains a correctness/release blocker.

[Rejected parse candidates](benchmarks/2026-10-20-parse-candidates/README.md)
retain raw first-use and live-heap evidence. Lower map count and alternate string
storage did not establish a broad improvement and remain outside production.

Historical checkpoints remain separately scoped:

- [Original VM and Swift baseline](benchmarks/2026-10-19-vm/README.md)
- [Native core attribution](benchmarks/2026-10-20-native-core/README.md)
- [Mutable copying](benchmarks/2026-10-20-mutable/README.md)
- [Copying candidates](benchmarks/2026-10-20-copying-candidates/README.md)
- [Direct construction comparison](benchmarks/2026-10-20-direct-comparison/README.md)
- [Initial publication](benchmarks/2026-10-20-initial-publication/README.md)
- [Registration schema](benchmarks/2026-10-20-registration-schema/README.md)
- [Schema cache](benchmarks/2026-10-20-schema-cache/README.md)
- [Gantt builder](benchmarks/2026-10-20-gantt-builder/README.md)
- [Gantt lookup](benchmarks/2026-10-20-gantt-lookup/README.md)
- [Gantt fixed fields](benchmarks/2026-10-20-gantt-fields/README.md)

These are warm public-API medians, excluding startup, downloads and SVG painting.
Read each report's exact workloads, stages, controls and limitations before
comparing checkpoints. No 0.3 release is authorized by the current results.
