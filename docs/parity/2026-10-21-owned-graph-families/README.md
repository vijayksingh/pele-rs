# Nine owned callback families

Frozen runtime `29d7d78`; Swift observer repair `232cd6a`. C4, Requirement,
Railroad, GitGraph, Usecase, Eventmodeling, Architecture, State and Block now
use owned Rust render jobs with shared typed/live geometry. Callback edits,
retained child identities, nested renders, original exceptions and recovery
are checked against the pinned Pele implementation.

- **Native:** 75 distinct integration features pass across scoped family runs,
  including existing SVG/model/error corpora. The final frozen-source run passes
  all 11 callback/resource features.
- **Swift:** 33 distinct selected features have passing evidence against a fresh
  ARM64 SDK. The combined run passed 32 and reported 228 state-observer assertions
  in Railroad. Its SVGs/traces passed. The test incorrectly retained Infinity
  where oracle JSON.stringify produces null; the repaired observer passes in a
  focused rerun. This is not one uninterrupted 33-pass run. All 496 new oracle
  rows and the all-33-entry/99-case regression pass.
- **C:** strict C15/C17/C18 consumers compile and execute successfully.
- **Web:** 45/45 features pass across Chromium, Firefox and WebKit. Nine new
  families contribute 1,488 exact candidate/oracle/fixture row comparisons
  (496 per engine). Collection/text limits, public repair, all-33-entry and
  source-initializer regressions also pass.
- **Identity:** native400 inputs, SDK13 files, web476 inputs and package118 files
  remain unchanged during the respective builds/checks. WASM SHA256:
  `8dbc8acd0386d64ebc093966447d7aa52c05a08489dc26ab7902eb90287e1b62`.

Fresh edit/throw audit matches **26/33** families. Remaining mismatches:
Flowchart, Swimlane, Agentflow, Gantt, Sequence, XYChart and Class. One observer
per family does not establish complete module or API parity. GitGraph live
Array-to-Set/String tag replacement, native inherited-helper reflection and
foreign-method cases remain explicit limitations. Calendar continuations,
configured logging, constructor reflection and unsupported dynamic/UTF16 cases
remain separate work. No performance run or release was performed.

`evidence.json.gz` retains 78 lossless records: original failures, repaired
checks, source/SDK/package maps, paired browser reports and the complete audit.
Original Railroad Swift test source is retained. Native test-only repairs
corrected read imports and Usecase nested-Map JSON serialization. Architecture's
initial signature mismatch was corrected before native execution. Failed
attempts are retained and are not counted as passing.

Run `python3 docs/parity/2026-10-21-owned-graph-families/verify.py --git` to
verify archive integrity, counts, paired reports and source bindings.
