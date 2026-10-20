# Native Sankey resource callback reentry

Status: implemented and validated for the focused native callback slice. Fresh host SDK, 31 distinct Swift integration tests, six C clients, and the 168-comparison Swift API consumer passed. This is scoped evidence, not complete native callback parity. This is functional compatibility work, with no performance objective. The browser/WASM implementation and immutable source-rendering APIs remain unchanged.

## Missing behavior and first feature

Pinned Pele permits a Sankey text-measurement or icon callback to call a helper on the same model and to render that model again. The current Swift mutable owner rejects these calls in `MutableDocument.withHandle`; the C operation also holds its document mutex across resource callbacks. The Rust public engine already has a sealed, creator-checked detached job with revision reservations and LIFO completion.

The first complete feature exposes that existing lifecycle through C and uses it for Swift mutable normal and prepared/raw Sankey rendering. It covers same-document helpers, ordinary snapshot-edit-apply, nested rendering, partial helper errors, original resource errors and disposal refusal. Other families and custom providers retain their existing path. This does not establish arbitrary JavaScript object/coercion semantics or full native presentation parity.

## Ownership and call sequence

1. Decode the existing options/numeric packet at the existing stage. Acquire the document mutex briefly and ask its metadata-only detached capability. Unsupported providers return an unselected result, allowing the existing operation path without a second options decode in the selected implementation.
2. Under that short borrow, call `prepare_detached_render(expected_revision, request)`. The job owns its input, captured registration/config and affine reservation; it holds no graph/document borrow. Release the mutex before returning the opaque C job.
3. Dispatch the owned job synchronously. The reader retains only the owner pointer and operation-local owned values; every graph read takes and releases a short document lock. No mutex guard, borrowed graph/string, host binding `RefCell` guard or Swift exclusivity access survives any resource callback. Clone bound callable `Rc`s under short borrows before dispatch, as the existing host-function layer does.
4. Store a completion for every dispatched core result, including operation errors. Nested operations use the actual current revision. A nested render gets its own job, resource/error context and completion; finish order is LIFO.
5. Reacquire the document briefly and finish against its latest revision. Serialize the graph-bearing result using existing v12 bounded metadata and canonical combined graph/metadata admission. The core commits its revision before an outbound failure; the bridge must not roll back inner writes or substitute the entry graph.
6. Return the current acknowledged revision separately from result decoding. Swift accepts that revision before rethrowing an original callback error. Keep existing binary outcome graph/result shapes. Release the job exactly once on success, operation error, transport failure or abandoned preparation.

A job/completion keeps the reservation until finish or drop. Successful preparation does not itself publish a new revision. The existing checked reservation arithmetic and creator filtering are reused, not duplicated. Native panic cleanup drops an owned unfinished job; it must not fabricate a completion or erase accepted inner writes.

## Additive C surface

The additive ABI suffix is `v15`. `PeleDetachedRenderJobV15` owns the affine core job/completion. Finish returns the existing owned `PeleResult` and writes current revision through a separate `uint64_t *out_revision`, including on outbound export failure.

- `pele_public_prepare_detached_render_v15`: same owner/revision/options/numeric/output-limit operands as v9; an out-job receives either one owned job or null for unsupported capability. Return selection/errors in the usual result envelope.
- `pele_public_prepare_detached_raw_v15`: same config/options/numeric operands as prepared v13, preserving raw config and options precedence.
- `pele_detached_render_run_v15`: job plus existing `PeleHostResourcesV3`; consumes the inner runnable job and stores its completion. An already-run job returns an ordinary semantic error.
- `pele_public_finish_detached_render_v15`: owner plus borrowed job; takes its stored completion, finishes, and exports. Owner mismatch, missing completion and LIFO errors use the existing core diagnostics.
- `pele_detached_render_job_free_v15` releases the job; existing `pele_result_free` releases each result. Result ownership remains explicit and one-use.

Existing symbols, structs and v4/v5/v9/v12/v13 behavior stay source/binary compatible. Existing C safety contract forbids freeing a document during its active call/callback; the new job contract extends that lifetime through finish/drop. Swift enforces this dynamically. A checked C close helper may be added if necessary for a public busy-close reproduction; silently changing the void v4 free contract is not part of this slice.

The header pins the function signatures and ownership rules. Finish serializes only after core completion and calls no host code.

## Native live values and graph identity

`LiveRenderRead` requires values, not a borrowed `ModelGraph`. Primitive values are copied. A read string owns its exact `JsString` units immediately; retaining a string-table ID would incorrectly retarget a captured value after nested apply. Enforce the existing 16-million-unit bound before copying. Number copies preserve IEEE bits. Root/field/index reads use `GraphReadView` directly with no per-read whole-graph snapshot or UTF-8 replacement.

A node reference is an explicit native graph ID. Snapshot-edit-apply with preserved historical slots can expose current edits to later phase reads; saved string values remain captured. Nested helper operations naturally preserve these IDs. Array replacement expressed by a new node ID leaves the old historical node available to a captured reference.

