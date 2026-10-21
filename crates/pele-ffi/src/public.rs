//! Additive mutable document ABI. Graph JSON preserves UTF-16 units and IEEE bits.
//! This bounded UTF-8 source/JSON lane is not the complete lossless source/options ABI.
use super::{calendar, compatibility_call, compatibility_input, PeleHostResourcesV3, PeleResult};
use pele::compat::{
    public::{GraphReadView, ModelGraph, PublicDocument, PublicEngine, ReadOutcome},
    ErrorCode, PeleError,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Mutex, MutexGuard, TryLockError},
};
const GRAPH_JSON_BUDGET: usize = 64 * 1024 * 1024;
/// Opaque owner. Operations serialize; callers must not free while an operation is active.
pub struct PelePublicDocument {
    document: Mutex<PublicDocument>,
    pub(super) host_functions: std::cell::RefCell<super::host_functions::Bindings>,
}
pub(super) fn owned_document(document: PublicDocument) -> *mut PelePublicDocument {
    Box::into_raw(Box::new(PelePublicDocument {
        document: Mutex::new(document),
        host_functions: Default::default(),
    }))
}
pub(super) fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn source_limit_options(json: &str) -> Result<pele::compat::ParseOptions, PeleError> {
    // The explicit source-limit argument is the sole authority in v6. Consume
    // the unused legacy field without applying its integer decoder; flattening
    // retains the existing decoder for every other option and future additions.
    #[derive(Deserialize)]
    struct Options {
        #[serde(rename = "limit", default)]
        _legacy_limit: Option<serde::de::IgnoredAny>,
        #[serde(flatten)]
        options: pele::compat::ParseOptions,
    }
    if json.trim().is_empty() {
        return Ok(Default::default());
    }
    serde_json::from_str::<Options>(json)
        .map(|value| value.options)
        .map_err(|error| {
            PeleError::new(
                format!("Invalid parse options: {error}"),
                ErrorCode::Semantic,
            )
        })
}
pub(super) fn parse_limits_options(
    json: &str,
    source_overridden: bool,
    edges_overridden: bool,
) -> Result<pele::compat::ParseOptions, PeleError> {
    #[derive(Deserialize)]
    struct Edges {
        #[serde(rename = "maxEdges", default)]
        _legacy_edges: Option<serde::de::IgnoredAny>,
        #[serde(flatten)]
        options: pele::compat::ParseOptions,
    }
    #[derive(Deserialize)]
    struct Both {
        #[serde(rename = "limit", default)]
        _legacy_source: Option<serde::de::IgnoredAny>,
        #[serde(flatten)]
        edges: Edges,
    }
    if json.trim().is_empty() {
        return Ok(Default::default());
    }
    // Only explicitly overridden fields bypass their legacy integer decoders.
    // In particular, tag3 must not hide an invalid value in the JSON options.
    let decoded = match (source_overridden, edges_overridden) {
        (false, false) => return pele::compat::parse_options(json),
        (true, false) => return source_limit_options(json),
        (false, true) => serde_json::from_str::<Edges>(json).map(|value| value.options),
        (true, true) => serde_json::from_str::<Both>(json).map(|value| value.edges.options),
    };
    decoded.map_err(|error| {
        PeleError::new(
            format!("Invalid parse options: {error}"),
            ErrorCode::Semantic,
        )
    })
}
fn output_limit_options(json: &str) -> Result<pele::compat::RenderOptions, PeleError> {
    // Explicit IEEE threshold supersedes only the legacy integer field.
    #[derive(Deserialize)]
    struct Options {
        #[serde(rename = "outputLimit", default)]
        _legacy_output_limit: Option<serde::de::IgnoredAny>,
        #[serde(flatten)]
        options: pele::compat::RenderOptions,
    }
    if json.trim().is_empty() {
        return Ok(Default::default());
    }
    serde_json::from_str::<Options>(json)
        .map(|value| value.options)
        .map_err(|error| {
            PeleError::new(
                format!("Invalid render options: {error}"),
                ErrorCode::Semantic,
            )
        })
}
pub(super) unsafe fn document<'a>(
    handle: *const PelePublicDocument,
) -> Result<MutexGuard<'a, PublicDocument>, PeleError> {
    let owner = unsafe { handle.as_ref() }
        .ok_or_else(|| invalid("Native public transport: null document"))?;
    owner.document.try_lock().map_err(|error| match error {
        TryLockError::WouldBlock => invalid("Native public document is already in use"),
        TryLockError::Poisoned(_) => {
            invalid("Native public document is unavailable after an engine panic")
        }
    })
}
struct Output {
    bytes: Vec<u8>,
    exceeded: bool,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > GRAPH_JSON_BUDGET - self.bytes.len() {
            self.exceeded = true;
            return Err(std::io::Error::other("public result budget"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
// v9 alone bounds outer errors as well as successful envelopes/exchanges.
// Constants are canonical version2 envelopes and avoid recursively serializing
// an oversized source-derived error or an unexpected serialization panic.
const BOUNDED_ERROR_FALLBACK: &[u8] = br#"{"version":2,"ok":false,"error":{"name":"PeleError","message":"Native public transport: result exceeds 64 MiB JSON budget","code":"limit","type":null,"line":0,"column":0,"snippet":""}}"#;
const BOUNDED_PANIC_FALLBACK: &[u8] = br#"{"version":2,"ok":false,"error":{"name":"PeleError","message":"Native transport: unexpected engine failure","code":"semantic","type":null,"line":0,"column":0,"snippet":""}}"#;
fn bounded_failure(error: &PeleError) -> PeleResult {
    #[derive(Serialize)]
    struct Envelope<'a> {
        version: u32,
        ok: bool,
        error: &'a PeleError,
    }
    let mut output = Output {
        bytes: Vec::new(),
        exceeded: false,
    };
    let encoded = serde_json::to_writer(
        &mut output,
        &Envelope {
            version: 2,
            ok: false,
            error,
        },
    );
    if output.exceeded || encoded.is_err() {
        return PeleResult {
            status: 2,
            data: BOUNDED_ERROR_FALLBACK.to_vec(),
            document: None,
        };
    }
    PeleResult {
        status: if error.code == ErrorCode::Limit { 2 } else { 1 },
        data: output.bytes,
        document: None,
    }
}
pub(super) fn bounded_call(run: impl FnOnce() -> Result<PeleResult, PeleError>) -> *mut PeleResult {
    let result = catch_unwind(AssertUnwindSafe(|| match run() {
        Ok(result) if result.data.len() <= GRAPH_JSON_BUDGET => result,
        Ok(_) => PeleResult {
            status: 2,
            data: BOUNDED_ERROR_FALLBACK.to_vec(),
            document: None,
        },
        Err(error) => bounded_failure(&error),
    }))
    .unwrap_or_else(|_| PeleResult {
        status: 1,
        data: BOUNDED_PANIC_FALLBACK.to_vec(),
        document: None,
    });
    Box::into_raw(Box::new(result))
}
pub(super) fn success(value: impl Serialize) -> Result<PeleResult, PeleError> {
    #[derive(Serialize)]
    struct Envelope<T> {
        version: u32,
        ok: bool,
        result: T,
    }
    let mut output = Output {
        bytes: vec![],
        exceeded: false,
    };
    let encoded = serde_json::to_writer(
        &mut output,
        &Envelope {
            version: 2,
            ok: true,
            result: value,
        },
    );
    if output.exceeded {
        return Err(PeleError::new(
            "Native public transport: result exceeds 64 MiB JSON budget",
            ErrorCode::Limit,
        ));
    }
    encoded.map_err(|_| invalid("Native public transport: result cannot be encoded"))?;
    Ok(PeleResult {
        status: 0,
        data: output.bytes,
        document: None,
    })
}
/// Parse an opted-in built-in. On every error `*out_document` is null; success transfers ownership.
/// # Safety
/// out_document must be a valid writable pointer. Other non-null inputs must name readable
/// byte lengths. Source is well-formed UTF-8 (16 MiB), options JSON at most 1 MiB.
#[no_mangle]
pub unsafe extern "C" fn pele_public_parse_v4(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
    out_document: *mut *mut PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let out = unsafe { out_document.as_mut() }
            .ok_or_else(|| invalid("Native public transport: null document output"))?;
        *out = std::ptr::null_mut();
        let source = unsafe { compatibility_input(source, source_len, 16 * 1024 * 1024) }?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::parse_options(options)?;
        let registry = PublicEngine::with_builtins();
        let document = registry.parse(source, &options).map_err(|mut error| {
            if error.code == ErrorCode::UnsupportedDiagram {
                if let Some(kind) = &error.diagram_type {
                    error.message = format!(
                        "Mutable public model codec for diagram type \"{kind}\" is unavailable."
                    );
                }
            }
            error
        })?;
        let result = success(
            serde_json::json!({"type":document.diagram_type(),"revision":document.revision().to_string()}),
        )?;
        *out = owned_document(document);
        Ok(result)
    })
}
/// Parse with the original JavaScript source-limit comparison, independently of
/// the legacy integer options.limit. Other options and v4 ownership are retained.
/// # Safety
/// Inputs must satisfy parse_v4's pointer/length and writable output requirements.
/// Source is well-formed UTF-8; numeric bits are transported without JSON coercion.
#[no_mangle]
pub unsafe extern "C" fn pele_public_parse_source_limit_v6(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
    limit_kind: u32,
    limit_bits: u64,
    out_document: *mut *mut PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let out = unsafe { out_document.as_mut() }
            .ok_or_else(|| invalid("Native public transport: null document output"))?;
        *out = std::ptr::null_mut();
        let source = unsafe {
            compatibility_input(source, source_len, pele::compat::MAX_PUBLIC_SOURCE_BYTES)
        }?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let limit = match limit_kind {
            0 => pele::compat::SourceLimit::Undefined,
            1 => pele::compat::SourceLimit::Null,
            2 => pele::compat::SourceLimit::Number(f64::from_bits(limit_bits)),
            _ => {
                return Err(invalid(
                    "Native public transport: unknown source-limit kind",
                ))
            }
        };
        let options = source_limit_options(options)?;
        let registry = PublicEngine::with_builtins();
        let document = registry
            .parse_with_source_limit(source, &options, limit)
            .map_err(|mut error| {
                if error.code == ErrorCode::UnsupportedDiagram {
                    if let Some(kind) = &error.diagram_type {
                        error.message = format!(
                            "Mutable public model codec for diagram type \"{kind}\" is unavailable."
                        );
                    }
                }
                error
            })?;
        let result = success(
            serde_json::json!({"type":document.diagram_type(),"revision":document.revision().to_string()}),
        )?;
        *out = owned_document(document);
        Ok(result)
    })
}
pub(super) fn decode_parse_limits(
    source_kind: u32,
    source_bits: u64,
    edge_kind: u32,
    edge_bits: u64,
) -> Result<pele::compat::ParseLimits, PeleError> {
    let source_limit = match source_kind {
        0 => Some(pele::compat::SourceLimit::Undefined),
        1 => Some(pele::compat::SourceLimit::Null),
        2 => Some(pele::compat::SourceLimit::Number(f64::from_bits(
            source_bits,
        ))),
        3 => None,
        _ => {
            return Err(invalid(
                "Native public transport: unknown source-limit kind",
            ))
        }
    };
    let edge_limit = match edge_kind {
        0 => Some(pele::compat::EdgeLimit::Undefined),
        1 => Some(pele::compat::EdgeLimit::Null),
        2 => Some(pele::compat::EdgeLimit::Number(f64::from_bits(edge_bits))),
        3 => None,
        _ => return Err(invalid("Native public transport: unknown edge-limit kind")),
    };
    Ok(pele::compat::ParseLimits {
        source: source_limit,
        max_edges: edge_limit,
    })
}
pub(super) fn publish_document(
    document: PublicDocument,
    out: &mut *mut PelePublicDocument,
) -> Result<PeleResult, PeleError> {
    let result = success(
        serde_json::json!({"type":document.diagram_type(),"revision":document.revision().to_string()}),
    )?;
    *out = owned_document(document);
    Ok(result)
}
/// Independently select exact source/edge thresholds or legacy integer options.
/// # Safety
/// Pointer ownership, budgets and writable output requirements match parse_v4.
#[no_mangle]
pub unsafe extern "C" fn pele_public_parse_limits_v7(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
    source_kind: u32,
    source_bits: u64,
    edge_kind: u32,
    edge_bits: u64,
    out_document: *mut *mut PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let out = unsafe { out_document.as_mut() }
            .ok_or_else(|| invalid("Native public transport: null document output"))?;
        *out = std::ptr::null_mut();
        let source = unsafe {
            compatibility_input(source, source_len, pele::compat::MAX_PUBLIC_SOURCE_BYTES)
        }?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let limits = decode_parse_limits(source_kind, source_bits, edge_kind, edge_bits)?;
        let options =
            parse_limits_options(options, limits.source.is_some(), limits.max_edges.is_some())?;
        let registry = PublicEngine::with_builtins();
        let document = registry.parse_with_limits(source, &options, limits).map_err(|mut error| {
            if error.code == ErrorCode::UnsupportedDiagram {
                if let Some(kind) = &error.diagram_type {
                    error.message = format!("Mutable public model codec for diagram type \"{kind}\" is unavailable.");
                }
            }
            error
        })?;
        publish_document(document, out)
    })
}
/// Flat graph schema 1 plus decimal-string revision. Result JSON is limited to 64 MiB.
/// # Safety
/// handle must be null or a live owned document, not concurrently freed.
#[no_mangle]
pub unsafe extern "C" fn pele_public_snapshot_v4(
    handle: *const PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { document(handle) }?;
        #[derive(Serialize)]
        struct Snapshot<'a> {
            r#type: &'a pele::compat::DiagramId,
            revision: String,
            graph: GraphReadView<'a>,
        }
        success(Snapshot {
            r#type: document.diagram_type(),
            revision: document.revision().to_string(),
            graph: document.snapshot_read()?,
        })
    })
}
/// Read only the current committed revision, without graph export or validation.
/// # Safety
/// handle must be null or a live owned document, not concurrently freed.
#[no_mangle]
pub unsafe extern "C" fn pele_public_revision_v15(
    handle: *const PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let revision = unsafe { document(handle) }?.revision();
        #[derive(Serialize)]
        struct Revision {
            revision: String,
        }
        success(Revision {
            revision: revision.to_string(),
        })
    })
}
/// Schema-declared fields, prototypes and native method IDs for this document.
/// # Safety
/// handle must be null or live and not concurrently freed.
#[no_mangle]
pub unsafe extern "C" fn pele_public_schema_v4(
    handle: *const PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { document(handle) }?;
        success(document.schema_view())
    })
}
/// Publish post-call graph and revision on both return and operation error.
/// Preflight errors (stale revision, bad method/refs/JSON) do not dispatch.
/// Export/transport budget errors can occur after mutation; an operation result
/// is only delivered when its post-call graph fits the transport budget.
/// # Safety
/// handle must be null or live, call names readable UTF-8 JSON up to 64 MiB;
/// resources/callbacks obey the same lifetime rules as render v3.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invoke_v4(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    call: *const u8,
    call_len: usize,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    compatibility_call(|| {
        let call = unsafe { compatibility_input(call, call_len, GRAPH_JSON_BUDGET) }?;
        let call: pele::compat::public::MethodCall =
            serde_json::from_str(call).map_err(|_| invalid("Invalid public model method call"))?;
        let mut document = unsafe { document(handle) }?;
        let outcome = unsafe {
            calendar::with_resources(resources, |host| {
                document.invoke_read(expected_revision, call, host)
            })
        }?;
        outcome_success(outcome.revision, &outcome.graph, &outcome.result)
    })
}
/// Apply a full flat graph without reparsing source. A stale revision leaves state unchanged.
/// # Safety
/// handle must be null or live; graph must name readable UTF-8 JSON bytes (at most 64 MiB).
#[no_mangle]
pub unsafe extern "C" fn pele_public_apply_v4(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    graph: *const u8,
    graph_len: usize,
) -> *mut PeleResult {
    compatibility_call(|| {
        let graph = unsafe { compatibility_input(graph, graph_len, GRAPH_JSON_BUDGET) }?;
        let graph: ModelGraph = serde_json::from_str(graph)
            .map_err(|_| invalid("Native public transport: invalid model graph JSON"))?;
        let mut document = unsafe { document(handle) }?;
        document.apply(expected_revision, graph)?;
        success(serde_json::json!({"revision":document.revision().to_string()}))
    })
}
/// Render the retained mutable Rust model with the existing v3 callback contract.
/// # Safety
/// handle must be null or live; options/host/context/callback borrows remain valid synchronously.
/// Reentrant operations on this same document return an error instead of blocking.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_v4(
    handle: *mut PelePublicDocument,
    options: *const u8,
    options_len: usize,
    host: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    compatibility_call(|| {
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::render_options(options)?;
        let mut document = unsafe { document(handle) }?;
        let result = unsafe {
            calendar::with_resources(host, |resources| document.render(&options, resources))
        }?;
        // Existing dimension envelope preserves nonfinite/negative-zero result bits.
        let bytes = pele::compat::render_result_envelope(&result).into_bytes();
        if bytes.len() > GRAPH_JSON_BUDGET {
            return Err(PeleError::new(
                "Native public transport: result exceeds 64 MiB JSON budget",
                ErrorCode::Limit,
            ));
        }
        Ok(PeleResult {
            status: 0,
            data: bytes,
            document: None,
        })
    })
}
/// Stateful render outcome: graph/revision publish prior mutations on success/error.
/// # Safety
/// Same handle, byte and callback ownership rules as render_v4; no concurrent free.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_operation_v4(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    options: *const u8,
    options_len: usize,
    host: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    compatibility_call(|| {
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::render_options(options)?;
        let mut document = unsafe { document(handle) }?;
        let outcome = unsafe {
            calendar::with_resources(host, |resources| {
                document.render_operation_read(expected_revision, &options, resources)
            })
        }?;
        outcome_success(
            outcome.revision,
            &outcome.graph,
            outcome
                .result
                .as_ref()
                .map(pele::compat::render_result_transport),
        )
    })
}
/// # Safety
/// Free null or one live owned handle exactly once, after all operations and callbacks return.
#[no_mangle]
pub unsafe extern "C" fn pele_public_document_free_v4(handle: *mut PelePublicDocument) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }));
}

