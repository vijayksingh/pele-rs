# Independent canonical-string-index review

No concrete blocker under the existing ordinary-intrinsic contract. Read-only source/patch review; no production edits, builds, tests or timing.

Induction holds: constructor/reset start both permanent maps empty; intern alone appends strings and writes the same text/ID to both maps, and a hit appends nothing. No raw string-table import writes duplicate slots into HostErOwner. Caller roots and audit traversal use intern; all first=true call sites now use the one canonical map. ModelBridge is untouched, where duplicate imported table strings can make first/last distinct.

Temporary strings remain separate and precede permanent lookup in observedString. Nested admission of a temporarily observed string therefore keeps outer operation IDs stable. All UTF16/table budgets and string IDs preserve their checks/order. Constructor, discard/reset, disposal and failed raw abandonment all initialize/clear the replacement index correctly. Generic findString/intern/internText ABI names and returned values remain unchanged. There is no new lazy or first-method work.

One map allocation and one duplicate set/entry per new permanent string disappear; total map or heap savings and latency remain unmeasured. Recovery after injected Map intrinsic exceptions or allocation failure is not promoted into a new contract. This review does not reuse the rejected object-ID Map timings.
