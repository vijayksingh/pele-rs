//! Named low-level Diagram parse/render, independently of core source preparation.
use super::{
    compatibility_input, host_functions, public, render_numbers, PeleHostResourcesV3,
    PeleRenderNumbersV9, PeleResult,
};
use pele::compat::{
    self, public::PublicEngine, DiagramId, EdgeLimit, Object, ParseContext, PeleError,
};
use std::ptr;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PelePreparedOptionsV13 {
    pub version: u32,
    pub struct_size: u32,
    pub line_offset: u64,
    pub max_edges_kind: u32,
    pub title_present: u32,
    pub max_edges_bits: u64,
    pub reserved: [u64; 2],
}
unsafe fn packet(
    pointer: *const PelePreparedOptionsV13,
) -> Result<(usize, bool, EdgeLimit), PeleError> {
    if pointer.is_null() {
        return Ok((0, false, EdgeLimit::Undefined));
    }
    let header = pointer.cast::<u32>();
    if unsafe { ptr::read_unaligned(header) } != 13
        || unsafe { ptr::read_unaligned(header.add(1)) } as usize
            != std::mem::size_of::<PelePreparedOptionsV13>()
    {
        return Err(public::invalid(
            "Native prepared packet version or size is invalid",
        ));
    }
    let value = unsafe { ptr::read_unaligned(pointer) };
    if value.reserved != [0; 2] || value.title_present > 1 {
        return Err(public::invalid(
            "Native prepared packet flags or reserved fields are invalid",
        ));
    }
    let edge = match value.max_edges_kind {
        0 => EdgeLimit::Undefined,
        1 => EdgeLimit::Null,
        2 => EdgeLimit::Number(f64::from_bits(value.max_edges_bits)),
        _ => return Err(public::invalid("Native prepared edge policy is invalid")),
    };
    let offset = usize::try_from(value.line_offset)
        .map_err(|_| public::invalid("Native prepared line offset is out of range"))?;
    Ok((offset, value.title_present != 0, edge))
}
unsafe fn config(bytes: *const u8, len: usize) -> Result<Object, PeleError> {
    let text = unsafe { compatibility_input(bytes, len, 1024 * 1024) }?;
    if text.is_empty() {
        return Ok(Object::default());
    }
    serde_json::from_str(text)
        .map_err(|error| public::invalid(&format!("Invalid prepared config: {error}")))
}
/// # Safety
/// All input spans/packet are readable for this call. Output is writable and
/// receives an owned normal public document only after successful serialization.
#[no_mangle]
pub unsafe extern "C" fn pele_parse_prepared_v13(
    kind: *const u8,
    kind_len: usize,
    source: *const u8,
    source_len: usize,
    config_bytes: *const u8,
    config_len: usize,
    title: *const u8,
    title_len: usize,
    options: *const PelePreparedOptionsV13,
    out: *mut *mut public::PelePublicDocument,
) -> *mut PeleResult {
    public::bounded_call(|| {
        if out.is_null() {
            return Err(public::invalid("Native prepared output document is null"));
        }
        unsafe {
            *out = ptr::null_mut();
        }
        if kind_len > 1024
            || source_len > compat::MAX_PUBLIC_SOURCE_BYTES
            || config_len > 1024 * 1024
            || title_len > 1024 * 1024
        {
            return Err(PeleError::new(
                "Native prepared declared input exceeds budget",
                compat::ErrorCode::Limit,
            ));
        }
        let (line_offset, has_title, edge_limit) = unsafe { packet(options) }?;
        let name = unsafe { compatibility_input(kind, kind_len, 1024) }?;
        let id = super::registry::builtin(name)
            .map(DiagramId::from)
            .unwrap_or_else(|| DiagramId::extension(name));
        let source =
            unsafe { compatibility_input(source, source_len, compat::MAX_PUBLIC_SOURCE_BYTES) }?;
        let config = unsafe { config(config_bytes, config_len) }?;
        let title = if has_title {
            Some(unsafe { compatibility_input(title, title_len, 1024 * 1024) }?)
        } else {
            None
        };
        let document = PublicEngine::with_builtins().parse_prepared_as(
            &id,
            ParseContext {
                source,
                config: &config,
                title,
                line_offset,
                max_edges: 5000,
            },
            edge_limit,
        )?;
        let result = public::success(
            serde_json::json!({"type":document.diagram_type(), "revision":document.revision().to_string(), "modelKind":"prepared"}),
        )?;
        unsafe {
            *out = public::owned_document(document);
        }
        Ok(result)
    })
}
/// # Safety
/// Normal public-document ownership; all input spans/resources remain borrowed
/// synchronously. No core default configuration, preprocessing or output policy.
#[no_mangle]
pub unsafe extern "C" fn pele_render_prepared_v13(
    handle: *mut public::PelePublicDocument,
    expected_revision: u64,
    config_bytes: *const u8,
    config_len: usize,
    options: *const u8,
    options_len: usize,
    numbers: *const PeleRenderNumbersV9,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let config = unsafe { config(config_bytes, config_len) }?;
        // Raw Diagram options have no core output-limit policy. The v9 packet
        // supplies raw scalar bits, while that unrelated JSON field is ignored.
        let settings = unsafe { render_numbers::raw_options(options, options_len, numbers) }?;
        let mut document = unsafe { public::document(handle) }?;
        let kind = document.diagram_type().clone();
        let outcome = unsafe {
            host_functions::with_resources(handle, resources, |host| {
                document.render_raw_operation_read(expected_revision, &config, &settings, host)
            })
        }?;
        let result = outcome.result.map(|rendered| compat::RenderResult {
            diagram_type: kind,
            rendered,
        });
        let wire = result.as_ref().map(compat::render_result_transport);
        public::binary_metadata(outcome.revision, &outcome.graph, &wire)
    })
}

