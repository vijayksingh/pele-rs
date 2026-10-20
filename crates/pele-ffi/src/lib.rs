//! Opaque, immutable owned results. Unsafe code is confined to the C adapter.
mod constructors;
pub use constructors::*;
mod detached_render;
pub use detached_render::*;
mod eager_projection;
pub use eager_projection::*;
mod host_functions;
pub use host_functions::*;
mod public;
pub use public::*;
mod render_numbers;
pub use render_numbers::*;
mod providers;
pub use providers::*;
mod native_resources;
pub use native_resources::*;
mod presentation;
pub use presentation::*;
mod registry;
pub use registry::*;
mod calendar;
pub use calendar::PeleHostResourcesV3;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    slice,
};

pub struct PeleResult {
    status: u32,
    data: Vec<u8>,
    document: Option<pele::compat::ParsedDocument>,
}
#[no_mangle]
pub extern "C" fn pele_abi_version() -> u32 {
    2
}

/// # Safety
/// Non-null result must be a live handle returned by a Pele operation.
#[no_mangle]
pub unsafe extern "C" fn pele_result_status(result: *const PeleResult) -> u32 {
    unsafe { result.as_ref() }.map_or(3, |r| r.status)
}
/// # Safety
/// Non-null result must be a live handle. Bytes are borrowed until free.
#[no_mangle]
pub unsafe extern "C" fn pele_result_data(result: *const PeleResult) -> *const u8 {
    unsafe { result.as_ref() }.map_or(std::ptr::null(), |r| r.data.as_ptr())
}
/// # Safety
/// Non-null result must be a live handle.
#[no_mangle]
pub unsafe extern "C" fn pele_result_len(result: *const PeleResult) -> usize {
    unsafe { result.as_ref() }.map_or(0, |r| r.data.len())
}
/// # Safety
/// Result must be null or a live result; free exactly once, after readers finish.
#[no_mangle]
pub unsafe extern "C" fn pele_result_free(result: *mut PeleResult) {
    if !result.is_null() {
        drop(unsafe { Box::from_raw(result) });
    }
}

/// Full-interface source preparation. JSON envelope version 2; no diagram rendering implied.
/// # Safety
/// Non-null pointers must name their declared readable byte lengths for this call.
#[no_mangle]
pub unsafe extern "C" fn pele_inspect(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
) -> *mut PeleResult {
    fn failure(message: &str, limit: bool) -> PeleResult {
        let error = pele::compat::PeleError::new(
            message,
            if limit {
                pele::compat::ErrorCode::Limit
            } else {
                pele::compat::ErrorCode::Semantic
            },
        );
        PeleResult {
            status: if limit { 2 } else { 1 },
            data: pele::compat::error_envelope(&error).into_bytes(),
            document: None,
        }
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        if source_len > 16 * 1024 * 1024 || options_len > 1024 * 1024 {
            return failure(
                "Native transport: declared input exceeds the 16 MiB source / 1 MiB options budget",
                true,
            );
        }
        if (source.is_null() && source_len != 0) || (options.is_null() && options_len != 0) {
            return failure("Native transport: null input with nonzero length", false);
        }
        let s = if source_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(source, source_len) }
        };
        let o = if options_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(options, options_len) }
        };
        let (Ok(s), Ok(o)) = (std::str::from_utf8(s), std::str::from_utf8(o)) else {
            return failure("Native transport: input must be UTF-8", false);
        };
        let (status, data) = pele::compat::inspect_envelope(s, o);
        PeleResult {
            status,
            data: data.into_bytes(),
            document: None,
        }
    }))
    .unwrap_or_else(|_| failure("Native transport: unexpected engine failure", false));
    Box::into_raw(Box::new(result))
}
#[no_mangle]
pub extern "C" fn pele_capabilities() -> *mut PeleResult {
    Box::into_raw(Box::new(PeleResult {
        status: 0,
        data: pele::compat::capabilities_json().into_bytes(),
        document: None,
    }))
}

