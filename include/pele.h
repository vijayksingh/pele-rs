#ifndef PELE_H
#define PELE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
#define PELE_ABI_VERSION 2
typedef struct PeleResult PeleResult;
uint32_t pele_abi_version(void);
/* Compatibility source preparation; result envelope version 2. Does not render.
 * Transport budgets: 16 MiB source / 1 MiB options, checked before borrowing.
 * Default source limit is 50,000 UTF-16 units, configurable in options JSON. */
PeleResult *pele_inspect(const uint8_t *source, size_t source_len,
                         const uint8_t *options, size_t options_len);
/* Owned version-2 capability report. Known types and installed renderers are separate. */
PeleResult *pele_capabilities(void);
/* Version-2 built-in parse/render. parse result retains a document until result_free.
 * render_document borrows it synchronously; source/options are copied into owned models.
 * A render/error result cannot serve as a document. Transport budgets match inspect. */
PeleResult *pele_parse_v2(const uint8_t *source, size_t source_len, const uint8_t *options, size_t options_len);
/* Additive eager immutable projection transport; existing v2 JSON ABI unchanged.
 * Success envelope result is {type, projectionBytes}; projectionBytes is the exact
 * original v2 envelope size. Above 64 MiB, no events are delivered; the caller must
 * reject the result using its existing compatibility-result admission error.
 * A successful handle retains only the typed document and this small envelope.
 * Callback receives 1..128 events; all pointers are borrowed only for that call.
 * It must copy strings eagerly, must not unwind, and returns 0 to continue or
 * nonzero to abort. Aborted calls return an error handle with no document.
 * Scalar tags: 0 null, 1 bool (count=0/1), 2 number, 3 UTF-8 string.
 * Container tags: 4 array begin (count=length), 5 array end,
 * 6 object begin (count=entry count), 7 key, 8 object end.
 * String/key count is UTF-8 byte length. Unused fields are zero/NULL.
 * Numbers match JSON projection (nonfinite=null, negative zero=positive zero).
 * Object keys follow JS enumeration order; consumers choose host key equality.
 * Callbacks/context are synchronous, never retained. No host resources invoked. */
typedef struct PeleProjectionEventV1 {
    uint32_t tag;
    size_t count;
    double number;
    const uint8_t *text;
} PeleProjectionEventV1;
typedef uint32_t (*PeleProjectionBatchV1)(void *, const PeleProjectionEventV1 *, size_t);
PeleResult *pele_parse_eager_v1(const uint8_t *source, size_t source_len,
    const uint8_t *options, size_t options_len, void *context, PeleProjectionBatchV1 receive);
PeleResult *pele_render_v2(const uint8_t *source, size_t source_len, const uint8_t *options, size_t options_len);
PeleResult *pele_render_document_v2(const PeleResult *document, const uint8_t *options, size_t options_len);
PeleResult *pele_builtin_capabilities(void);
/* Read-only retained model operation, UTF-8 name <=4096 bytes and JSON args <=1 MiB. */
PeleResult *pele_document_query_v2(const PeleResult *, const uint8_t *, size_t, const uint8_t *, size_t);
/* Per-render resources. Callback status: 0 success, nonzero failure.
 * Icons are trusted host SVG, never source strings. Returned bytes are borrowed
 * until the next callback; Rust copies them immediately. All callbacks/context
 * are borrowed synchronously and never retained. NULL callback selects fallback. */
typedef struct PeleBorrowedBytes { const uint8_t *data; size_t len; } PeleBorrowedBytes;
typedef struct PeleHostResources {
    void *context;
    uint32_t (*width)(void *, const uint8_t *, size_t, double, uint32_t,
                      const uint8_t *, size_t, const uint8_t *, size_t, double *);
    uint32_t (*icon)(void *, const uint8_t *, size_t, PeleBorrowedBytes *);
    uint32_t (*now)(void *, double *);
} PeleHostResources;
PeleResult *pele_render_with_resources_v2(const uint8_t *, size_t, const uint8_t *, size_t, const PeleHostResources *);
PeleResult *pele_render_document_with_resources_v2(const PeleResult *, const uint8_t *, size_t, const PeleHostResources *);
/* Calendar extension preserves the v2 resource struct ABI. Operation 0 takes a
 * JSON millisecond instant, returning CalendarFields or null. Operation 1 takes
 * {civil:{year,month,day,hour,minute,second,millisecond},utcAnchorMillis:number|null},
 * returning the local millisecond instant or null. Month=1..12, weekday=0 Sunday,
 * utcOffsetMinutes is east-positive. Civil fields are normalized; resolve folds
 * to the earliest instant and gaps forward by their duration, as JavaScript Date.
 * The optional date_parser callback uses operation 2, takes a date string and returns its parsed instant or null.
 * Optional utc_offset receives an instant in milliseconds and writes integer offset
 * seconds east of UTC; returns zero on success. Rust resolves proleptic civil dates and DST gaps/folds.
 * Explicit calendar/calendar_instant callbacks take precedence over utc_offset.
 * Calendar output is borrowed until next callback; NULL/0 also means null. */
