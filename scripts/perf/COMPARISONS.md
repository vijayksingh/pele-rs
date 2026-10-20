# Controlled VM diagnostics

`web-vm.mjs` and `check-web-parity.mjs` remain the complete JS release gate.
The source, fresh-parse and retained render paths now each compare complete
SVG and exact dimensions before and after timing. The checks remain outside
timed loops. Historical reports retain their original harness hashes.

`compare-web-vm.mjs` loads independent control/candidate packages plus pinned JS
in one browser/page. Every package file is hashed before/after. Fifteen trials
rotate and reverse runtime order, balancing each position. Both WASM packages
use the smaller calibrated batch count, with median-of-three calibration;
this prevents a transient calibration sample from assigning 128 operations to
one WASM package and 16 to the other. Warmup is at least 500ms per runtime.
Source, fresh parse and retained output are independently checked against JS.

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/compare-web-vm.mjs \
  --control old/package --candidate new/package --trials 15 \
  --cases flowchart/small,sequence/small --stages parse,render-model \
  --output controlled.json
```

This is diagnostic, not release acceptance. Two WASM heaps and shared JS/GC
pressure differ from the single-package release harness. Identical-byte A/A
packages still showed roughly 4–6% parse variation. Preserve A/A and reversed
package controls; do not declare every small row difference a causal win.
The earlier version used independent one-sample calibration; its exact source
is retained with the experiment evidence under docs/benchmarks.

`disposed-storage-web.mjs` performs a retained-reference census, without reading
an operation timer. In one synchronous job, it wraps ModelBridge publication
to observe weak bridge references, then calls public parse/dispose 200 times.
WeakRef construction already keeps bridge targets alive until the job ends.
Immediately afterward it counts transport graphs, records, UTF-16 units and
host-object slots still attached to those disposed bridges. It also counts
unique invoke functions, revealing per-model closure retention. Source output
is checked against pinned JS; artifacts are hashed before/after.

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/disposed-storage-web.mjs \
  --control old/package --candidate new/package --output retained.json
```

These counts are references/graph content, not allocated bytes, heap size, RSS,
or a claim of a leak across event-loop jobs. Caller-owned model data and aliases
remain independently live when the caller retains them. Functional browser
coverage exercises disposal, borrowed helpers, descriptors, busy callbacks and
recovery against both old and candidate packages.

`compare-native.mjs` compares two frozen native Rust consumer binaries and pinned
Pele JS on the same host. Native timing happens inside the Rust consumer, with
NDJSON control and output verification outside the timer. It validates both
binaries against their build manifests, requires the candidate source fingerprint
to match the checkout before/after, and checks complete outputs before/after.
The control's historical source fingerprint remains explicitly recorded.

```sh
node --expose-gc scripts/perf/compare-native.mjs --quiet-run \
  --control frozen/control/pele-perf \
  --candidate scripts/perf/native/target/release/pele-perf \
  --output native-controlled.json
```

Fifteen trials balance cyclic/reverse runtime order; three calibrations choose
matched control/candidate counts. At least 500ms measured warmup precedes timing.
Seven medium workloads cover source and typed retained rendering; `--cases`
and `--stages` can bound an identical-byte A/A check. Node GC runs outside each
JS batch. Compiler/Pele-test guards and shared-host load records remain active.
This diagnostic neither measures Swift/WASM nor subtracts local native time from
Linux browser measurements. It is not the browser release gate.

`--stages parse,parse-public` additionally compares typed/native public parsing
against pinned JS parse. Native public parsing constructs an additional owned
identity graph; neither lane times its NDJSON pipe or graph serialization.
Use typed parse as a negative control for graph-construction changes. All
rendering output checks still run before/after a parsing comparison.

`--stages render-mutable-publish --batch-count 16` fixes every runtime to the
same bounded count (1–128). Native resets and warms before each batch; recorded
before/after node counts expose retained-history growth. JS still does not
return the extra native graph/revision. Compare native control/candidate at the
same history, never label this indefinitely steady-state latency. Allocation-only
runs honor the same fixed count and record it explicitly.

Native typed documents remain retained across batches; only native public state
is reset. JS retained models are reparsed and warmed for each batch. Historical
reports' `freshRetainedStatePerBatch` field was too broad; their implementation
already had this distinction. Current reports name each setup explicitly and
verify complete JS retained output before timing and after the measured count.

`--allocations` uses separate instrumented binaries whose hashes are checked
against each manifest; those binaries refuse timing commands. `--allocation-only`
collects the same counters and public output checks without reading any operation
timer. It requires no quiet window and emits no elapsed-time rows. Requested
allocation traffic and peak logical requested bytes are not RSS or latency.
