# Performance

Current matched web benchmarks run on an independent Linux VM. Actual Swift APIs
run locally on macOS because the VM cannot supply the macOS/AppKit SDK.

**The Rust/WASM API is currently slower than Pele JS on the measured workloads.**
No release, language choice or GPU availability implies otherwise. The target of
matching or beating Pele JS remains open. Measured results, raw samples, artifact
hashes, reproduction steps and unshipped compiler/schema experiments are in the
[current direct-parse report](benchmarks/2026-10-20-direct-parse/README.md).
The latest complete gate still fails all 126 measured conditions. Source,
parse and retained medians across the matrix are 2.39×, 11.39× and 4.02× JS.
The matched Gantt comparison reports a scoped parse improvement, with first-use
and control regressions disclosed. No 0.3 release.

| Large workload | Pele JS | Rust/WASM | WASM / JS |
| --- | ---: | ---: | ---: |
| Flowchart | 1.327 ms | 2.398 ms | 1.81× |
| Sequence | 0.518 ms | 1.040 ms | 2.01× |
| ER | 2.818 ms | 6.420 ms | 2.28× |
| Pie | 0.276 ms | 0.628 ms | 2.27× |
| Sankey | 0.863 ms | 1.431 ms | 1.66× |
| Gantt | 0.689 ms | 2.563 ms | 3.72× |
| Railroad | 0.769 ms | 1.847 ms | 2.40× |

The [current Swift checkpoint](benchmarks/2026-10-20-swift-eager/README.md)
compares frozen Rust-backed Swift executables on this Mac. Sequence-small parse
falls from 0.3084 to 0.0587 ms (5.25×); Gantt-medium falls from 5.3088 to
0.8451 ms (6.28×), each winning 15/15 matched pairs. Source and retained render
medians stay near flat. This includes the combined source changes since
`02ee639`, not isolated bridge causality, all-family timing or a JS comparison.
The 33-family Swift model corpus and 168 consumer API conditions pass; native UI
activation/focus assertions and current Intel validation remain open.

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
