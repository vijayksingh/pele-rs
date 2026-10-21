# Accepted native baseline authority

Internal browser architecture following `7417fed53e34f482f726dd07a12fec4dd32b9425`. Native admission and mutable-model policies stay unchanged. Performance requires independent VM measurement.

## Authority and host equality

Renderer-private WeakMap holds `{revision, token, graph, root}`. Token is a fresh opaque object, never a counter. It proves which bridge baseline native accepted; it does not certify that live JS objects remain unchanged. Every zero-argument shortcut still executes the existing complete `matchesPublishedGraph`, then rechecks receipt identity, revision, token, graph and root. `confirmedGraph` still becomes false after detached jobs.

Shared traversal replaces the token before every snapshot, including prototype checks and attempts that fail after appending object/string history. It also invalidates before reconcile writes. Both reconcileDelta entrypoints invalidate before stale/type checks; LayoutGraph reconcile invalidates before its type gate. Constructor invalidates before registration/initializer effects. Host-function snapshots and delta publications use these same entrypoints.

Completed snapshots are WeakMap-bound to their actual returned input, exact graph/root and entry token. Completed publications are bound to the exact reconciled graph/root and reconcile invocation token. Reentrant superseding observations invalidate that proof even when the outer graph pointer appears unchanged. Successful generated initialization records the same proof; generic initialization uses reconciliation. No outer publication can mint authority from a newer nested token.

## Grants and invalidation

Complete publication grants only with completed-publication proof. Source ingress successful native apply grants only with completed-observation proof: its sealed source capability accepts that exact graph. Other providers obtain authority through full publication. The existing logical no-op branch can renew only a previously valid receipt and an unsuperseded completed observation, after its existing exact delta/root/count comparison.

Mutating invoke and legacy render clear authority before native dispatch. Failed apply leaves the observation token changed; failed publication cannot grant. The readonly finish captures the CURRENT receipt (including any nested helper publication), accepts only its exact next native revision and stable proof, and updates before materialization. Failed finish/malformed returned ACK clears authority. Factory execution cannot overwrite an outer receipt afterward: a factory throw alone cannot mutate native state; live edits remain subject to next mandatory matching; a nested helper may retain its newer accepted publication. Original callback/factory throw priority stays unchanged.

Current native prepare/finish/schema validation, full canonical size and metadata limits still execute. No graph-pointer-only shortcut, failed-history rollback, pure-resource assumption, skipped dirty comparison, new API or source flag.

## Feature proof and limitations

One public browser integration compares normal/raw results, objects, aliases, descriptors, errors and subsequent operations against actual pinned Pele. It counts real prepare/finish/apply calls. Cases: three unchanged renders; callback mutation+original throw; failed proxy capture with retained detached history; registered-prototype rejection/recovery; partial helper failure+repair; result-factory nested helper+original throw. Reports persist before assertions.

The unchanged fixture fails on the predecessor package: three real readonly jobs/finishes but two source applies, instead of zero. Existing comprehensive callback, provider, schema, full graph and budget suites remain integration gates. This feature does not independently force every schema/combined-budget failure or every possible Proxy reentry.

## Terminal receipt release

Explicit disposal first rejects busy/native-busy/detached operations. Only after that gate does it delete the renderer's accepted-baseline receipt. `releaseDisposedBridge` deletes the observation token and completed-publication proof before clearing existing transport fields. This is terminal cancellation, not a new observation: it allocates no replacement token and cannot grant acceptance.

A completed-publication WeakMap value contains `{token, graph, root}`. Weak keys do not release that value while their bridge remains reachable. Registration WeakRefs can keep a dereferenced bridge alive through the current JavaScript job; an externally retained native-created leaf also keeps its bridge reachable through `registry.owners`. Clearing bridge fields alone therefore retained the entire public root and canonical graph through the publication proof. Explicit proof deletion removes that additional edge without changing the retained leaf, its descriptors, or helper routing.

`completedObservations` remains keyed by ephemeral operation inputs. The bridge has no edge to those inputs; disposal's busy gate excludes active operations. No cleanup index is introduced. Registry ownership, shared methods/prototypes, registration WeakRefs and disposed-receiver routing remain unchanged. Rejected busy disposal leaves every receipt intact.

Public lifecycle regression retains only a leaf and a WeakRef to its root, crosses a browser job boundary, then requests browser garbage collection. Parse-only, rendered and helper-mutated lanes must release the root while keeping the leaf usable and borrowed-helper routing intact. The predecessor package retained all three roots. This establishes terminal reachability; performance and GC-cost attribution require separate matched measurements.