typedef struct PeleHostResourcesV3 {
    PeleHostResources base;
    uint32_t (*calendar)(void *, uint32_t, const uint8_t *, size_t, PeleBorrowedBytes *);
    uint32_t (*calendar_instant)(void *, uint32_t, const uint8_t *, size_t, PeleBorrowedBytes *);
    uint32_t (*date_parser)(void *, uint32_t, const uint8_t *, size_t, PeleBorrowedBytes *);
    uint32_t (*utc_offset)(void *, double, int32_t *);
} PeleHostResourcesV3;
PeleResult *pele_render_with_resources_v3(const uint8_t *, size_t, const uint8_t *, size_t, const PeleHostResourcesV3 *);
PeleResult *pele_render_document_with_resources_v3(const PeleResult *, const uint8_t *, size_t, const PeleHostResourcesV3 *);
/* NULL-safe accessors: status 3, data NULL, length 0. Status: 0 success,
 * 1 input error, 2 resource limit, 3 internal error. Bytes contain versioned JSON,
 * not a NUL-terminated string; data remains valid until result_free. */
/* Mutable public documents: synchronize destruction with all operations.
 * Graph JSON preserves UTF-16 code units, IEEE number bits and object identity.
 * Operation results use existing owned version-2 envelopes. Revisions are decimal
 * strings for snapshot/apply, uint64 JSON integers for invocation/render outcomes.
 * Expected revisions accept exact uint64 values. Graph input <=64 MiB.
 * NULL resources select default native resources. Parse sets *out_document=NULL
 * before work; a successful handle must be freed once. Recursive/concurrent
 * operations on one handle return an error, except synchronous callbacks in the
 * additive v15 detached lifecycle below. Never free during an active call. */
typedef struct PelePublicDocument PelePublicDocument;
/* Owned selective registry. mode0 empty, mode1 all33 built-ins; all successful
 * results use owned version2 envelopes. create writes NULL before any failure
 * and publishes ownership only after result encoding. Free once after all calls
 * return; free(NULL) is safe. Concurrent/reentrant operations return busy rather
 * than blocking. Registries and documents have independent lifetimes: returned
 * documents retain their module/config/state after registration replacement/free.
 * Exact built-in IDs only for registration; duplicate registration replaces its
 * handler without adding a capability row. IDs are UTF-8 <=4096 bytes.
 * Discovery preprocesses source <=16MiB without parse thresholds or grammar
 * validation. detect returns {type:string|null}; preprocessing errors propagate.
 * supports returns bool, false on preprocessing failure or missing registration;
 * invalid UTF-8/native transport failures remain errors. capabilities includes
 * every known built-in even when unregistered. NULL registry operations fail.
 * Synchronize free with all operations; no registry pointer is retained by docs. */
typedef struct PeleRegistry PeleRegistry;
/* Captured builtin constructor (v16). Capture returns {selected:bool}; false
 * writes NULL for unported/foreign prototypes. A selected factory retains its
 * registration after registry replacement/free. Construct has no source input
 * or callbacks and creates low-level mutable state, not core parse provenance.
 * Construction returns ordinary document metadata plus constructorCreated.
 * Free each factory/document once; never free during an active call.
 * id is readable UTF-8 <=4096 bytes; outputs writable and nulled before failure.
 * prototype is the exact ID exposed by the public model schema. */
typedef struct PelePublicConstructorV16 PelePublicConstructorV16;
PeleResult *pele_registry_capture_constructor_v16(const PeleRegistry *, const uint8_t *, size_t,
    uint32_t prototype, PelePublicConstructorV16 **out_constructor);
PeleResult *pele_public_constructor_construct_v16(const PelePublicConstructorV16 *, PelePublicDocument **);
void pele_public_constructor_free_v16(PelePublicConstructorV16 *);
/* Lossless host-value/owned invocation lane (v17). Actor/thread confined.
 * tag 0 undefined,1 null,2 bool(payload0/1),3 IEEE bits,4 borrowed UTF16,
 * 5 owner-local opaque identity. UTF16 is copied, <=16M units. NULL=undefined.
 * reply_state: 0 start, 1 value, 2 original exception token (tag5),
 * 3 JavaScript TypeError message (tag4).
 * No host callbacks run inside these calls. Keep document alive through job free.
 * Resume returns PELGBIN1 with Host step or Complete metadata; finish returns
 * PELGBIN1 Result<Atom,PublicError>. Invoke host operations between calls only.
 * Every dispatched phase advances current revision before mutation. The output
 * revision acknowledges it even if subsequent export fails. Drop cancels future
 * phases without rolling back accepted partial state. Native refs are graph IDs.
 */