fn outcome_success<T: Serialize>(
    revision: u64,
    graph: &GraphReadView<'_>,
    result: T,
) -> Result<PeleResult, PeleError> {
    #[derive(Serialize)]
    struct Outcome<'a, 'graph, T> {
        revision: u64,
        graph: &'a GraphReadView<'graph>,
        result: T,
    }
    success(Outcome {
        revision,
        graph,
        result,
    })
}

fn binary_success(
    revision: u64,
    graph: &GraphReadView<'_>,
    metadata: &[u8],
) -> Result<PeleResult, PeleError> {
    let data = graph.encode_binary(revision, metadata, GRAPH_JSON_BUDGET)?;
    Ok(PeleResult {
        status: 0,
        data,
        document: None,
    })
}
/// Binary graph schema1, revision and JSON metadata; graph references never expand.
/// # Safety
/// Same live-handle and synchronous callback ownership as the v4 operations.
#[no_mangle]
pub unsafe extern "C" fn pele_public_snapshot_binary_v5(
    handle: *const PelePublicDocument,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { document(handle) }?;
        let metadata = serde_json::to_vec(&serde_json::json!({"type":document.diagram_type()}))
            .map_err(|_| invalid("Invalid snapshot metadata"))?;
        let graph = document.snapshot_read()?;
        binary_success(document.revision(), &graph, &metadata)
    })
}
/// # Safety
/// Graph names a readable binary exchange up to64MiB; handle is live or null.
#[no_mangle]
pub unsafe extern "C" fn pele_public_apply_binary_v5(
    handle: *mut PelePublicDocument,
    graph: *const u8,
    len: usize,
) -> *mut PeleResult {
    compatibility_call(|| {
        if len > GRAPH_JSON_BUDGET {
            return Err(PeleError::new(
                "Public binary graph exceeds transport budget",
                ErrorCode::Limit,
            ));
        }
        let bytes = if len == 0 {
            &[][..]
        } else {
            if graph.is_null() {
                return Err(invalid("Native public transport: null input"));
            }
            unsafe { std::slice::from_raw_parts(graph, len) }
        };
        let exchange = pele::compat::public::decode_binary_exchange(bytes, GRAPH_JSON_BUDGET)?;
        if !exchange.metadata.is_empty() {
            return Err(invalid("Apply binary metadata must be empty"));
        }
        let mut document = unsafe { document(handle) }?;
        document.apply(exchange.revision, exchange.graph)?;
        success(serde_json::json!({"revision":document.revision().to_string()}))
    })
}
/// # Safety
/// Same call/host ownership as invoke_v4; result graph uses binary schema1.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invoke_binary_v5(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    call: *const u8,
    call_len: usize,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    compatibility_call(|| {
        let call = unsafe { compatibility_input(call, call_len, GRAPH_JSON_BUDGET) }?;
        let call: pele::compat::public::MethodCall =
            serde_json::from_str(call).map_err(|_| invalid("Invalid public model method call"))?;
        let mut document = unsafe { document(handle) }?;
        let outcome = unsafe {
            calendar::with_resources(resources, |host| {
                document.invoke_read(expected_revision, call, host)
            })
        }?;
        let metadata = serde_json::to_vec(&outcome.result)
            .map_err(|_| invalid("Invalid invocation metadata"))?;
        binary_success(outcome.revision, &outcome.graph, &metadata)
    })
}
/// # Safety
/// Same options/host ownership as render_operation_v4; result graph is binary.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_operation_binary_v5(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    options: *const u8,
    options_len: usize,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    compatibility_call(|| {
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::render_options(options)?;
        let mut document = unsafe { document(handle) }?;
        let outcome = unsafe {
            calendar::with_resources(resources, |host| {
                document.render_operation_read(expected_revision, &options, host)
            })
        }?;
        let metadata = serde_json::to_vec(
            &outcome
                .result
                .as_ref()
                .map(pele::compat::render_result_transport),
        )
        .map_err(|_| invalid("Invalid render metadata"))?;
        binary_success(outcome.revision, &outcome.graph, &metadata)
    })
}

