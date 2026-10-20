# Performance

Current matched web benchmarks run on an independent Linux VM. Actual Swift APIs
run locally on macOS because the VM cannot supply the macOS/AppKit SDK.

**The Rust/WASM API is currently slower than Pele JS on the measured workloads.**
No release, language choice or GPU availability implies otherwise. The target of
matching or beating Pele JS remains open. Measured results, raw samples, artifact
hashes, reproduction steps and unshipped compiler/schema experiments are in the
[current normalized-projection report](benchmarks/2026-10-20-normalized-projection/README.md).
The latest complete gate still fails all 126 measured conditions. Source,
parse and retained medians across the matrix are 2.46×, 11.67× and 4.11× JS.
Earlier direct Gantt construction improves scoped parse costs; current normalized
projection improves Swift Gantt parse. VM diagnostics remain mixed, with control
slowdowns disclosed. No 0.3 release.

| Large workload | Pele JS | Rust/WASM | WASM / JS |
| --- | ---: | ---: | ---: |
| Flowchart | 1.392 ms | 2.467 ms | 1.77× |
| Sequence | 0.513 ms | 1.020 ms | 1.99× |
| ER | 2.731 ms | 6.305 ms | 2.31× |
| Pie | 0.278 ms | 0.630 ms | 2.27× |
| Sankey | 0.853 ms | 1.455 ms | 1.71× |
| Gantt | 0.688 ms | 2.524 ms | 3.67× |
| Railroad | 0.787 ms | 1.910 ms | 2.43× |

The [earlier eager Swift checkpoint](benchmarks/2026-10-20-swift-eager/README.md)
compares frozen Rust-backed Swift executables on this Mac. Sequence-small parse
falls from 0.3084 to 0.0587 ms (5.25×); Gantt-medium falls from 5.3088 to
0.8451 ms (6.28×), each winning 15/15 matched pairs. Source and retained render
medians stay near flat. This includes the combined source changes since
`02ee639`, not isolated bridge causality, all-family timing or a JS comparison.
The 33-family Swift model corpus and 168 consumer API conditions pass; native UI
activation/focus assertions and current Intel validation remain open.

The [current direct normalization](benchmarks/2026-10-20-normalized-projection/README.md)
removes the first JSON buffer/reparse for captured builtin Gantt. Against the
preceding eager binary, Gantt-medium Swift parse falls from 0.8582 to 0.6292 ms
(26.69% paired, 15/15 faster). Negative-control medians remain near flat. Six
separate windows have an aggregate 48/42 first-side order imbalance. Current
Gantt corpus, eager checks, native contracts and three browser engines pass;
current VM diagnostics remain mixed and do not establish a web improvement.

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