typedef struct PeleHostValueV17 {
    uint32_t version, struct_size, tag, reserved;
    uint64_t payload;
    const uint16_t *units;
    size_t unit_count;
} PeleHostValueV17;
typedef struct PeleInvocationJobV17 PeleInvocationJobV17;
PeleResult *pele_public_constructor_construct_with_v17(const PelePublicConstructorV16 *, const PeleHostValueV17 *, PelePublicDocument **);
PeleResult *pele_public_invocation_prepare_v17(PelePublicDocument *, uint64_t, const uint8_t *, size_t, PeleInvocationJobV17 **);
PeleResult *pele_public_invocation_resume_v17(PelePublicDocument *, PeleInvocationJobV17 *, uint32_t reply_state, const PeleHostValueV17 *, uint64_t *revision);
PeleResult *pele_public_invocation_finish_v17(PelePublicDocument *, PeleInvocationJobV17 *, uint64_t *revision);
void pele_public_invocation_free_v17(PeleInvocationJobV17 *);
PeleResult *pele_host_number_string_v17(uint64_t ieee_bits);
/* Shared identity namespace with v12 bound functions; IDs never repeat. */
PeleResult *pele_host_identity_v17(void);
PeleResult *pele_host_primitive_number_v17(const PeleHostValueV17 *value);
PeleResult *pele_public_constructor_policy_v17(const PelePublicConstructorV16 *constructor);
/* Unpublished constructor bodies (v18). Same value/reply/step formats as v17.
 * Getters/calls run only between calls. Dispose cannot overlap an active call.
 * policy returns argumentCount, arity and argumentTransfers (retain/value).
 * Retain arguments use caller-owned opaque identities without inspecting their
 * values; Value arguments copy known scalars/UTF16, leaving foreign objects
 * opaque. No argument beyond the declared transfer list is inspected.
 * finish transfers a partial owner even when it returns an error; always free
 * a non-null out_document. No callback may use a freed provisional job. */
typedef struct PeleConstructionJobV18 PeleConstructionJobV18;
PeleResult *pele_public_constructor_policy_v18(const PelePublicConstructorV16 *);
PeleResult *pele_public_constructor_begin_v18(const PelePublicConstructorV16 *, const PeleHostValueV17 *, const PeleHostValueV17 *, PeleConstructionJobV18 **);
PeleResult *pele_public_construction_resume_v18(PeleConstructionJobV18 *, uint32_t, const PeleHostValueV17 *, uint64_t *);
PeleResult *pele_public_construction_snapshot_v18(PeleConstructionJobV18 *);
PeleResult *pele_public_construction_apply_v18(PeleConstructionJobV18 *, const uint8_t *, size_t);
PeleResult *pele_public_construction_invoke_v18(PeleConstructionJobV18 *, uint64_t, const uint8_t *, size_t, uint64_t *);
PeleResult *pele_public_construction_get_v18(PeleConstructionJobV18 *, uint32_t, const uint8_t *, size_t);
PeleResult *pele_public_construction_finish_v18(PeleConstructionJobV18 *, PelePublicDocument **, uint64_t *);
void pele_public_construction_free_v18(PeleConstructionJobV18 *);
PeleResult *pele_registry_create_v8(uint32_t mode, PeleRegistry **out_registry);
void pele_registry_free_v8(PeleRegistry *registry);
PeleResult *pele_registry_capabilities_v8(const PeleRegistry *registry);
PeleResult *pele_registry_registered_v8(const PeleRegistry *registry, const uint8_t *id, size_t id_len);
PeleResult *pele_registry_register_builtin_v8(PeleRegistry *registry, const uint8_t *id, size_t id_len);
PeleResult *pele_registry_detect_type_v8(const PeleRegistry *registry, const uint8_t *source, size_t source_len);
PeleResult *pele_registry_supports_v8(const PeleRegistry *registry, const uint8_t *source, size_t source_len);
/* Same independent numeric policy tags as parse_limits_v7:0 undefined,1 null,
 * 2 raw IEEE-754 bits,3 legacy integer option. Source16MiB/options1MiB budgets
 * always apply. Only explicitly overridden fields bypass old integer decoding.
 * *out_document is NULL on every error. Missing-registration errors retain core
 * type/message. Live documents use the existing public document operations. */
