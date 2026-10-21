# Six owned callback families

Frozen runtime `6bebaf3`; Swift test repairs `735fc4a`. Kanban, Treemap,
Mindmap, ER, Cynefin and Wardley use Rust-owned render jobs and shared geometry.
Resources can edit current models, invoke helpers, nest rendering and throw
original errors without restoring stale state over their changes.

- **Native:** 46 distinct integration features across scoped family runs. A final
  frozen-source run passes all eight new features, including collection/text
  limits and repair. Existing family SVG/model/error corpora pass.
- **Swift:** all 24 selected features have passing evidence against a freshly
  built ARM64 SDK. The combined run passed 23 and failed one test's incorrect
  `PeleError` cast; the corrected `CompatibilityError` assertion passes in a
  focused rerun. This is not a single uninterrupted 24-pass run. All 318 new
  oracle scenarios and the all-33-entry/99-case regression pass.
- **C:** strict C15/C17/C18 clients compile and execute successfully.
- **Web:** 54/54 features pass across Chromium, Firefox and WebKit. The six
  families contribute 954 exact same-engine candidate/oracle/fixture comparisons
  (318 per engine). Existing ER host/prepared routes and all-33-entry regressions
  pass. Constructed ER also verifies joined-text limits before allocation and
  recovery in every browser.
- **Identity:** native383 build inputs, SDK13 files, web459 inputs and package118
  files remain unchanged during their respective builds/checks. WASM SHA256:
  `37bf7faf34da3375cce22e4ec3832e67d017d856e94605daf47f527341a5442a`.

The fresh 33-family edit/throw audit matches 17 cases; 16 remain mismatched.
This is one observer per family, not a count of complete modules or proof of
full API parity. ER's new native/Swift owned route is now exercised separately
from its existing web host route. Remaining callback families include Flowchart,
Class, State, Sequence, Gantt, C4, Requirement, Usecase and others. Configured
logging, calendar continuations, reflection and unsupported dynamic/UTF16 cases
remain separate work. No performance work or release ran.

`evidence.json.gz` retains 81 lossless records: original compile/assertion
failures, repaired checks, source/SDK/package maps, paired browser reports,
resource-limit reports and the full audit. Native test repairs corrected Atom
reference comparisons, property-name matching and declared Array lengths;
Swift repairs corrected comparisons, actor capture and the error assertion.
The failed attempts are retained rather than counted as passing.

Run `python3 docs/parity/2026-10-21-owned-domain-families/verify.py --git` to
verify archive hashes, counts, paired reports and source bindings.
