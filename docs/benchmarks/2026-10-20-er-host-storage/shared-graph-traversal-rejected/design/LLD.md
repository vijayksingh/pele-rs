# Shared public graph traversal extraction

Private three-file candidate: graph.js, graph-storage.js, package.json. No flat
storage implementation or backend selector is included. ModelRegistry, owner
registration, bridge construction, host-reference tables, value resolution and
legacy reconcileDelta stay in graph.js. Its three ordinary operations delegate
to shared functions using NestedStorage by default.

NestedStorage is an identity sentinel for the inline legacy adapter. Shared
loops use original nested fields, Atom literals, Map/Set arrays and conversion
helpers directly on that branch. They do not route each legacy property/atom
through adapter methods. This is intended to keep the default call sites simple;
it is not proof of JIT monomorphism or neutral performance. Parent must measure
negative controls before adoption.

The alternate storage branch is an internal seam, unselected by ModelBridge.
It reads direct node/property/item/root slots and emits scalar atom payloads to
one operation-local capture transaction. No per-atom wrapper, handle table,
second property mirror or temporary argument payload array is introduced there.
An alternate owner calls shared functions explicitly with its backend; this
candidate adds no public loader option or hidden lazy upgrade.

## Exact boundary

Exports used by existing graph.js: numberValue, stringValue (reexported),
reconcileGraph, matchesGraph, snapshotGraph. ModelBridge method signatures and
returned legacy graph/delta shapes remain unchanged. graph-storage.js retains
the original captured Error constructors/diagnostic set and original conversion/
comparison helpers. ModelRegistry remains one implementation in graph.js.

See INTERFACE.md for scalar storage methods. Additional concrete requirements:

- nodeKind returns existing kind strings; nodeAux returns prototype/length for
  Instance/Array, exact hexadecimal bits for Date, existing Error brand string.
  This is storage metadata, not a live caller property read.
- propertySlots/nodeIds iterate direct numeric addresses/IDs without materialized
  arrays. No per-item wrapper is allocated. Legacy property iteration remains
  the original for-of over its array.
- itemCount is Map pair count or Set atom count; Map itemAtomSlot indexes atoms
  at 2*i / 2*i+1. atomNumberBits returns exact original hexadecimal words only
  for actual Number atoms. The descriptor guard checks tag4 before calling it.
- beginCapture/finishNode/finish are private staging operations. No W node is
  observable through the baseline. Failed capture releases W, but owner identity,
  string and host-reference additions retain their original partial effects.
- capture.root precedes arguments. Indexed argument writes preserve original
  fixed length and holes. Nested values.map remains exact, including its custom
  map/species behavior. Future flat owners supply ordinary argument arrays under
  the existing ordinary-intrinsics contract; no generic Array monkey-patch
  equivalence is claimed for an unselected adapter.
- future flat owner provides hostReferenceId with the same table limits and
  insertion behavior as ModelBridge.hostReference, without an Atom wrapper.

## Preserved order

Snapshot checks registration prototype descriptors first. Then root, ordered
arguments, dynamic all-history classification, LIFO traversal. Keys/descriptors,
array-length exclusion, Error diagnostic accessor handling, work charge, key
interning, value classification and flags keep their original sequence. Own
properties precede collection items. Map keys precede values. Final Date reads,
repeated Error brand reads, late Array.length and prototype classification remain
in place. bridge.graph is replaced only after traversal and string finalization.

Reconcile sets bridge.graph and string state before shells. It allocates all
missing shells before restoring any existing node. Only these new shells enter
registry.owners. Deletion, Map/Set replacement, Date, Array length and ordered
properties keep their order. Descriptor suppression retains exact numeric NaN
bits; hostReference resolving to NaN follows defineProperty as before. Root is
resolved last. Existing symbol keys/prototypes and partial-publication behavior
remain unchanged. Full ordinary publication still restores every node.

matchesPublishedGraph retains prototype checks and live own-descriptor order,
canonical last string IDs, numeric bits, collection intrinsic sizes/iteration,
and its existing refusal to treat host references as unchanged. No hash replaces
comparison and no observer is skipped because a retained object already has an ID.

## Packaging and verification

package.json explicitly includes graph-storage.js so graph.js's import exists
in the npm-packed artifact. No generated package has been changed here.

Private checks: Node syntax checks for both runtime files; frozen patch/manifest
hashes; patch applicability. Parent owns focused public graph/binary/callback/
provider/error integration, full necessary browser gates, packed import runtime
verification, and representative negative-control timings. No browser, build,
benchmark, integration test or production edit is performed by this preparation.

Private flat-storage code can consume this seam only after independent source
review. It must not make staged headers public earlier or introduce native
per-property boundary calls. Selection of complete Sankey host storage remains
a later, explicit, all-lanes gate.
