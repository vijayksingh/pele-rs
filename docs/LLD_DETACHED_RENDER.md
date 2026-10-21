# Detached live rendering

Info and Pie use the same authenticated owned-job lifecycle as Sankey. Typed
and host inputs share each family's geometry through a phase-source interface.
Info captures title before layout and reads accessibility after resource calls.
Pie captures section iteration for each draw, reads showData per legend label,
then title/accessibility at their upstream phases; responsive draws recapture
the current collection. No model borrow spans a resource callback.

`LiveRenderRead::snapshot_iterable` adds bounded host iteration. JavaScript
retains yielded values; C15 creates owned pair sequences from a short Map read,
including owned UTF-16 strings. The default rejects unsupported readers explicitly.
No C ABI or JavaScript diagram algorithm is added. Existing malformed UTF-16,
coercion and native graph-coordinate lifetime limits remain separate.

The native reader owns captured UTF-16 through snapshot-local `Arc` values.
Repeated Map string IDs share one retained allocation, so copying entry pairs
cannot multiply admitted catalog text before scalar validation. Each snapshot
has a fresh cache; later graph reads observe current values.

Generic live operations also include indexed membership (`has_index`), ordinary
object spread (`shallow_copy`) and dynamic `map_get`. Their defaults explicitly
reject readers that have not opted in. WASM requests numeric operations5/6/7;
JavaScript executes `key in value`, `{...value}` and `value.get(key)`. Rust
chooses all operands, ordering and family policy.

C15 implements declared graph membership, captures own enumerable fields into
an owned immutable record, and looks up admitted Map keys with SameValueZero.
Captured records preserve child references; no document guard crosses resource
calls. Native custom indexed prototypes, accessors and overridden Map get are
outside this carrier's declared graph contract and are not silently emulated.
String spread retains the existing eight-million-property work bound. These
operations do not add a C ABI or host-side diagram algorithm.

Internal builtin capability for Sankey resource callbacks. The [oracle audit](compatibility/sankey-resource-callbacks/AUDIT.md) records the previous restoration, read-order and reentry mismatches. This design is not a claim of complete compatibility or improved performance.

## Boundary

Parsing, geometry, responsive fitting and SVG remain in Rust. Native/C/Swift use the ordinary typed adapter. A private phase-reader interface lets the same Rust renderer consume live browser values without borrowing its mutable document across callbacks. JavaScript supplies generic property, index and string operations; it contains no diagram algorithm.

Only the concrete builtin creator can supply this capability. Hidden public carriers have private constructors; erased dispatch checks creator TypeId for both capability and input. Matching a schema/type name or forwarding a builtin carrier through another provider is insufficient. Other providers and families retain existing render/publication paths.

## Owned lifecycle

```text
PublicDocument::prepare_detached_render(revision, request)
    -> Result<Option<DetachedRenderJob>, PeleError>
DetachedRenderJob::run(self, live, resources)
    -> DetachedRenderCompletion
PublicDocument::finish_detached_render(completion)
    -> Result<DetachedRenderOutcome, PeleError>
```

Prepare performs the existing options/revision/structure checks before dispatch. It returns no job for an unsupported concrete provider; fallback is allowed only then. A dispatched operation is never replayed. The job owns immutable input, captured registration/config and a one-use lease. Neither document references nor graph borrows escape prepare. Sankey's immutable legacy projection uses Arc ownership so nested helpers can replace document state without invalidating an outer job.

A lazy Arc/Mutex book records checked frame IDs and pending revision reservations. No lock is held through callbacks. Jobs hold weak owner leases, finish in nesting order, and release reservations on drop. Ordinary advancing operations at revision r with k pending frames check r+k+1 at their original overflow stage while advancing only to r+1. Finish advances the current revision after nested operations, never the revision at entry. Owner drop safely invalidates remaining leases.

Normal and raw jobs reuse existing render orchestration, output-limit, resource scope and presentation policy. Errors remain job-owned completions, so resource failure still reaches finish. Native finish validates the current native snapshot and returns only a short-lived graph borrow for transport admission.

## Live read phases

- Every draw obtains current node count and link endpoints/index/value scalars. It does not eagerly acquire node labels. Geometry stays fixed for that draw; responsive retries obtain fresh geometry.
- Each node acquires its current ID before its width callback and retains that exact value for its rectangle and label. Color lookup remains a separate current ID read at the original phase.
- After node measurements, links resolve their captured geometry indices against current public nodes. Source/target labels and color reads retain upstream ordering.
- Title is captured when its label pipeline starts. Accessibility fields are read after title resources, including distinct truthiness and subsequent property reads.