PeleResult *pele_registry_parse_limits_v8(const PeleRegistry *registry,
    const uint8_t *source, size_t source_len, const uint8_t *options, size_t options_len,
    uint32_t source_kind, uint64_t source_bits, uint32_t edge_kind, uint64_t edge_bits,
    PelePublicDocument **out_document);
PeleResult *pele_public_parse_v4(const uint8_t *, size_t, const uint8_t *, size_t, PelePublicDocument **out_document);
/* Additive exact source-limit lane for well-formed UTF-8 source. limit_kind:
 * 0 undefined (default 50,000 UTF-16 units), 1 null, 2 IEEE-754 number bits.
 * Overrides only options.limit; config/maxEdges retain their existing contract.
 * NaN/infinity never disable the independent 16 MiB source-byte budget.
 * Ownership/error rules match parse_v4; unknown kinds fail without a handle. */
PeleResult *pele_public_parse_source_limit_v6(const uint8_t *, size_t,
                                           const uint8_t *, size_t,
                                           uint32_t limit_kind, uint64_t limit_bits,
                                           PelePublicDocument **out_document);
/* Independent parse policies. Each kind: 0 undefined, 1 null, 2 IEEE-754 bits,
 * 3 legacy options field (bits ignored). Explicit source overrides only limit;
 * explicit edge overrides only maxEdges. Non-overridden fields retain their
 * existing integer decoder, including errors. Undefined uses defaults50,000
 * UTF-16 source units/5,000 edges; null compares as0. Actual edge accounting is
 * diagram-specific. Native source16MiB/options1MiB budgets always remain active.
 * Source checks precede diagram parsing/edge checks. Parse_v4 ownership applies;
 * every failure leaves *out_document=NULL. Unknown tags reject without a handle. */
PeleResult *pele_public_parse_limits_v7(const uint8_t *source, size_t source_len,
    const uint8_t *options, size_t options_len, uint32_t source_kind,
    uint64_t source_bits, uint32_t edge_kind, uint64_t edge_bits,
    PelePublicDocument **out_document);
PeleResult *pele_public_snapshot_v4(const PelePublicDocument *);
/* Native schema and method operations. invoke returns an operation outcome with
 * graph/revision/result even when the method throws, subject to output budgets.
 * Outer errors include preflight rejection and export/transport failure after
 * dispatch; state may advance in the latter case. call JSON is bounded to 64 MiB. */
PeleResult *pele_public_schema_v4(const PelePublicDocument *document);
PeleResult *pele_public_invoke_v4(PelePublicDocument *document, uint64_t expected_revision,
                                const uint8_t *call, size_t call_length,
                                const PeleHostResourcesV3 *resources);
PeleResult *pele_public_apply_v4(PelePublicDocument *, uint64_t expected_revision, const uint8_t *, size_t);
PeleResult *pele_public_render_v4(PelePublicDocument *, const uint8_t *, size_t, const PeleHostResourcesV3 *);
/* Publishes post-render graph/revision on success or operation error. Outer
 * export/transport failure may follow mutation, as for invoke_v4. */
PeleResult *pele_public_render_operation_v4(PelePublicDocument *, uint64_t expected_revision,
                                          const uint8_t *, size_t, const PeleHostResourcesV3 *);
/* Binary graph schema1: PELGBIN1, LE version/revision/metadata length,
 * root/string/node tables, trailing JSON metadata. Each result is owned/freed
 * exactly as v4. A nonzero status carries the existing JSON error envelope. */
PeleResult *pele_public_snapshot_binary_v5(const PelePublicDocument *document);
PeleResult *pele_public_apply_binary_v5(PelePublicDocument *document, const uint8_t *graph, size_t graph_len);
PeleResult *pele_public_invoke_binary_v5(PelePublicDocument *document, uint64_t expected_revision, const uint8_t *call, size_t call_len, const PeleHostResourcesV3 *resources);
PeleResult *pele_public_render_operation_binary_v5(PelePublicDocument *document, uint64_t expected_revision, const uint8_t *options, size_t options_len, const PeleHostResourcesV3 *resources);
/* Exact output-limit lane, binary graph outcome as v5. limit_kind: 0 undefined,
 * 1 null (both default 4,000,000 UTF-16 units), 2 raw IEEE-754 number bits.
 * Overrides only options.outputLimit; other options retain existing decoding.
 * Rendering and host callbacks run before output comparison; operation errors
 * retain post-call graph/revision. Stale revisions reject before dispatch.
 * Independent returned SVG ceiling:64 MiB UTF-8, including with NaN/infinity.
 * This bounds accepted output, not all peak allocations inside renderers.
 * Options <=1 MiB, complete binary exchange <=64 MiB; outer export errors may
 * follow state mutation. Unknown kinds reject before dispatch. Ownership and
 * synchronous host callback rules match render_operation_binary_v5. */
