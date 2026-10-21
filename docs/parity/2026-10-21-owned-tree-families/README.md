# Five more owned callback renderers

Source `653f127` adds Venn, Ishikawa, Timeline, Journey and TreeView to the owned
render lifecycle. Each shares typed/live geometry. Resource callbacks preserve
retained collections, partial edits, nested rendering and original errors.

- Native: 45 distinct feature checks pass across the five implementation slices.
  Final focused checks pass 6/6, including three callback expansion/repair cases
  preserving the existing text budgets. Full prior family corpora also regress.
- Swift: 17 features pass on a fresh ARM64 SDK, including all 234 new oracle rows:
  Venn22, Ishikawa48, Timeline52, Journey48 and TreeView64. Earlier callback
  families and the 99-case/all-33-entry regression pass in the same batch.
- Strict C15/C17/C18 clients compile and run successfully against that SDK.
- Chromium/Firefox/WebKit: 39/39 features pass. All 234 new scenarios compare
  complete output/state/traces against actual pinned Pele in each browser
  (702 comparisons), alongside prior chart/Pie/Info/all-entry regressions.

The fresh **web** audit matches one callback edit/throw case in **12/33 families**;
21 still mismatch. All 22 class-root factories construct. ER already passes this
web observer through its separate hosted route; native/Swift ER owned callbacks
remain separate work. These counts do not certify full API parity.

TreeView required a generic job-local opaque object identity operation so a
mutated node ID cannot break guide lookup. Host tokens are bounded; native
owned values remain pinned to prevent allocator-address reuse. No addresses are
exposed. Native graph-coordinate lifetime restrictions remain explicit. Its
first native feature run used the wrong test icon hook; the original failure is
preserved. Corrected hooks exercise the actual HostResources icon channel.
An earlier bridge compile attempted Timeline before its dispatcher was merged;
that failed integration attempt and successful follow-up are also preserved.

The maps bind 376 native inputs, 13 SDK files, 452 web inputs and 118 package
files. Source/SDK/package/archive remain unchanged after checks. WASM SHA-256:
`be661311e5961bb8793554e01d598403b4f2d8b523e1f60c53ea28410c6bdaf3`.
ARM64 execution does not prove physical Intel or macOS14 support. Later source
needs its own matching artifacts. No performance experiment ran; no 0.3 release.

The lossless compressed bundle contains 43 records: complete logs, before/after
maps, full paired browser reports and the fresh audit. No binaries duplicated.
From repository root, verify integrity and source bindings with:

```sh
python3 docs/parity/2026-10-21-owned-tree-families/verify.py --git
```

Verification reads archived evidence; it does not rerun the library. Remaining
work includes other family callbacks, native ER, Gantt calendar callbacks,
configured Agentflow logging, constructor reflection, active malformed UTF-16,
and arbitrary coercion/dispatch. Swift Error identity does not represent throwing
JavaScript undefined. Earlier exact WebKit Gantt diagnostic differences remain
recorded in their own checkpoint.
