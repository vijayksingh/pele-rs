# Shared browser graph traversal

`bindings/web/graph-storage.js` owns the host-value observer, publisher and
comparison walks. `graph.js` keeps registries, document ownership, the existing
nested graph representation and public bridge lifecycle. The default bridge
imports traversal closures bound once to `NestedStorage`; it does not select a
backend on each operation. Alternate storage binds its scalar accessors through
`createGraphTraversal` and reuses the same three walks.

The observer preserves historical IDs, LIFO discovery, descriptor and prototype
checks, UTF-16 strings, exact floating-point words and late scalar reads. A failed
observation can extend identity/string history but cannot commit its graph. The
publisher allocates missing shells before writing references, restores every
record after a callback, preserves descriptors and only registers native-created
shells as owned. The root is published last. An unchanged acknowledgment cannot
replace callback restoration.

The scalar interface is an internal storage seam, not a new public graph format.
It does not move diagram parsing, methods or rendering out of Rust. A flat store
and complete captured document lifecycle still need independent admission and
end-to-end integration before selection; this refactor selects neither.

## Verification

The exact packed candidate passed all 417 browser checks across Chromium, Firefox
and WebKit, plus 54 focused checks. Control and candidate used identical WASM
`3f6c40133130905f170ba582b3b9ef0c8718e795f2305e402200d5877563fc5b`,
built from `365ec6d`. Later native admission/core changes were not in that WASM.

The independent VM comparison measured 42 rows with 21 paired trials and 1,000 ms
warmup. Median paired changes were parse -0.19%, retained render +0.22% and
source render +0.14% (conventional medians across row medians). A separate 40-row first-use run used 15 paired trials and
1,000 ms warmup; every row was within 5% of its control. These descriptive results
do not establish speedup or performance neutrality. The earlier per-operation
backend abstraction was rejected after a +9.16% retained-render row; its evidence
remains published separately. The revised implementation does not satisfy the
Pele JavaScript release gate.
