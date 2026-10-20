# Independent implementation review

Reviewed frozen patch c5a80eb5362bf708386908ad2811b06bb21e625ba76838bcf18da222278cd51c and both original/candidate file hashes. No concrete blocker under the documented ordinary-intrinsic contract. No production edits, builds, tests or profiling by this reviewer.

Rust specializes only existing eligible layouts with arity 3/4/6 and every value String or Ref strictly before the current node. Node admission remains after key/layout interning and before event selection/reservation. The event bytes remain 6+5*N for eligible values; all number/bool/null/undefined/function/host/self/future-ref cases retain event11. Generic record/layout fallback is untouched. Scalar-branch coverage concern from the proposal is resolved by excluding all scalar atoms.

JS requires a known layout with accepted arity, consecutive node ID and no partial construction. Object-table limit precedes decoding. recordValue checks String ID against the existing canonical pool and Ref ID against current node ID, reads only private arrays and cannot call user conversion. Fixed locals preserve atom order without a values-array allocation. Literal fields use validated primitive keys in their original order; own descriptors match the prior flags7 path for unique ordinary keys. Existing layout exclusions remain load-bearing. NaN never enters event13 and keeps the original descriptor route.

The completed object is appended, object-ID indexed, and registry-owner indexed immediately, in the same order as allocate, before any next event/import return. No delayed ownership or method initialization. Object creation itself now precedes registration, safe because permitted value resolution uses only prior published nodes and immutable strings, and changed intrinsics/inherited setters are outside the existing contract. Self/forward references deliberately do not use this route.

Canonical bytes 5+10*N are exact: five node header bytes, five key/flag bytes and five Atom bytes per field. refs=N. nodeBytes/nodeRefs and totals update exactly once after registration. Frame bounds, string history, constructor reset/admission and generic event11 stay unchanged. Invalid private tags/truncation may be rejected after a different amount of internal byte reading; no supported public raw byte input targets this private event. Such malformed protocol behavior is not promoted into a new public guarantee.

Only static coherence is established. Existing public graph/descriptor/prepared/fallback/retained/callback slices and matched default-build VM comparisons remain integration gates. Completed-object literals still allocate every final object and perform both WeakMap insertions; no claimed percentage gain follows from the pre-dense allocate profile.

## Rebase after canonical-index rejection

Rechecked refreshed manifest and complete original/candidate hashes. Focused patch remains exactly c5a80eb…; all literal hunks are unchanged. JS original now a9c749d8… and candidate 0d44947d…, restoring both permanent firstStrings/lastStrings Maps; canonicalStringIds absent. Rust remains 10b2fd8c… candidate. Previous oneMap-baseline review artifacts are archived separately. No new literal semantic concern or runtime validation claim.
