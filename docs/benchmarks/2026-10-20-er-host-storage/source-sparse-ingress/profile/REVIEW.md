# First-edited profile: prepared, not executed

Frozen e148 / ef9 / pinned PeleJS, Sankey medium fixed. Exactly three CPU windows, 2,000 ms each, 250 µs sampling, 500 ms warmup. No latency comparison or extra stage.

`profile-source-first-edit.mjs` embeds verbatim bytes 10454..15345 of executed `compare-first-use-web-vm.mjs` (SHA5469078): taskSignature, scenario and verify. That includes creation of the cyclic benchmarkExtension, parse, complete render, identity/cycle verification and finally disposal. The original work closure is copied byte-for-byte too. The new outer loop only repeats that closure for the specified CPU window; no parse/edit/dispose exclusion. Both extracted ranges are hash-checked against the preserved original script before browser launch.

Same fixed measurer and workload/options. Verbatim verify runs before and after all three windows, compares whole SVG/dimensions and cycle/extension identity against JS. All three untimed activation probes (8+16+12) precede the profile in separate closed browsers. Package/oracle/harness/cases hashes checked before and after; default exports only. Cleanup closes both renderers and browser.

Attribution retains raw profiles and original native names. Disjoint leaf roles and deduplicated ancestor unions are separate. Added observable symbol groups for prepare_overlay, admit_observed, Counter, DecoderBudget, CapturedLogical and imported/string readers. Missing/inlined frames remain unassigned: these samples cannot count duplicate passes or establish conversion call counts. Parser rule matches parser functions, never IndexedSankey generic render types.

Node syntax and local package preflight passed. A first local preflight rejected a mistyped CLI control digest before browser creation; corrected to frozen digest. No VM/browser execution yet. Await parent review GO.