Opaque host values survive callbacks without premature stringification. ToString occurs only at actual coercing expressions; strict string uses retain their first native JavaScript TypeError. A job-local presence/value channel preserves original object, symbol and undefined throws. A caught nested failure cannot poison its parent. Public RangeError conversion remains unchanged.

Accepted input keeps its existing structural, schema, text and transport budgets. Live strings and callback-expanded geometry require bounds before allocation. Finishing does not observe/admit all callback writes: upstream does not validate them on render exit. The next ordinary helper/render observes them through existing ingress.

Rich-label icon scanning also preserves the pinned shared regex cursor across nested resource renders. A thread-local UTF16 cursor uses short Cell accesses; no mutable borrow or lock crosses callbacks. Nested reset, failure and backwards-slice behavior match the JavaScript expression without duplicating layout or SVG code.

## Browser protocol

`MutableDocument.prepare_detached_render_numbers` and `prepare_detached_render_raw` return an optional independent job. `job.run(root, read, width, icons, clock, inlineIcons, dispatcher)` consumes its core job and stores completion. `MutableDocument.finish_detached_render(job)` consumes completion during a short document borrow.

The acknowledgement exposes its decimal revision before a separate `materialize()` invokes the existing result factory. Its private packet is `[1, revisionString, graphBytes, status, payload]`. `graphBytes` counts the canonical frame with a zero metadata length. Error status0 carries the exact serialized public Result error; success status1 carries the existing `[1, metadataBytes, renderedValue]` packet. Exact legacy combined graph-plus-metadata byte admission remains required even though no graph is transferred. Success packet admission checks `graphBytes + metadataBytes` against the unchanged limit.

The facade releases its native busy guard only during job execution and result materialization. Active detached frames prevent disposal. Every dispatched frame finishes or is dropped safely. Revision acknowledgement precedes materialization because a factory can itself call a helper; that newer revision must survive.

The facade never reconciles an outer render snapshot over callback mutations. It retains the latest bridge baseline, acknowledges the current revision and marks observation unconfirmed. Nested helpers continue their ordinary apply/invoke/publication, including partial writes on failure. Writes after an inner helper remain visible after the outer render.

Failed native JavaScript primitives use a typed `JavaScriptFailure` resource operation. Portable hosts retain the existing PublicError wire and message. Browser hosts derive the current engine's name/message by running the intrinsic on an isolated descriptor-equivalent array; caller mutations and callbacks are never replayed. Resource decorators forward this operation. Strict string failures execute the pinned expression form because WebKit includes that expression in its TypeError message.

## Typed browser read commands

The browser import uses numeric commands0=get,1=index,2=ToString,3=requireString and context1=t/0=e. Its typed WASM signature passes integers directly, rather than converting control strings for every property read. The job-local Reader interns property/method JavaScript string handles on first use. It retains at most128 keys and16KiB of owned key text; further keys use the existing uncached conversion, without a new public rejection. It retains no public field values and shares no cache across jobs.

The core phase-reader contract, property-read order, callback boundaries, original-error channel and UTF16 budgets remain unchanged. String operands still return before method validation; wrong-type string use preserves the exact pinned expression. This layer changes transport metadata only. Independent VM profiles attribute33.61/35.05% of candidate medium/large retained samples to live control-string decoding, so the change targets measured repeated conversion rather than a presumed Rust speedup.

## Verification boundary

Integration controls cover owner/lifetime/revision behavior and default typed rendering. Browser comparisons use actual pinned Pele in the same engine and compare complete SVG, dimension bits, callback traces, ordered descriptors, aliases, retained references, partial helper effects and original arbitrary throws. Existing native frame/provider tests remain independent controls. Generic JavaScript Error stack locations remain host implementation details and are retained separately in reports; PeleError source line, column and snippet remain strict comparison fields.

Custom coercion, getter/Proxy behavior, sparse/malformed arrays and callback-created lone UTF16 surrogates require additional proof before a blanket compatibility claim. Rust SVG remains UTF8; replacing lone surrogates silently is not acceptable. Correctness and performance gates remain separate, and 0.3 stays blocked until both pass.
