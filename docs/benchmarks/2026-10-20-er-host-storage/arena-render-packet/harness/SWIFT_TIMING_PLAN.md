# One bounded Swift timing attempt

Prepared only. Do not run until parent explicitly says go and the existing full-browser log has terminal `417 passed`. Log completion alone is not authorization.

```sh
node dist/perf-patches/parse-architecture/arena-render-packet/bounded-swift-timing.mjs --parent-go
```

Run from repository root. The script checks the log again at entry, verifies the current candidate source/SDK and both binaries against frozen manifests, and invokes the unchanged public comparator once for each Sankey/medium stage: `render`, `lifetime`, `parseMutable`, `parse`. Each condition has complete15 pairs and1000ms matched warmup per side; comparator constants determine these settings. Order offsets alternate0,1,0,1, so all four completed conditions yield30 control-first and30 candidate-first pairs.

The overall budget is180 seconds from scheduler start, including setup. No retries and no waiting for global compiler activity: any guard failure immediately ends the attempt. A budget timeout terminates only the newly created comparator process group (including its consumer children) and rejects that entire condition. Complete earlier conditions remain separate; partial results never enter accepted conditions. Existing attempt directories cause refusal instead of overwrite.

Output: `runtime/swift-bounded-attempt1/` contains the schedule, each exact command/log, raw comparator report when created, accepted/unmeasured condition lists, input/manifest/binary/harness hashes, host process snapshots and failures. An initial guard rejection is a recorded blocked attempt with all four conditions unmeasured. Shared guard rejects global compiler processes even at zero CPU and Pele tests. Other host load is recorded and is not described as idle.

Control: owned construction `b19f38d`, binary `368ab53b...`, original frozen manifest. Candidate: arena `9ae8142`, binary `2e1491d5...`, source/SDK aggregate `e632dc00...`; preserved frozen binary and actual manifest binary must match. No build, source edit, guard override or browser execution occurs.