**Unsolved parity boundary:** the native full-graph API also accepts entirely replaced, truncated or renumbered identity tables. Unlike the browser facade, it does not supply persistent JavaScript object identities for removed/reused slots. A raw ID reader must not silently claim original-object lifetime parity for such input. Do not add a blanket apply rejection or change existing apply acceptance to conceal this. Before selecting a reader for unrestricted nested replacement, either implement an explicit retained identity model or expose a bounded host-read seam whose owner actually preserves object identity. The first feature's evidence must distinguish ID-preserving snapshot edits from arbitrary table replacement, and record the latter as unproven/unsolved. No whole-graph copy presented as a live reader.

String coercion and strict string errors must use existing native conversion/error helpers and retain current typed C/Swift fallback messages. No JavaScript engine or family algorithm moves to Swift. Unsupported live kinds and active lone-surrogate rendering remain explicit existing boundaries.

## Swift operation state and errors

Replace the single meaning of `active` with two separate private states: a short native-call busy guard and a detached-render depth. Prepare/finish/read/invoke/apply still acquire the busy guard. During owned job execution the short guard is released, permitting the callback's nested public operations; the detached depth stays positive and `close()` continues to fail. `withExtendedLifetime(self)` encloses the entire lifecycle.

Every render owns its resource bridge and original-error slot. `withHostFunctionResources` currently overwrites per-document error/resource closures; nested operations must save/restore those bindings as a stack and must not clear an outer pending exception. A caught inner resource error must not leak into the outer operation. Newly bound functions remain owned, and partial helper writes survive whether the helper error is caught or rethrown.

On return, finish is attempted after every dispatched job, even when the resource bridge recorded a Swift error. Decode/publish the native acknowledgement/revision before rethrowing the original error with its identity. Preserve the existing precedence of original resource errors over adapter decoding errors. No error cleanup may overwrite newer nested state with an older graph/revision.

Normal mutable and prepared/raw paths share this orchestration. Immutable `Document`, actor ownership, AppKit presentation resources and other families remain on their current routes. Native presentation entrypoints require a separate explicit integration decision if they cannot reuse the same completion/presentation result without changing ordering; this feature must not silently advertise those routes as reentrant.

## Exact file scope

Implementation owner (this agent):

- new `crates/pele-ffi/src/detached_render.rs`;
- `crates/pele-ffi/src/lib.rs` for module/export wiring;
- `crates/pele-ffi/src/prepared.rs` exposes its existing config decoder to the sibling adapter;
- `crates/pele/src/compat/public/{detached_render,mod}.rs` expose a tiny hidden wrapper around the existing JavaScript number formatter;
- `crates/pele-ffi/src/host_functions.rs` only for callback-scoped binding access if required;
- `include/pele.h` additive declarations and safety contract;
- new `bindings/swift/Sources/Pele/DetachedRender.swift`;
- `bindings/swift/Sources/Pele/MutableDocument.swift` for operation guards, dispatch and acknowledgement;
- `bindings/swift/Sources/Pele/PreparedDocument.swift` for shared raw dispatch;
- `bindings/swift/Sources/Pele/HostFunction.swift` for nested error/resource scope restoration.

No Rust family geometry/parser changes, WASM/web edits, generated SDK artifacts or tests are included in this authoring scope. A required additional core API or presentation route must be reported before expanding scope. Parent owns public integration tests and consolidated builds.

## Validation scope and remaining proof

Actual pinned Pele supplies the checked SVG, dimension, measurement-trace and recovery fixtures. The old Swift implementation failed all three initial callback tests; the fresh SDK passed those and two added recovery features. Each of the five new features exercises normal and prepared/raw rendering (ten lane cases). The first Swift run passed 22 tests; the second passed nine additional distinct tests, for 31 total. The new strict C lifecycle client and five existing strict C clients passed. The correctness-only Swift consumer passed 168 comparisons across 21 cases.

The C lifecycle test covers callback snapshot/helper/nested rendering, current revision acknowledgement, repeated job-use errors, cancellation and unsupported-family fallback. Native source/SDK build records match across 515 inputs. Later test-only extensions and fixtures are recorded separately from that build snapshot.

The following broader matrix remains a planning checklist, not a claim that every combination was exercised:

- Same-document `findOrCreateNode` from width/icon callbacks, existing/new IDs and exact aliases; normal and prepared/raw lanes.
- Nested render from width/icon, nested helper then render, other-document control, and post-operation recovery.
- Read-only array length causing a helper's partial mutation, caught/uncaught errors, current revision and repair.
- Original Swift reference-type errors, nested error caught by the outer callback, and an outer original error after successful inner mutation.
- ID-preserving apply, root/array replacement using appended historical nodes, stale rejection, and separate explicit arbitrary reindex/truncation boundary cases.
- Close/disposal refusal during callbacks, close after unwind, abandoned/double-run/double-finish jobs, foreign owner and LIFO violations.
- Unchanged source/model outputs and numeric/output-limit behavior, custom provider fallback, retained registration/config, limits and export failure after committed state.

No performance timing is part of acceptance. The scoped `cargo check -p pele-ffi --locked`, Swift syntax parsing and parent-owned runtime checks passed. Icon-specific reentry, arbitrary graph-table replacement/identity lifetime, native presentation/image rendering and the complete broader matrix above remain outside this proof. See [the focused evidence](parity/2026-10-21-native-sankey-callbacks/README.md).
