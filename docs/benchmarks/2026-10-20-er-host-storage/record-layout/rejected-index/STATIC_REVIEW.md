# Private host object-ID indexes

Static review finds no supported-contract blocker. This candidate changes only three allocations in `bindings/web/host-er-store.js`: initial `ids`, reset `ids`, and outer-operation `temporary.objectsIds` become `Map`. Registry owner routing, root documents, prototype/function registrations and raw disposal tracking remain weak. No production edits, builds, browser runs, tests or performance measurements by this reviewer.

## Lifetime and identity

`allocate()` and admitting `atom()` push each key into `objects` before installing its numeric ID. No code individually removes entries from that array or index. `audit()` visits every historical object, including detached nodes, and keeps last admitted accounting for objects whose prototypes later become unsupported. Using a strong index adds no newly reachable object: every key already has a strong edge from the same owner. Object identity equality is identical for Map and WeakMap; assigned numeric IDs, lookup precedence and budgets are unchanged. No object is wrapped and registry publication remains eager.

`disposeConstruction()` replaces both objects and IDs together, including direct-grammar fallback. Old registry weak owner associations are unchanged; an unexposed discarded object's registry key does not make it live. `dispose()` and failed provisional raw `abandonCallerRoot()` null both fields. A live owner already retained the entire array; an unreachable owner and its map/array cycle remain collectable. Finalizers and registry WeakMaps are untouched. This is a reachability argument, not measured heap or GC behavior.

`temporaryId('objects', value)` likewise pushes into `temporary.objects` before setting the map. Nested resource operations share that temporary table; existing temporary-before-permanent lookup precedence survives even if nested admission gives a temporary object a permanent ID. Outermost `finish()` clears the entire temporary container in finally, including error paths. Phase/depth checks and callback schedules do not change. No iterator or size query is added to either index.

## Root and error stages

Null/undefined are handled before classification. Numbers, booleans, strings, functions and symbols never enter these object indexes. Unsupported object prototypes and unregistered functions still route through the existing host-reference maps. Accepted Array/Map/Set/Date/Error/plain/registered-instance objects retain classification calls and their order. Proxies still execute the same prototype/brand/descriptor operations before admission; Map key insertion itself performs no coercion or property access.

Raw provisional adoption still uses `documents.set(model, entry)` as its weak-key gate. Primitive rejection, registered/nonregistered Symbol behavior, first config serialization, zero/+13 frame gates, ownership commit, second serialization and later importer failures therefore retain their existing stages. No primitive is admitted merely because Map accepts primitive keys: classification still controls every private index insertion. Arbitrary thrown callback values, first-error precedence and state retention remain untouched.

Normal intrinsic semantics assumed, as throughout this adapter (it already dynamically uses Map for strings, host values, schema and registration tables). This patch does not promise equivalence under user replacement of Map/WeakMap constructors or their prototype methods; those can observe different internal containers. Allocation failure timing can also differ. No new universal Proxy or hostile-intrinsic guarantee is introduced.

## Why measure this candidate

Current fresh-record profile supplied by parent attributes 25.17% self time to JS allocate, 12.01% to GC, and 2.02% to text. Allocate is inclusive of several operations; these percentages do not isolate WeakMap cost and cannot predict a speedup. Candidate replaces one of two eager ephemeron-table insertions per constructed object with a normal Map insertion; total insertion count stays two. Map capacity/memory policy can be better or worse. Ordinary parse, retained/source negative controls, first-use and eventual heap evidence must decide adoption. The UTF8 string-blob proposal remains deferred until inclusive string attribution supports its complexity.

This is **different from ownership-index/**: that older graph.js experiment combined owner and ID in one per-object `{owner,id}` wrapper and had inconclusive timings. Here no wrapper, no registry redesign, no delayed indexing, no graph.js edit, no reuse of those timings. Only ER host storage changes; native Rust/Swift/WASM artifacts need no rebuild for this JS-only candidate.

Existing public feature slices can validate alias/descriptor identity, post-edit retained history, nested callbacks, arbitrary throw retention, raw accepted roots/primitive stages, constructor fallback and disposal. No new implementation-level tests are proposed. Browser parity and matched VM timing are parent integration gates, not completed claims.
