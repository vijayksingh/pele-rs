# Native Rust cost, without WASM or Swift

[Raw baseline](baseline.json) measures the current release Rust library directly
against pinned Pele JS in Node on the same Apple M1 Pro. There is no WASM, C,
Swift, JSON control pipe or result serialization inside these operation timers.
The native typed source/retained interfaces do not construct the JS public
ModelGraph. These are separate measurements, not browser timings minus an
estimated bridge cost. Do not subtract them from Linux VM figures.

Seven medium workloads, nine alternating trials, at least 500ms measured warmup,
fixed identical UTF-16 text metrics/UTC clock and explicit Node GC outside each
JS batch. Complete SVG hashes/bytes and exact dimension bits are checked before
and after work across source, typed retained and mutable publication interfaces.
The host is shared: initial load averages are 6.36/7.03/7.97 and external activity
is recorded. These results are not all-33 coverage or a universal Rust speed claim.

## Actual typed rendering

Ratio = native median / JS median; below 1 means less elapsed time.

| Medium workload | Source Rust / JS | Ratio | Typed retained Rust / JS | Ratio |
| --- | ---: | ---: | ---: | ---: |
| flowchart | 0.397 / 0.580 ms | 0.69× | 0.292 / 0.365 ms | 0.80× |
| sequence | 0.128 / 0.259 ms | 0.49× | 0.092 / 0.130 ms | 0.70× |
| er | 0.896 / 0.984 ms | 0.91× | 0.702 / 0.674 ms | 1.04× |
| pie | 0.142 / 0.317 ms | 0.45× | 0.126 / 0.250 ms | 0.50× |
| sankey | 0.171 / 0.437 ms | 0.39× | 0.146 / 0.356 ms | 0.41× |
| gantt | 0.297 / 0.289 ms | 1.03× | 0.240 / 0.209 ms | 1.15× |
| railroad | 0.245 / 0.266 ms | 0.92× | 0.222 / 0.161 ms | 1.38× |

Four source workloads clearly favor native in this sample; ER, Gantt and Railroad
source confidence intervals cross parity. Typed retained ER is 4% slower,
Gantt 15% slower and Railroad 38% slower. Native typed parsers also have real
exceptions: ER takes 1.67x and Gantt 1.41x the paired JS parse time. Parser APIs
return different model types, so these are actual interface costs, not isolated
algorithm instructions.

## Native compatibility work remains expensive

The public mutable representation has a different return contract from typed
parse or JS raw-model parse. It constructs an identity arena, graph descriptors
and native mutable state even without an external language crossing. Gantt's
public parse measures 0.360ms vs JS 0.043ms (8.38x). Its render plus native graph
publication measures 0.803ms vs JS raw-model render 0.201ms (4.00x). These are
additional native costs; attributing them to WASM would be incorrect. Publication
also returns graph/revision state that the JS render reference does not return.
Gantt retains scheduling/Date identities and reports before/after arena history
per fresh batch; this is not indefinitely steady retained-document latency.

Allocation counts come from a separately instrumented binary, never the timing
binary. One warmed typed retained render requests 4,221 allocations / 797,871
bytes for Flowchart and 9,677 / 1,824,194 for ER. These are requested allocations,
not RSS or comparisons with JS GC. Source SVG finalization still repeatedly
scans/copies output and render-local views clone model payloads. They are concrete
native improvement targets, separate from the compatibility transport layer.

## Provenance and gate

Runtime source is 31b4a04 (report HEAD f547415 plus the refreshed independent
consumer lock, with exact source and binary hashes in its build manifest).
The consumer lock now follows the existing 0.3 candidate path dependency and
removes the already-removed rustybuzz dependency closure; no production dependency
was changed. This does not publish or release 0.3.

Native results do not pass the browser gate: all 126 latest WASM comparisons
remain slower. The SVG-root rewrite and compact publication prototypes are not
included here. Swift is independently tested through the actual local adapter;
these native numbers do not measure Swift performance.

## Controlled native diagnostics

The [identical-byte three-way control](aa.json) compares two independent Rust
processes and pinned JS with matched Rust batch counts, fifteen balanced trials
and full output checks. Flowchart source/retained varied +2.2%/-1.2%; ER varied
+0.1%/-0.6%. These observations bound this diagnostic, not future shared-host
variance. [Comparison method](../../../scripts/perf/COMPARISONS.md) records
compiler/test guards and binary/source fingerprint requirements.