PeleResult *pele_public_render_operation_output_limit_v6(PelePublicDocument *document,
    uint64_t expected_revision, const uint8_t *options, size_t options_len,
    uint32_t limit_kind, uint64_t limit_bits, const PeleHostResourcesV3 *resources);
/* Native number overrides: exact56-byte host-ABI packet; never a wire format.
 * NULL packet selects no overrides. Readable8-byte header required; full56bytes
 * required when struct_size is56. version=9, exactsize56, present bits0..4 only.
 * Payload slots contain binary64 bits; unselected slots ignored. Synchronous
 * borrow only. NaN payload identity is transport-only, not after arithmetic. */
#define PELE_RENDER_NUMBER_FONT_SIZE_V9 (UINT64_C(1) << 0)
#define PELE_RENDER_NUMBER_MAX_WIDTH_V9 (UINT64_C(1) << 1)
#define PELE_RENDER_NUMBER_DIRECTION_BREAKPOINT_V9 (UINT64_C(1) << 2)
#define PELE_RENDER_NUMBER_PADDING_V9 (UINT64_C(1) << 3)
#define PELE_RENDER_NUMBER_NOW_V9 (UINT64_C(1) << 4)
typedef struct PeleRenderNumbersV9 {
    uint32_t version, struct_size;
    uint64_t present;
    uint64_t font_size_bits, max_width_bits, direction_breakpoint_bits, padding_bits, now_bits;
} PeleRenderNumbersV9;
/* v9 checks options<=1MiB then source<=16MiB before buffer borrows, then packet
 * and output tag, then options UTF-8/JSON, then source/document provenance or
 * mutable lock/revision. Packet/preflight errors never dispatch callbacks or
 * advance revision. Selected top-level JSON fields alone bypass old decoding;
 * duplicate selected fields/trailing input reject. Output tags0/1/2 select exact
 * undefined/null/number;3 retains legacy outputLimit/dispatch. Unknown tags reject.
 * Returned SVG independently <=64MiB before/after native theme, all result
 * envelopes/exchanges <=64MiB. Bounds accepted output, not renderer peak memory.
 * Immutable results retain existing width/height IEEE sidecars. Mutable results
 * publish dispatched graph/revision on operation errors as v5/v6. All inputs,
 * packet, documents and resources borrow synchronously; existing result freeing
 * and callback-error ownership apply. Old C exports retain their dispatch. */
PeleResult *pele_render_with_numbers_v9(const uint8_t *source, size_t source_len,
    const uint8_t *options, size_t options_len, const PeleRenderNumbersV9 *numbers,
    uint32_t output_limit_kind, uint64_t output_limit_bits, const PeleHostResourcesV3 *resources);
PeleResult *pele_render_document_with_numbers_v9(const PeleResult *document,
    const uint8_t *options, size_t options_len, const PeleRenderNumbersV9 *numbers,
    uint32_t output_limit_kind, uint64_t output_limit_bits, const PeleHostResourcesV3 *resources);
PeleResult *pele_public_render_operation_with_numbers_v9(PelePublicDocument *document,
    uint64_t expected_revision, const uint8_t *options, size_t options_len,
    const PeleRenderNumbersV9 *numbers, uint32_t output_limit_kind,
    uint64_t output_limit_bits, const PeleHostResourcesV3 *resources);
void pele_public_document_free_v4(PelePublicDocument *);

/* Additive detached builtin rendering (v15). Preparation returns {selected:bool}
 * and an owned job, or NULL job for unsupported providers. Existing entrypoints
 * stay unchanged. The document MUST outlive every job, including abandoned jobs.
 * Never free the document/job during any active run/callback. Run releases the
 * document lock, permitting synchronous same-owner snapshot/apply/invoke/nested
 * detached rendering. Native Ref IDs remain graph coordinates: arbitrary table
 * replacement/truncation is not JavaScript retained-object lifetime parity.
 * Finish consumes completion in nesting order and returns the existing bounded
 * binary render outcome. out_revision is updated to current committed revision
 * before returning an export error; callers must acknowledge it before decoding
 * or rethrowing their original resource exception. Job free is required once
 * after finish, or to abandon a prepared/completed job. Null free is safe.
 * All options/resource input spans have the same bounds as v9/v13. */