/// # Safety
/// The input is a bounded PELGBIN1 exchange with revision zero and no metadata.
/// Each borrowed span remains readable through this synchronous construction.
#[no_mangle]
pub unsafe extern "C" fn pele_create_prepared_v13(
    kind: *const u8,
    kind_len: usize,
    graph: *const u8,
    graph_len: usize,
    config_bytes: *const u8,
    config_len: usize,
    out: *mut *mut public::PelePublicDocument,
) -> *mut PeleResult {
    public::bounded_call(|| {
        if out.is_null() {
            return Err(public::invalid("Native prepared output document is null"));
        }
        unsafe {
            *out = ptr::null_mut();
        }
        if kind_len > 1024 || graph_len > 64 * 1024 * 1024 || config_len > 1024 * 1024 {
            return Err(PeleError::new(
                "Native prepared declared input exceeds budget",
                compat::ErrorCode::Limit,
            ));
        }
        let name = unsafe { compatibility_input(kind, kind_len, 1024) }?;
        let id = super::registry::builtin(name)
            .map(DiagramId::from)
            .unwrap_or_else(|| DiagramId::extension(name));
        let bytes = if graph_len == 0 {
            &[][..]
        } else {
            if graph.is_null() {
                return Err(public::invalid("Native prepared graph input is null"));
            }
            unsafe { std::slice::from_raw_parts(graph, graph_len) }
        };
        let exchange = compat::public::decode_binary_exchange(bytes, 64 * 1024 * 1024)?;
        if exchange.revision != 0 || !exchange.metadata.is_empty() {
            return Err(public::invalid(
                "Prepared construction requires revision zero and empty metadata",
            ));
        }
        let config = unsafe { config(config_bytes, config_len) }?;
        let document =
            PublicEngine::with_builtins().create_from_graph_as(&id, exchange.graph, config)?;
        let result = public::success(
            serde_json::json!({"type":document.diagram_type(),"revision":"0","modelKind":"prepared"}),
        )?;
        unsafe {
            *out = public::owned_document(document);
        }
        Ok(result)
    })
}
/// # Safety
/// The named type span remains readable for this synchronous schema query.
#[no_mangle]
pub unsafe extern "C" fn pele_prepared_schema_v13(
    kind: *const u8,
    kind_len: usize,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let name = unsafe { compatibility_input(kind, kind_len, 1024) }?;
        let id = super::registry::builtin(name)
            .map(DiagramId::from)
            .unwrap_or_else(|| DiagramId::extension(name));
        public::success(PublicEngine::with_builtins().schema_for(&id)?)
    })
}