/// Exact IEEE output threshold with binary graph/revision and bounded metadata.
/// # Safety
/// Same live-handle/options/callback ownership as render_operation_binary_v5.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_operation_output_limit_v6(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    options: *const u8,
    options_len: usize,
    limit_kind: u32,
    limit_bits: u64,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    compatibility_call(|| {
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = output_limit_options(options)?;
        let limit = match limit_kind {
            0 => pele::compat::OutputLimit::Undefined,
            1 => pele::compat::OutputLimit::Null,
            2 => pele::compat::OutputLimit::Number(f64::from_bits(limit_bits)),
            _ => {
                return Err(invalid(
                    "Native public transport: unknown output-limit kind",
                ))
            }
        };
        let mut document = unsafe { document(handle) }?;
        let outcome = unsafe {
            calendar::with_resources(resources, |host| {
                document.render_operation_with_output_limit_read(
                    expected_revision,
                    &options,
                    limit,
                    host,
                )
            })
        }?;
        render_outcome(&outcome)
    })
}

pub(super) fn render_outcome(
    outcome: &ReadOutcome<'_, pele::compat::RenderResult>,
) -> Result<PeleResult, PeleError> {
    binary_metadata(
        outcome.revision,
        &outcome.graph,
        &outcome
            .result
            .as_ref()
            .map(pele::compat::render_result_transport),
    )
}
pub(super) fn binary_metadata(
    revision: u64,
    graph: &GraphReadView<'_>,
    value: &impl Serialize,
) -> Result<PeleResult, PeleError> {
    let mut metadata = Output {
        bytes: Vec::new(),
        exceeded: false,
    };
    let encoded = serde_json::to_writer(&mut metadata, value);
    if metadata.exceeded {
        return Err(PeleError::new(
            "Native public transport: result exceeds 64 MiB JSON budget",
            ErrorCode::Limit,
        ));
    }
    encoded.map_err(|_| invalid("Invalid render metadata"))?;
    binary_success(revision, graph, &metadata.bytes)
}
/// # Safety
/// Same document ownership as v5; packet/options/resources are synchronous borrows.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_operation_with_numbers_v9(
    handle: *mut PelePublicDocument,
    expected_revision: u64,
    bytes: *const u8,
    len: usize,
    numbers: *const super::PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    bounded_call(|| {
        let (options, limit) =
            unsafe { super::render_numbers::options(bytes, len, numbers, kind, bits) }?;
        let mut document = unsafe { document(handle) }?;
        let outcome = unsafe {
            super::host_functions::with_resources(handle, resources, |host| {
                document.render_operation_bounded_read(expected_revision, &options, limit, host)
            })
        }?;
        render_outcome(&outcome)
    })
}
