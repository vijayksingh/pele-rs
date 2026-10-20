# Independent evidence audit

The frozen comparison passed arithmetic and identity checks for all126 conditions,9 samples per runtime/row,500 ms minimum warmup and matched control/candidate batch counts. Reported standalone medians, sample p95 of batch means and all three runtime ratios recompute from raw arrays. The summary additionally calculates paired median changes and wins. No samples are dropped or pooled.

Both actual local frozen packages and both physical VM directories were rehashed independently after timing. Each has106 regular files; all maps match the benchmark's before/after identity and the build freeze. The source freeze has657 entries and matches the seven-file candidate manifest, including the public feature test. The source-before file hash and exact build-log hash match package-freeze. This is source/build provenance plus physical package identity, not a reproducible second build.

The 351 native inputs and existing SDK/consumer hashes match the source freeze and the new before/after Swift checks. Fifteen selected integrations and168 API conditions passed. PriorC3 is explicitly reused from the identical dense native layer, not a new run.

Focused33 browser checks passed across all three engines. The bundled24 explicit reports preserve full SVG/results, resource traces, raw numeric bits, fallback counts and error/budget evidence; the data-only summarizer rechecks their reported equalities. The timing report itself retains successful pre/post verification byte counts and checksums, not full SVG bodies. Its full-output assertions are in the frozen executed harness.

All42 source paired medians are lower; parse and retained controls contain adverse rows. These are observed diagnostic results. Nine odd trials imply five control-before-candidate and four candidate-before-control orders per row even though each runtime's three-way position is balanced. No regression-free claim, control adjustment, memory reduction or new full release-gate result is inferred.

The full405 browser run passed in13.1 minutes. All657 pre-build source-map entries independently match immutable Git blobs at640deee; no future working-tree source was used. status.json binds that completed evidence. Eighteen subsequent profiles use the same106-file candidate package; raw compressed profiles and their original self-time summary are retained. No additional latency conclusion is inferred from profile sampling.