fn compatibility_failure(error: pele::compat::PeleError) -> PeleResult {
    PeleResult {
        status: if error.code == pele::compat::ErrorCode::Limit {
            2
        } else {
            1
        },
        data: pele::compat::error_envelope(&error).into_bytes(),
        document: None,
    }
}
fn compatibility_call(
    run: impl FnOnce() -> Result<PeleResult, pele::compat::PeleError>,
) -> *mut PeleResult {
    let result = catch_unwind(AssertUnwindSafe(run))
        .unwrap_or_else(|_| {
            Err(pele::compat::PeleError::new(
                "Native transport: unexpected engine failure",
                pele::compat::ErrorCode::Semantic,
            ))
        })
        .unwrap_or_else(compatibility_failure);
    Box::into_raw(Box::new(result))
}
unsafe fn compatibility_input<'a>(
    ptr: *const u8,
    len: usize,
    max: usize,
) -> Result<&'a str, pele::compat::PeleError> {
    use pele::compat::{ErrorCode, PeleError};
    if len > max {
        return Err(PeleError::new(
            "Native transport: declared input exceeds budget",
            ErrorCode::Limit,
        ));
    }
    if len == 0 {
        return Ok("");
    }
    if ptr.is_null() {
        return Err(PeleError::new(
            "Native transport: null input with nonzero length",
            ErrorCode::Semantic,
        ));
    }
    std::str::from_utf8(unsafe { slice::from_raw_parts(ptr, len) })
        .map_err(|_| PeleError::new("Native transport: input must be UTF-8", ErrorCode::Semantic))
}
// Shared by the legacy JSON entry and eager Swift projection entry. Admission,
// option decoding and captured native registration stay on one parse path.
unsafe fn parse_document_input(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
) -> Result<pele::compat::ParsedDocument, pele::compat::PeleError> {
    let source = unsafe { compatibility_input(source, source_len, 16 * 1024 * 1024) }?;
    let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
    let options = pele::compat::parse_options(options)?;
    pele::compat::Engine::with_builtins().parse(source, &options)
}
/// Parse once. The owned result also retains the immutable Rust document until result_free.
/// # Safety
/// Non-null inputs must name readable byte lengths for this call.
#[no_mangle]
pub unsafe extern "C" fn pele_parse_v2(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { parse_document_input(source, source_len, options, options_len) }?;
        let data = pele::compat::document_envelope(&document)?.into_bytes();
        Ok(PeleResult {
            status: 0,
            data,
            document: Some(document),
        })
    })
}
/// # Safety
/// Inputs must name their declared readable lengths for this synchronous call.
#[no_mangle]
pub unsafe extern "C" fn pele_render_v2(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
) -> *mut PeleResult {
    compatibility_call(|| {
        let source = unsafe { compatibility_input(source, source_len, 16 * 1024 * 1024) }?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let (status, data) = pele::compat::render_envelope(source, options);
        Ok(PeleResult {
            status,
            data: data.into_bytes(),
            document: None,
        })
    })
}
/// Renders a live parse result without reparsing; parse config/limits in options are ignored.
/// # Safety
/// Document must be a live parse result, options must name readable bytes. Borrow ends on return.
#[no_mangle]
pub unsafe extern "C" fn pele_render_document_v2(
    document: *const PeleResult,
    options: *const u8,
    options_len: usize,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { document.as_ref() }
            .and_then(|r| r.document.as_ref())
            .ok_or_else(|| {
                pele::compat::PeleError::new(
                    "render() takes diagram text or a model that parse() returned.",
                    pele::compat::ErrorCode::Semantic,
                )
            })?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::render_options(options)?;
        let rendered = pele::compat::Engine::empty().render_document(
            document,
            &options,
            &mut pele::compat::text::MetricsResources,
        )?;
        Ok(PeleResult {
            status: 0,
            data: pele::compat::render_result_envelope(&rendered).into_bytes(),
            document: None,
        })
    })
}
#[no_mangle]
pub extern "C" fn pele_builtin_capabilities() -> *mut PeleResult {
    Box::into_raw(Box::new(PeleResult {
        status: 0,
        data: pele::compat::builtin_capabilities_json().into_bytes(),
        document: None,
    }))
}

