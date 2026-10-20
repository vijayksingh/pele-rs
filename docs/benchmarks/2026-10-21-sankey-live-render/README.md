# Live render architecture: rejected performance result

Source `8175006345f3b66a6c9fd942814ed0ba5ff3014d`; default WASM `d5a8cbdccd9641e63acefb2d5ed66d3baa48c3d9cb70e5dc1917ca7c17f772b4`. Corrected [callback semantics](../../compatibility/sankey-resource-callbacks/implementation/README.md) pass Node122, browser105 across three engines (including366 callback comparisons), native57, Swift16, C3 and API168. **Performance is rejected; 0.3 remains unreleased.**

## Independent VM result

Same vm-agent host, pinned Pele1.6.0, original e148 Rust control, fixed/canvas measurers, Sankey8/48/200 plus medium Flowchart, three stages,21 balanced trials and1000ms warmup:24 diagnostic rows. Full package maps117/113, exact timed harness1429f773…, casesb16a2813…, and36 untimed activation/continuity/ingress checks pass. All raw/adverse rows are retained.

| Sankey stage | Paired change versus e148 | Current / Pele JS |
|---|---:|---:|
| Parse | −4.29% to −40.21% |5.84–14.03× |
| Retained model render | +193.86% to +223.45% |5.63–9.11× |

All six retained conditions have zero lower candidate trials out of21. Small fixed parse has a −4.29% paired change but a +0.63% ratio of lane medians; neither measure is hidden. Flowchart controls regress1.36–1.45% in parse and1.16–4.53% in retained render. The [complete table](RESULTS.md) includes absolute times, source rendering and both profiles. These are within-run paired comparisons, not isolated gains compounded from older runs; e148 lacks the corrected mutating-callback behavior.

First-use was not rerun for this candidate: the warm retained regression already rejects it. The next step is a bounded profile of the corrected retained path, then a coarser host boundary that preserves read ordering. No performance improvement is inferred from Rust or from removing reconciliation.

Raw warm report SHA256: `43c0bffeaf2bab3d30f0eec8c05df3f726d1570ca565e7a4b3120525b590b506`.

[Artifact manifest](artifact-manifest.json) records original/stored hashes. Gzip files decompress to exact original bytes. Executed harnesses/probes and package/source provenance are included; no failing/adverse rows were dropped. This diagnostic run is not the complete126-condition release gate.

## Retained-path profile

Four bounded CDP profiles use medium/large fixed retained models, candidate/control,5 seconds per lane at250µs sampling. Parsing is outside the window. Full SVG, dimension bits and resource traces match Pele before and after; package bytes remain unchanged. These are diagnostic profiles, not timing-gate samples.

Decode/decodeText leaf samples specifically under live Reader::call or its inlined phase caller account for33.61% of medium and35.05% of large candidate samples. Rust recreates operation, property-key, method and context JavaScript strings on every read, so generated glue repeatedly UTF8-decodes command metadata. Result-factory and resource-width decoding are separate; per-unit charCodeAt self samples are only0.06/0.11%. prepareMutableInput inclusive samples are24.44/23.97%; inclusive regions overlap and must not be added to leaf shares.

The next interface change replaces string command/context operands with numeric values and retains property-key handles for the job. It must preserve property-read order, original errors and callback boundaries. No speedup is claimed before another matched run. Full profiles, executed program and attribution are preserved under profile/.

The profiler [attribution notes](profile/profiler-RESULTS.md) retain the exact decoder ancestry calculation. The automatic native-parser name category can match IndexedSankey generic type names during rendering and must not be interpreted as parser work in a retained-only window.