typedef struct PeleDetachedRenderJobV15 PeleDetachedRenderJobV15;
PeleResult *pele_public_prepare_detached_render_v15(PelePublicDocument *, uint64_t,
    const uint8_t *, size_t, const PeleRenderNumbersV9 *, uint32_t, uint64_t,
    PeleDetachedRenderJobV15 **);
PeleResult *pele_public_prepare_detached_raw_v15(PelePublicDocument *, uint64_t,
    const uint8_t *, size_t, const uint8_t *, size_t, const PeleRenderNumbersV9 *,
    PeleDetachedRenderJobV15 **);
PeleResult *pele_detached_render_run_v15(PeleDetachedRenderJobV15 *, const PeleHostResourcesV3 *);
PeleResult *pele_public_finish_detached_render_v15(PelePublicDocument *, PeleDetachedRenderJobV15 *, uint64_t *);
void pele_detached_render_job_free_v15(PeleDetachedRenderJobV15 *);


/* Additive owner-confined provider prototype (v10). Only canonical built-in IDs
 * can be replaced. Registration retains provider context once. Existing documents
 * retain their provider/model across replacement and Registry destruction.
 * No callback may unwind; callers serialize Registry/document calls and destruction.
 * Callback return buffers are independently owned leases released exactly once.
 * Reply status: 0 success, 1 payload is PeleError JSON, 2 operation-scoped exception.
 * status2 requires a nonnull invocation_context and a nonzero exception token.
 * Call pointers/resources are valid only inside that synchronous callback.
 * Finite Config/value-copy and well-formed UTF-8 prototype; no graph codec or VM.
 * Provider calls may allocate arbitrary memory; accepted outputs remain <=64 MiB.
 */
typedef struct PeleProviderDocument PeleProviderDocument;
typedef struct PeleForeignCallV10 PeleForeignCallV10;
typedef struct PeleForeignBytesV10 {
    void *owner; const uint8_t *data; size_t len; void (*release)(void *);
} PeleForeignBytesV10;
typedef struct PeleForeignReplyV10 {
    uint32_t status, exception; void *model;
    PeleForeignBytesV10 payload, links;
    uint64_t width_bits, height_bits;
} PeleForeignReplyV10;
typedef struct PeleProviderParseV10 {
    const uint8_t *source; size_t source_len;
    const uint8_t *config; size_t config_len;
    const uint8_t *title; size_t title_len;
    uint32_t has_title; uint64_t line_offset;
    uint32_t edge_kind; uint64_t edge_bits;
} PeleProviderParseV10;
typedef struct PeleProviderRenderV10 {
    const uint8_t *config; size_t config_len;
    const uint8_t *options; size_t options_len;
    PeleRenderNumbersV9 numbers;
    uint32_t output_kind; uint64_t output_bits;
} PeleProviderRenderV10;
typedef struct PeleDiagramProviderV10 {
    uint32_t version, struct_size; void *context;
    void (*retain)(void *); void (*release)(void *); void (*release_model)(void *, void *);
    void (*describe)(void *, PeleForeignCallV10 *, uint32_t, PeleForeignReplyV10 *);
    void (*parse)(void *, PeleForeignCallV10 *, const PeleProviderParseV10 *, PeleForeignReplyV10 *);
    void (*render)(void *, void *, PeleForeignCallV10 *, const PeleProviderRenderV10 *, PeleForeignReplyV10 *);
    uint64_t reserved[2];
} PeleDiagramProviderV10;
PeleResult *pele_registry_register_provider_v10(PeleRegistry *, const uint8_t *, size_t, const PeleDiagramProviderV10 *);
/* New provider-only document lane; built-ins keep the existing v8 parse API.
 * Inputs: source<=16 MiB, options<=1 MiB; exact threshold tags match v7.
 * out_document initializes null on all failures. Successful result.result.Ok
 * carries type/revision/modelKind; callback failures use result.result.Err.
 */
PeleResult *pele_registry_parse_provider_v10(const PeleRegistry *, const uint8_t *, size_t,
    const uint8_t *, size_t, uint32_t, uint64_t, uint32_t, uint64_t, void *, PeleProviderDocument **);
PeleResult *pele_provider_model_v10(const PeleProviderDocument *, void **);
PeleResult *pele_provider_status_v10(const PeleProviderDocument *);
PeleResult *pele_provider_render_v10(PeleProviderDocument *, uint64_t,
    const uint8_t *, size_t, const PeleRenderNumbersV9 *, uint32_t, uint64_t,
    const PeleHostResourcesV3 *, void *);