#[repr(C)]
pub struct PeleBorrowedBytes {
    pub data: *const u8,
    pub len: usize,
}
#[repr(C)]
pub struct PeleHostResources {
    pub context: *mut std::ffi::c_void,
    pub width: Option<
        unsafe extern "C" fn(
            *mut std::ffi::c_void,
            *const u8,
            usize,
            f64,
            u32,
            *const u8,
            usize,
            *const u8,
            usize,
            *mut f64,
        ) -> u32,
    >,
    pub icon: Option<
        unsafe extern "C" fn(
            *mut std::ffi::c_void,
            *const u8,
            usize,
            *mut PeleBorrowedBytes,
        ) -> u32,
    >,
    pub now: Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut f64) -> u32>,
}
struct NativeResources<'a> {
    host: Option<&'a PeleHostResources>,
}
impl pele::compat::RenderResources for NativeResources<'_> {
    fn text_width(
        &mut self,
        r: pele::compat::TextRequest<'_>,
    ) -> Result<f64, pele::compat::PeleError> {
        if let Some((host, width)) = self.host.and_then(|h| h.width.map(|w| (h, w))) {
            let mut out = 0.;
            let font = r
                .font_family
                .map_or((std::ptr::null(), 0), |s| (s.as_ptr(), s.len()));
            let mono = r
                .font_family_mono
                .map_or((std::ptr::null(), 0), |s| (s.as_ptr(), s.len()));
            let status = unsafe {
                width(
                    host.context,
                    r.text.as_ptr(),
                    r.text.len(),
                    r.size,
                    r.style as u32,
                    font.0,
                    font.1,
                    mono.0,
                    mono.1,
                    &mut out,
                )
            };
            if status != 0 {
                return Err(pele::compat::PeleError::new(
                    "Host text measurer failed",
                    pele::compat::ErrorCode::Semantic,
                ));
            }
            Ok(out)
        } else {
            Ok(pele::compat::text::metrics_width(r.text, r.size, r.style))
        }
    }
    fn icon(
        &mut self,
        name: &str,
    ) -> Result<Option<pele::compat::TrustedSvg>, pele::compat::PeleError> {
        let Some((host, icon)) = self.host.and_then(|h| h.icon.map(|i| (h, i))) else {
            return Ok(None);
        };
        let mut bytes = PeleBorrowedBytes {
            data: std::ptr::null(),
            len: 0,
        };
        let status = unsafe { icon(host.context, name.as_ptr(), name.len(), &mut bytes) };
        if status != 0 {
            return Err(pele::compat::PeleError::new(
                "Host icon resolver failed",
                pele::compat::ErrorCode::Semantic,
            ));
        }
        if bytes.data.is_null() && bytes.len == 0 {
            return Ok(None);
        }
        let text = unsafe { compatibility_input(bytes.data, bytes.len, 8 * 1024 * 1024) }?;
        Ok(Some(pele::compat::TrustedSvg::from_trusted_markup(
            text.into(),
        )))
    }
    fn now_millis(&mut self) -> Result<f64, pele::compat::PeleError> {
        if let Some((host, now)) = self.host.and_then(|h| h.now.map(|n| (h, n))) {
            let mut out = 0.;
            if unsafe { now(host.context, &mut out) } != 0 {
                return Err(pele::compat::PeleError::new(
                    "Host clock failed",
                    pele::compat::ErrorCode::Semantic,
                ));
            }
            Ok(out)
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|t| t.as_secs_f64() * 1000.)
                .map_err(|_| {
                    pele::compat::PeleError::new(
                        "Host clock predates the epoch",
                        pele::compat::ErrorCode::Semantic,
                    )
                })
        }
    }
}
/// # Safety
/// Inputs and optional callbacks/context must remain valid synchronously. Callbacks return
/// borrowed UTF-8 valid until their next invocation; the adapter copies icon bytes immediately.
#[no_mangle]
pub unsafe extern "C" fn pele_render_with_resources_v2(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
    host: *const PeleHostResources,
) -> *mut PeleResult {
    compatibility_call(|| {
        let source = unsafe { compatibility_input(source, source_len, 16 * 1024 * 1024) }?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::render_options(options)?;
        let mut resources = NativeResources {
            host: unsafe { host.as_ref() },
        };
        let rendered = pele::compat::Engine::with_builtins().render_source(
            source,
            &options,
            &mut resources,
        )?;
        Ok(PeleResult {
            status: 0,
            data: pele::compat::render_result_envelope(&rendered).into_bytes(),
            document: None,
        })
    })
}
/// # Safety
/// Document must be a live parse result. Optional resource callbacks follow the same synchronous
/// borrow contract as pele_render_with_resources_v2, and are never retained after return.
#[no_mangle]
pub unsafe extern "C" fn pele_render_document_with_resources_v2(
    document: *const PeleResult,
    options: *const u8,
    options_len: usize,
    host: *const PeleHostResources,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { document.as_ref() }
            .and_then(|r| r.document.as_ref())
            .ok_or_else(|| {
                pele::compat::PeleError::new(
                    "render() takes diagram text or a model that parse() returned.",
                    pele::compat::ErrorCode::Semantic,
                )
            })?;
        let options = unsafe { compatibility_input(options, options_len, 1024 * 1024) }?;
        let options = pele::compat::render_options(options)?;
        let mut resources = NativeResources {
            host: unsafe { host.as_ref() },
        };
        let rendered =
            pele::compat::Engine::empty().render_document(document, &options, &mut resources)?;
        Ok(PeleResult {
            status: 0,
            data: pele::compat::render_result_envelope(&rendered).into_bytes(),
            document: None,
        })
    })
}

/// Query a live retained document. All buffers are borrowed for this call.
/// # Safety
/// Document must remain live; non-null buffers must cover their declared lengths.
#[no_mangle]
pub unsafe extern "C" fn pele_document_query_v2(
    document: *const PeleResult,
    operation: *const u8,
    operation_len: usize,
    arguments: *const u8,
    arguments_len: usize,
) -> *mut PeleResult {
    compatibility_call(|| {
        use pele::compat::{ErrorCode, PeleError, Value};
        let document = unsafe { document.as_ref() }
            .and_then(|r| r.document.as_ref())
            .ok_or_else(|| {
                PeleError::new("Query requires a live parsed document", ErrorCode::Semantic)
            })?;
        let operation = unsafe { compatibility_input(operation, operation_len, 4096) }?;
        let arguments = unsafe { compatibility_input(arguments, arguments_len, 1024 * 1024) }?;
        let arguments: Value = if arguments.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(arguments)
                .map_err(|_| PeleError::new("Invalid model query arguments", ErrorCode::Semantic))?
        };
        let result = document.query(operation, &arguments)?;
        Ok(PeleResult {
            status: 0,
            data: pele::compat::success_envelope(&result).into_bytes(),
            document: None,
        })
    })
}

mod prepared;
pub use prepared::*;
