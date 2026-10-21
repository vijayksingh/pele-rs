# Current ef9 first-edited CPU profile

Exactly one authorized session, three windows: e148 control, ef9 candidate, pinned PeleJS; Sankey medium/fixed; 2 seconds, 250 µs sampling, 500 ms warmup. The sampled operation is the byte-identical scenario/work closure from first-use harness5469078: parse, cyclic extension creation, render, disposal. Before/after verification compares whole SVG/dimensions and extension identity/cycle. All8+16+12 separate activation checks passed. Candidate116/control113 package files, oracle and script hashes unchanged; renderers disposed and browser closed. No latency rerun or source edits.

Frozen profiler SHA e704ec2e47dbb53b3fa27d4e09e3af69efec04d0fb6b6631862618431502264d. Raw summary SHA46cac7fc8ebf49aa8f123f80ccecebe2c6bf726efd00e0cb1fa10f50851800c4. `index.json` binds raw profiles, fetched executed scripts and corrected offline analyzer.

## Current candidate attribution

Percentages are weighted CPU samples of the complete first-edited operation, not elapsed-latency ratios. Ancestor rows overlap; do not sum them indiscriminately.

| Boundary | Self | Inclusive |
|---|---:|---:|
| JS prepareMutableInput |0.11%|42.95%|
| Native apply_source_input entry |0.05%|19.83%|
| JS snapshotGraph |6.60%|11.85%|
| JS encodeSourceInput |0.43%|5.60%|
| JS matchesGraph |2.35%|4.60%|
| Native with_candidate validation callback |0.02%|8.84%|
| Native commit_admitted, including eager projection |0.52%|9.11%|
| CapturedLogical::own_fields |1.01%|5.35%|
| SourceCandidate::string_chunks |2.19%|3.77%|
| SourceCandidate::properties |0.70%|10.37%|
| JS complete fresh parse |0.13%|19.35%|
| JS fresh initializer |5.16%|5.16%|
| Native actual parser |0.63%|1.37%|
| Native renderer/layout/SVG union |6.72%|15.81%|

SourceCandidate properties overlap validation and projection. Captured field/string ancestor union is6.08%; do not add that to commit. Native validation/count within apply_source_input is8.62% of the whole operation. Visible ingress admission+commit union is18.98%. Whole JS graph-boundary union is33.45% (fresh initializer, observation/matching/publication, layout decoding/access, binary); observation/matching/publication alone22.30%.

Disjoint leaf language partition: candidate WASM52.30%, JS41.81%, GC1.36%, engine/profiler4.53%. Control WASM41.00%, JS44.80%, GC8.51%, engine/profiler5.70%. JS oracle JS92.30%, GC4.06%, engine/profiler3.64%. Cross-profile shares have different denominators; they do not establish speed ratios or allocation counts.

## Actionable source chain

Current `PublicDocument::apply_source_input` does:

1. Borrowed physical scan and `SourceCanonical::prepare_overlay`: exact virtual64MiB frame count; then complete DecoderBudget tables/units/work; only then materialization.
2. `admit_observed`: repeats exact frame count and DecoderBudget, then structural validation.
3. After stale/revision checks and schema compilation, `graph_read::validate_schema`: structural validation again, then schema IDs.
4. Explicit module structural validation again.
5. `commit_admitted`: eager importer through generic CapturedLogical, then atomic adoption.

The source confirms duplicate traversals; CPU sampling does not count invocations. No callback or state mutation occurs between candidate validation stages. A future single immutable-candidate admission transaction can reuse completed physical/structural evidence at later original stages, while retaining original pre-allocation gates, deferred schema errors, current transport budgets and generic provider behavior. Its measured target is the8.62% validation/count portion inside native apply, not the whole42.95% preparation boundary.

Projection is a separate similarly sized cost (9.11% inclusive). CapturedLogical::own_fields synthesizes virtual Property batches and compares field names through string_chunks. Unchanged source catalog strings are encoded UTF8 to a1024-unit UTF16 scratch, including tiny field-name comparisons; importer then decodes label content. The6.08% field/string union supports inspecting a direct source-aware CapturedRead adapter that shares the existing importer and handles accepted overrides exactly. This would remove redundant conversions, not skip eager projection. No implementation authorized or performed here.

## Classifier limitations

Frozen runtime regex omitted angle brackets in `Counter>::`/`DecoderBudget>::` and some owner symbols. `analyze.py` corrects them from actual raw names without changing sampled code. Counter visible self/inclusive1.67%; DecoderBudget visible0.14%. Direct prepare_overlay/admit_observed/string_owned frames are absent; closure fragments identify at least1.03% prepare_overlay ancestry. Absence is inlining/coverage, not zero work.

A native symbol named `hash_one::<PrototypeId>` accounts for4.56% self, but raw caller stacks place it under SourceCandidate header/properties/string_chunks, including integer-key override lookups. Likely merged/shared hash code: this is **not** proof of4.56% prototype-schema validation. Analyzer labels it shared-hash-symbol and preserves complete stacks. HTTP WASM URLs remain WASM even when generic trampoline names mention control during candidate execution. Native parser classification is anchored to actual parser modules, not generic IndexedSankey/SankeyModel type arguments.

No cache, validation relaxation, pipeline change or new performance claim follows automatically from these samples.
