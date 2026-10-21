# Pie, Info and Gantt functional checkpoint

The frozen native SDK and web package at `847b4bc` add owned Pie/Info rendering
and Gantt reference-warning callbacks. Rendering releases the document before
calling resources and reads mutable fields at the upstream phases. Agentflow's
raw default diagnostics also use native dispatch. This checkpoint is functional
verification; no performance experiment ran.

- Swift: 9 features pass against the fresh ARM64 SDK: Pie 36 oracle rows, Info
  10 scenarios, Gantt 9 rows, 5 Sankey regressions and 99 cases across 33 entries.
- Strict C17/C18 clients compile and run successfully against that SDK.
- Browser: 59/60 strict checks pass across Chromium, Firefox and WebKit. Pie's
  36 and Info's 12 paired scenarios pass in each engine (144 total comparisons).
- The remaining strict failure contains two WebKit Gantt TypeError message
  differences. Graphs, order, callbacks, aliases and results match. A separate
  semantic test permits only those recorded message pairs and passes 3/3 engines.
  The exact test and failed report remain executable and archived.
- Native Pie, Info, Gantt and Agentflow feature checks pass. Earlier compile and
  assertion failures are preserved alongside their successful follow-ups.

The public audit now matches its one callback-edit/throw case in **4/33 families**
(Sankey, ER, Pie, Info), with **29 still mismatching**. All 22 class-root factories
construct successfully. This audit is one observer, not complete callback or
constructor-reflection coverage.

Build identity: 368 native input paths, 13 SDK files, 444 web input paths and
118 package files are recorded before/after the checks. WASM SHA-256:
`de75423f6cb152abc22793d2f0165c4490c58530303da7d38d0bf7cfb1cae34a`.
The Swift test-only Sendable correction is `c458d27`; Gantt's browser test is
`6225f6d`. See [source bindings](source-binding.json).

Run `python3 docs/parity/2026-10-21-live-render-families/verify.py --git` from the
repository root to check all compressed records and exact source bindings.
This verifies archived evidence; it does not rerun the library. Raw records are
lossless gzip with deterministic headers. No SDK or WASM binary is duplicated.

Limits remain explicit: native active malformed UTF-16/coercion, presentation,
remaining family callbacks, Gantt calendar scheduling callbacks, configured
Agentflow graph-build logging, constructor reflection and native graph-ID
lifetimes. Swift Error identity does not represent throwing JavaScript undefined.
The C15 snapshot ownership correction after this frozen build needs a new SDK
check; this checkpoint does not certify later source. Full API parity and the
performance release gate remain unmet. No 0.3 release.
