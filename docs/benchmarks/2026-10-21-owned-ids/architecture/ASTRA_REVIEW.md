# Independent owned-index review

No concrete source counterexample found. Patch bf845aa0e1e715919fefa6822b293c009723ef5f2a8be0c97ea44b90de80256f. Both applied hashes and original11f Git blobs verified.

All three ModelBridge ID writes append the exact object to bridge.objects before insertion; active object history is append-only. Failed observation and native truncation retain this same history. Map adds no reachable key absent from the preexisting object array. Only object keys enter the index; no size/iteration/delete behavior is used. External registry owners remains WeakMap.

Disposal clears objects and ids synchronously in the same no-callback release block. Borrowed-helper routing rejects disposed direct owners and selects a live registration owner before invoking its separate index. Raw/caller-object observation still does not register registry ownership. Closed bridge cycles add no independently rooted reference; existing escaped aliases retain the same already-strong history until release.

Scope is ordinary supported intrinsics/public lifecycle, not mutation of internal ModelBridge fields or replacement of Map/WeakMap methods. No runtime/GC/performance claim; no builds/tests executed by reviewer.
