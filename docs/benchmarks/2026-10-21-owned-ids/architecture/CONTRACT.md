# ModelBridge document-owned object index

Private two-file candidate against `11f27516d98c3d89799a7fe2065eec0bd4e147cf`. `ModelBridge.ids` becomes `Map`; disposal nulls it alongside `objects`. Registry receiver ownership remains WeakMap. HostErOwner/CapturedOwner and all traversal, admission and publication ordering remain unchanged. No performance claim.

## Strong-reference domination

All library writes to ModelBridge.ids are exactly these three paths:

- `source-initializer.js:57`: push the fresh literal/collection/root into `bridge.objects`, then set its ID, then register its registry owner.
- `graph-storage.js:167`: generic reconciliation allocates one object, pushes it before ID registration, then registers its registry owner.
- `graph-storage.js:342`: observation of a previously unseen accepted object pushes it before ID registration. Objects failing brand admission remain host references and never enter ids.

Every ID key therefore already has a strong reference from the same bridge's objects array. The array is append-only while active: no slot assignment, delete, pop, splice, truncation or array replacement exists after constructor initialization. Reconciliation of fewer native rows leaves historical objects retained; snapshot explicitly revisits all objects. Failed captures leave the newly observed objects and IDs retained together. Delta rejection/reset changes graph, not objects or ids. Readonly callbacks do not change this invariant; later observations use those same append paths.

Raw adoption uses a fresh empty ModelBridge then snapshot; imported caller objects follow the same append-before-set rule. No registry ownership is added by that observation, preserving old last-live/borrowed-helper behavior. Failed constructor/adoption may leave a bridge reachable temporarily through registration WeakRefs or externally escaped objects; its preexisting objects array already retains exactly these Map keys. Without any external root, changing WeakMap to Map adds no new reachable root: the bridge/object cycle remains collectable.

## Disposal and routing

`full.js:releaseDisposedBridge` nulls graph/catalog/objects and now ids in the same synchronous no-callback block. Thus an externally retained object whose registry WeakMap value keeps its disposed bridge alive cannot retain detached history through the new Map. Disposal rejects active native/readonly work first. No index use can resume on a disposed owner through the public dispatcher: ModelRegistry dispatch excludes disposed direct owners and selects the appropriate current live owner before invoke. Helpers borrowed from an old receiver use that live owner's distinct index; registry.owners and registration/function identity stay intact. Renderer close/provider replacement behavior is unchanged.

All keys are objects, so Map and WeakMap identity equality/get/set behavior coincide for these accesses. No iteration, size dependence, primitive key use, deferred index construction or per-object metadata wrapper is introduced. The audit concerns library-managed public APIs, not callers mutating internal ModelBridge fields or replacing built-in collection intrinsics.

## Evidence / execution gate

Source audit searched all bindings/web object-array and ID-index writes. Existing `disposal.spec.js` covers externally retained aliases/descriptors, disposed borrowed receivers, shared cross-schema helpers, no-live rejection and active disposal blocking. Existing accepted-baseline/source-layout/callback tests cover failed observation with detached history, raw adoption, partial helper failure, repair and nested operations. No new implementation test or GC-timing assertion added.

Private node syntax and git apply/whitespace checks pass. No browser/Node feature execution, build or benchmark performed. Parent should run relevant public feature slices and matched VM comparisons; previous unrelated ER index experiments are not evidence of this candidate's speed.
