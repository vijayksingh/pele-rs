# Flowchart, Swimlane and XYChart owned callbacks

Rust core/web runtime `d729e7c`; corrected native bridge/SDK `ce282ce`.
Flowchart and Swimlane share graph construction, style resolution and compound
geometry. Each responsive redraw captures fresh graph values; existing style
arrays stay live at node measurement. XYChart retains the pinned axis/series
read phases. Both immutable and mutable paths use the same Rust geometry.

- **Native:** 26 distinct integration features pass, including existing family
  SVG/model/error corpora, 192 new callback oracle rows and expanded class/default
  edge style limits with write retention and repair.
- **Swift:** a freshly rebuilt ARM64 SDK passes all 36 selected features in one
  run, including all 192 new oracle rows and all-33-entry/99-case regression.
- **C:** strict C15/C17/C18 consumers compile and execute. C15 additionally renders
  Flowchart and Swimlane through actual Array capture, with snapshot access and
  nested rendering during width callbacks.
- **Web:** 30 distinct checks pass across Chromium, Firefox and WebKit. The three
  families contribute 576 exact same-engine candidate/oracle/fixture comparisons
  (192 per engine). Expanded style limits, recovery, all-33-entry and source
  initializer regressions pass.
- **Identity:** native404 inputs, SDK13 files, web480 inputs and package118 files
  remain unchanged within each final build/check. WASM SHA256:
  `f9ea26dbee539cb3c0246558543abc325bea794ef76ea11c3cc1ca3cf99b0136`.

Failures are retained. The first browser run passed24 and failed six checks
before scenarios because generated tests accidentally changed two oracle hash
literals while changing row counts. The corrected six checks pass a focused
rerun. Initial Swift checks also caught a real C15 gap: iterable capture accepted
Maps but not Arrays. The corrected bridge captures bounded ordinary sparse
Arrays, retaining shared string identities; the SDK was rebuilt before the final
36-pass Swift run. No failure is counted as passing.

Fresh edit/throw audit matches **29/33** families. Agentflow, Gantt, Sequence and
Class remain mismatched. This is one observer per family, not full API parity.
Flowchart/Swimlane foreign-helper overrides, inherited reflection, exotic
iterables and other previously documented dynamic/UTF16 cases remain separate
work. No performance run or release was performed.

The archive retains 56 lossless records, including original test sources,
failures, source/SDK/package maps, repaired checks and paired browser reports.
Run `python3 docs/parity/2026-10-21-owned-flow-families/verify.py --git` to verify
archive integrity, counts, comparisons and source bindings.
