# Numeric live-read retained profile

Candidate7417fed53e34f482f726dd07a12fec4dd32b9425/WASM96f4e39 versus original e148. Same exact7aae1db1 profiling harness: medium/large fixed, four serial5-second CPU windows,250µs sampling,500ms warmup, parse once outside windows. Oracle SVG/dimensions/ordered callback traces exact before/after; width callbacks48/200 for all lanes. No prototype interception, no extra counting run. All117/113 package, oracle, harness, case hashes unchanged; browser and models closed.

| Candidate case | Prepare inclusive | Snapshot inclusive | Sparse encode inclusive | Apply inclusive | Reader union inclusive | Live-phase text decode leaf |
|---|---:|---:|---:|---:|---:|---:|
| sankey-medium-fixed-render-model-candidate | 39.41% | 17.58% | 7.87% | 12.82% | 12.83% | 0.48% |
| sankey-large-fixed-render-model-candidate | 39.05% | 18.80% | 7.13% | 12.33% | 12.98% | 0.21% |

The repeated operation/key-string decode target is essentially gone. Total text-decode leaf5.07%medium/4.53%large now mostly result-factory2.37/2.28 and resource-width1.70/1.89. Remaining live-phase decode0.48/0.21; do not turn share changes across runs into an isolated latency claim.

The largest coherent remaining boundary is accepted-input preparation. `bindings/web/full.js:214` clears confirmedGraph at detached completion. `prepareMutableInput` at221 then snapshots/encodes/applies again, even on this unchanged repeated-render workload. Snapshot inclusive17.58/18.80, encodeSourceInput7.87/7.13, and native apply12.82/12.33 all sit within prepare39.41/39.05. Native commit_admitted alone7.03/6.89 is inside apply. They are not independent ceilings. An exact accepted-baseline receipt may avoid redundant full capture/apply only after proving no revision/publication/failed-observation discrepancy; callbacks and factories can mutate live objects, so observation itself cannot be skipped by a readonly label.

Explicit Reader call/key/units ancestry union12.83/12.98 remains separate from preparation. Its call5.96/6.19,key4.87/4.59,units2.01/2.20 includes nested work. Live HostPhase44.16/46.46 also contains geometry/SVG, not just interop. charCodeAt self0.12/0.11 and direct JS reader switch0.14/0.01 are small. These data favor the coarse preparation boundary ahead of another string-transfer microchange.

Leaf role categories in attribution.json partition all weighted samples. Ancestor categories deduplicate per sample but overlap; do not add them. Native symbols may be inlined. The inherited summary native-parser regex falsely includes IndexedSankey generic type names in render symbols; ignore it—no parse occurs in the windows. Exact raw4 profiles remain authoritative. No allocation count, speedup ceiling or broad parity claim.

Execution: one remote run, exit0; no preflight/runtime failure. One local analyzer attempt raced an incomplete scp and failed JSON parsing; after the same fetch completed, offline analysis passed. No rerun of profiling. VM idle afterward; no first-use.
