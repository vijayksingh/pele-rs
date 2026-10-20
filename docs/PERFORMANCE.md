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

The following Swift table is the
[original baseline](benchmarks/2026-10-19-vm/README.md), on separate hardware;
it is not a cross-machine ratio or timing of the newer eager projection:

| Medium workload | Source render | Retained render | Mutable render |
| --- | ---: | ---: | ---: |
| Flowchart | 0.597 ms | 0.493 ms | 0.621 ms |
| Sequence | 0.275 ms | 0.235 ms | 0.290 ms |
| ER | 1.624 ms | 1.411 ms | 1.580 ms |
| Pie | 0.306 ms | 0.290 ms | 0.309 ms |
| Sankey | 0.399 ms | 0.372 ms | 0.420 ms |
| Gantt | 0.450 ms | 0.383 ms | 1.130 ms |
| Railroad | 0.469 ms | 0.442 ms | 0.501 ms |

These are warm public-API medians, excluding startup, downloads and SVG painting.
Seven families at three sizes are measured. Mutable operations include graph
exchange/publication costs. See the report for all profiles/stages and boundaries.
