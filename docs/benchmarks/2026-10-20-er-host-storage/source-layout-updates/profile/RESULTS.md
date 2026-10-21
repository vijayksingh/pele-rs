# Frozen 9d CPU diagnostic

One authorized execution; nine medium/fixed windows, 2 seconds each, 250µs sampling, 500ms warmup. Default public API; no experimental selector. Control e148 (`ec9c0d09…`), candidate9d (`032bfbec…`), pinned PeleJS. Sampled work is byte-identical to the previously executed d35 profiler. Only candidate pins and the untimed PLCSRC02/PLCSRU01 same-layout identity probe changed. Models disposed, browser closed, all output checks passed, and all package/oracle/harness/case hashes matched before/after. SSH execution2887 exited0; transfer11011 and fetch18311 exited0. Full raw profiles and original summary remain in runtime/cpu-profiles.

`analyze.py` verifies the raw hashes and regenerates `attribution.json`. That analysis corrects the inherited broad regex that classified renderer generic `IndexedSankey` type arguments as parser work. It also separates actual JSON counting from wrappers that include the whole renderer. The original summary is retained unchanged. No allocation or invocation counts are inferred.

## Current sampled boundaries

Percentages are weighted sample time, not benchmark deltas. Ancestors overlap; the named unions count each sample once.

| Boundary |9d parse|9d retained|e148 parse|e148 retained|
|---|---:|---:|---:|---:|
|JS publication inclusive|35.36%|22.44%|25.54%|14.12%|
|JS matching inclusive|0%|21.46%|0%|10.33%|
|JS publish/match/observe union|35.36%|43.89%|25.54%|24.45%|
|JS graph boundary union, including decode/access/binary|41.76%|45.23%|34.19%|29.58%|
|9d structural/schema/count stage union|13.31%|0.10%|not comparable by these logical-stage names|not comparable|
|GC self|11.18%|0.56%|15.96%|0.51%|

9d parse admission splits into structural4.20%, schema-ID3.54%, canonical count5.58% inclusive (union13.31%). Native layout encoding4.96% and source catalog interning5.16% are separate observed phases. Native parser4.07% inclusive. JS initial recipe decode6.08% inclusive and logical scalar access8.65% self. `reconcileGraph` alone26.98% self.

Retained operations no longer repeatedly encode/decode the immutable recipe: native layout packet0.151% inclusive, JS decode0.96%. Native admission union0.096%. Remaining JS scalar access14.27% inclusive, publication22.44%, matching21.46%. Property accessor5.39%, item3.35%, expression evaluation1.80%, property-slot iteration1.60% are disjoint leaf samples. Actual native renderer35.99% inclusive; bounded JSON count4.71% inclusive remains. Untagged engine `decode`7.02% is retained as engine work, not mislabeled Rust/JS. GC0.56% does not support an allocation-churn claim for this retained window.

Schema serialization/registration is not the primary current parse boundary: schema-definition validation0.81% inclusive, schema/cache-key/metadata named boundary union0.382%, native JSON0.586%. `snapshot_source_layout`17.18% inclusive includes admission/count/encoding and must not be labeled schema work. Inlined registration work can remain inside larger frames; absent samples do not prove zero cost.

Direct source controls remain separate: native renderer65.30% candidate/65.13% control inclusive; bounded JSON count8.20%/approximately8% and engine decode10.41% candidate. Source creates no public graph. This diagnostic does not convert those proportions into latency claims.

## Concrete next boundary

Current `LayoutStorage.readProperty` already evaluates a full property once during the fill/match phase; do not claim five full evaluations per property there. But `LayoutGraph.property`, count, kind, aux, item and property-key calls each independently resolve the node override/group. `propertySlots` repeats propertyCount for every yielded field. Retained deletion-key collection calls the full property evaluator, including an unused value. Map/Set atomTag and atomPayload resolve the same item separately, with a third numeric-bit lookup during matching. Snapshot comparisons likewise repeatedly resolve the same immutable baseline record.

The bounded correction is an operation-local node cursor: select the override/group once per current node, reuse its scalar count/template references, evaluate key-only reads without the value, and hold one copied atom row for repeated tag/payload/bits access. No eager model expansion, no graph-level string-index reuse, no skipped observation/restoration, no change to fresh assignment policy. Reuse existing optional traversal read-session hooks; keep generic publication code and default NestedStorage unchanged. Snapshot capture owns a separate cursor so reentrant callbacks cannot corrupt an outer traversal. This is a candidate architecture correction, not a measured gain.

Prior ER literal publication was rejected; this proposal changes no allocation/literal factory or setter behavior. Dense-array wire construction is unrelated to this logical read boundary. Native fused initial admission remains a separate possible parse-only layer; it cannot explain away the much larger JS retained boundary.