void pele_provider_document_free_v10(PeleProviderDocument *);
void *pele_foreign_call_context_v10(const PeleForeignCallV10 *);
PeleResult *pele_foreign_text_width_v10(PeleForeignCallV10 *, const uint8_t *, size_t,
    uint64_t, uint32_t, uint64_t *);
/* Borrowed resources: active synchronous render only, one call at a time across
 * all helpers. Reentry returns a semantic error; never retain the call pointer.
 * Icon names are entity-decoded with no cache. Result owns {value: string|null};
 * each accepted icon is <=8 MiB. Names/date strings are UTF-8, <=1 MiB.
 * Clock uses the render session's explicit finite now or cached host clock.
 * Calendar/date methods reuse existing TimeClip and field validation. Optional
 * numeric outputs initialize (present=0,bits=0) before failure; success true is
 * published before outputs. Calendar fields result owns {value: object|null}.
 * Caller frees every PeleResult, including failures.
 */
PeleResult *pele_foreign_icon_v10(PeleForeignCallV10 *, const uint8_t *, size_t);
PeleResult *pele_foreign_now_millis_v10(PeleForeignCallV10 *, uint64_t *);
PeleResult *pele_foreign_parse_date_v10(PeleForeignCallV10 *, const uint8_t *, size_t, uint32_t *, uint64_t *);
PeleResult *pele_foreign_calendar_fields_v10(PeleForeignCallV10 *, uint64_t);
PeleResult *pele_foreign_calendar_instant_v10(PeleForeignCallV10 *, const uint8_t *, size_t, uint32_t *, uint64_t *);

/* Additive native presentation lane. NULL packet selects defaults; otherwise
 * version=11, struct_size=sizeof(PelePresentationOptionsV11), reserved all zero.
 * Requested budgets are clamped to native hard ceilings. Successful render
 * result adds presentation: available artifact or explicit unavailable reason.
 * Existing v9 numeric/output policy and v3 host-resource contracts apply.
 */
typedef struct PelePresentationOptionsV11 {
    uint32_t version, struct_size;
    uint64_t max_elements, max_text_bytes;
    uint64_t reserved[2];
} PelePresentationOptionsV11;
PeleResult *pele_render_presentation_v11(const uint8_t *, size_t, const uint8_t *, size_t,
    const PeleRenderNumbersV9 *, uint32_t, uint64_t,
    const PelePresentationOptionsV11 *, const PeleHostResourcesV3 *);
PeleResult *pele_render_document_presentation_v11(const PeleResult *, const uint8_t *, size_t,
    const PeleRenderNumbersV9 *, uint32_t, uint64_t,
    const PelePresentationOptionsV11 *, const PeleHostResourcesV3 *);
PeleResult *pele_public_render_presentation_v11(PelePublicDocument *, uint64_t,
    const uint8_t *, size_t, const PeleRenderNumbersV9 *, uint32_t, uint64_t,
    const PelePresentationOptionsV11 *, const PeleHostResourcesV3 *);

/* Owner-thread host methods for mutable built-in models. Binding retains context
 * once until document close (maximum 1024). IDs are never reused and only resolve
 * on their owner document. Callback/table must not unwind across C. */
typedef struct PeleActiveModelV12 PeleActiveModelV12;
typedef struct PeleHostFunctionReplyV12 {
    uint32_t status, exception; /* 0 Atom JSON, 1 PublicError JSON, 2 exception token */
    PeleForeignBytesV10 payload; /* copied then released exactly once */
} PeleHostFunctionReplyV12;
typedef struct PeleHostFunctionV12 {
    uint32_t version, struct_size; /* 12, sizeof(PeleHostFunctionV12) */
    void *context;
    void (*retain)(void *);
    void (*release)(void *);
    void (*invoke)(void *, PeleActiveModelV12 *, const uint8_t *, size_t, PeleHostFunctionReplyV12 *);
    uint64_t reserved[2];
} PeleHostFunctionV12;
PeleResult *pele_public_bind_host_function_v12(PelePublicDocument *, const PeleHostFunctionV12 *, uint32_t *);
/* Captured builtin invocation with the document's v12 function bindings.
 * Returns the same binary graph/result envelope as invoke_binary_v5. */
PeleResult *pele_public_invoke_with_functions_v12(PelePublicDocument *document, uint64_t expected_revision, const uint8_t *call, size_t call_len, const PeleHostResourcesV3 *resources);
PeleResult *pele_public_render_operation_with_functions_v12(PelePublicDocument *, uint64_t,
    const uint8_t *, size_t, const PeleRenderNumbersV9 *, uint32_t, uint64_t, const PeleHostResourcesV3 *);
