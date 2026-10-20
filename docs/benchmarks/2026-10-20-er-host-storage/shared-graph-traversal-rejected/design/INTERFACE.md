# Shared traversal extraction: pre-code interface

Files: graph.js and new graph-storage.js only. ModelRegistry and ModelBridge
ownership/host reference helpers stay in graph.js. Move existing snapshot,
reconcile and matchesPublishedGraph control flow into exported shared functions.
Default storage is a singleton NestedStorage identity; no flat selector exists.

Shared functions take `(bridge, graph/values, changed/delta, storage)`. Each uses
`const nested = storage === NestedStorage`. For the default backend keep direct
`graph.nodes`, `node.properties`, `property.value`, original Atom literals,
Map/Set arrays and legacy string construction. Do not route every default
property through an adapter method. The scalar branches are inline alternative
accesses in those same loops, not a copied observer/publisher implementation.
Default calls remain monomorphic. There is one storage identity per bridge.

Reader surface for future flat storage uses node IDs, absolute property/item
slot offsets, and root/argument atom slots. These are direct storage addresses,
not newly allocated wrapper handles. Functions:

- stringCount/stringValueAt (flat epochs already retain JS strings), nodeCount,
  nodeIds iterator; legacy keeps its original UTF16 decoder;
- rootSlot(graph), nodeKind(graph,id), nodeAux(graph,id), propertyCount(graph,id),
  propertySlot(graph,id,index), propertySlots(graph,id) iterator of direct offsets,
  propertyKey/Flags(graph,slot), propertyAtomSlot;
- itemCount(graph,id), itemAtomSlot(graph,id,index), atomTag/atomPayload(graph,slot).

Payload is a scalar JS value: Number for numbers, u32 for IDs, zero otherwise.
Canonical number bits remain in storage; reading a number for JS publication uses
the existing DataView conversion. Matching NaN additionally compares raw words/
hex with current value bits; backend provides atomNumberBits. No per-atom object.
Methods are local JS calls, not WASM imports. Default branch uses original nested
fields and numberBits/numberValue without per-property dispatch.

Observer surface is one capture transaction object per snapshot:

- beginCapture(baseline,delta) -> capture (not published);
- capture.root(tag,payload); capture.beginArguments(length);
  capture.argument(index,tag,payload) (holes remain unwritten);
- capture.beginNode(id); capture.property(keyId,flags,tag,payload);
- capture.beginItems(kind); capture.mapEntry(keyTag,keyPayload,valueTag,valuePayload);
  capture.setItem(tag,payload);
- capture.finishNode(id,kind,aux); capture.finish(strings) -> graph/result metadata.

Flat atom classifier returns a scalar payload and sets an operation-local `tag`
variable. Caller copies the tag before classifying another atom, especially Map
key before value. Property/item/root destination is supplied immediately; no
scratch Atom handle table or intermediate generic graph. Default classifier
returns the exact existing Atom literal, so legacy loops retain allocation shape.
Flat numbers pass the original JS number directly to storage, which captures bits
once. Host references use an ID-only owner primitive in flat mode; default uses
existing bridge.hostReference and its existing exception/budget behavior.

Snapshot preserves dynamic all-history classification even when result is not
stored. Pending traversal/order is unchanged. Nodes enter capture only after
all properties/collection/Date/Error reads succeed. Incomplete capture drops;
owner object/string/host-reference side effects remain; bridge.graph commits only
at the same final point. Error brand is not cached and Array.length stays late.

Publisher/matcher share loops and exact original order, using direct nested
accesses or flat slot primitives at each read. No flattened materialized view,
per-property Vec, changed ownership semantics or new lifecycle. Legacy delta
method remains unchanged until a flat backend later implements its own storage
update; ordinary full graph functions are the current extraction scope.

Before code: independent interface review. After code: frozen focused patch,
manifest and source review. Parent owns runtime/browser/performance gates. No
performance claim: default branches are designed to avoid dispatch overhead,
but only representative negative controls can show whether extraction is neutral.

Reviewed clarifications: flat argument capture preserves fixed initial length and
indexed holes via HasProperty checks; its owner supplies ordinary argument arrays
under the existing ordinary-intrinsics contract. Legacy retains values.map(atom)
exactly, including its existing custom-map/species behavior. Do not introduce a
temporary array of flat payloads. finishNode commits only after final kind/aux
reads finish. NaN descriptor suppression requires original numeric atom bits; a
hostReference resolving to NaN is not a numeric atom and stays on defineProperty.
Default monomorphism is intent, not a proven JIT optimization across both backends.