/* Active view only valid synchronously inside its callback. Never retain pointer.
 * Access JSON <=1MiB; one million operations per call plus existing arena budgets.
 * Operations: root, read/set {receiver,key[,value]}, intern {units:[u16]},
 * string/dateMillis {value}, object, array, date {value:NumberAtom},
 * push {receiver,value}, invokeBuiltin {call:{method,receiver,arguments}}.
 * Result envelope contains Result<{kind:atom|string,value},PublicError>.
 * Structural array mutation uses push; direct length assignment is unsupported.
 * Same-view reentry during invokeBuiltin fails; public document is busy throughout.
 */
PeleResult *pele_active_model_access_v12(PeleActiveModelV12 *, const uint8_t *, size_t);

/* Named per-diagram parse/render. Source is verbatim already-prepared UTF-8.
 * Packet NULL: offset0, no title, undefined edge cap (raw disables semantic cap).
 * Packet version13/size48, flags<=1, reserved0. Edges0undefined/1null/2bits.
 * Native source16MiB, config/title1MiB, returnedSVG64MiB bounds remain.
 * Successful parse returns an owned public document; existing snapshot/apply/
 * schema/invoke/free functions operate on it. Use explicit raw render below.
 */
typedef struct PelePreparedOptionsV13 {
    uint32_t version, struct_size;
    uint64_t line_offset;
    uint32_t max_edges_kind, title_present;
    uint64_t max_edges_bits, reserved[2];
} PelePreparedOptionsV13;
PeleResult *pele_parse_prepared_v13(const uint8_t *, size_t, const uint8_t *, size_t,
    const uint8_t *, size_t, const uint8_t *, size_t,
    const PelePreparedOptionsV13 *, PelePublicDocument **);
PeleResult *pele_render_prepared_v13(PelePublicDocument *, uint64_t,
    const uint8_t *, size_t, const uint8_t *, size_t,
    const PeleRenderNumbersV9 *, const PeleHostResourcesV3 *);

/* Construct directly from a caller-owned PELGBIN1 model, revision0/no metadata.
 * The graph is copied into the new document. Existing builtin method IDs must
 * belong to this type; bind host functions after construction, then apply them. */
PeleResult *pele_create_prepared_v13(const uint8_t *, size_t, const uint8_t *, size_t,
    const uint8_t *, size_t, PelePublicDocument **);
PeleResult *pele_prepared_schema_v13(const uint8_t *, size_t);

/* Native resource lane: image callbacks see only core-approved references.
 * No resolver means images fail before platform decoding. Returned borrowed PNG/
 * JPEG bytes stay readable until the next callback or render call completion.
 * Hard caps: 8 MiB/image, 16 MiB aggregate emissions/operation, 16,777,216 pixels, 8192/axis, 128 refs.
 * Requests may lower caps. Platform callers additionally validate image pixels.
 * NULL packet selects strict defaults without an image resolver. All callbacks
 * are synchronous; document reentry remains rejected. Existing APIs unchanged.
 */
typedef uint32_t (*PeleImageLoaderV14)(void *, const uint8_t *, size_t, PeleBorrowedBytes *);
typedef struct PeleNativeResourcesV14 {
    uint32_t version, struct_size; void *context; PeleImageLoaderV14 image_loader;
    uint64_t max_image_bytes, max_image_pixels, max_total_bytes; uint64_t reserved[2];
} PeleNativeResourcesV14;
PeleResult *pele_render_native_v14(const uint8_t *, size_t, const uint8_t *, size_t,
    const PeleRenderNumbersV9 *, uint32_t, uint64_t, const PelePresentationOptionsV11 *,
    const PeleHostResourcesV3 *, const PeleNativeResourcesV14 *);
PeleResult *pele_render_document_native_v14(const PeleResult *, const uint8_t *, size_t,
    const PeleRenderNumbersV9 *, uint32_t, uint64_t, const PelePresentationOptionsV11 *,
    const PeleHostResourcesV3 *, const PeleNativeResourcesV14 *);
PeleResult *pele_public_render_native_v14(PelePublicDocument *, uint64_t, const uint8_t *, size_t,
    const PeleRenderNumbersV9 *, uint32_t, uint64_t, const PelePresentationOptionsV11 *,
    const PeleHostResourcesV3 *, const PeleNativeResourcesV14 *);

uint32_t pele_result_status(const PeleResult *result);
const uint8_t *pele_result_data(const PeleResult *result);
size_t pele_result_len(const PeleResult *result);
void pele_result_free(PeleResult *result);
#ifdef __cplusplus
}
#endif
#endif
