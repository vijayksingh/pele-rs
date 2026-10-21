//! Additive native presentation lane; existing render exports remain unchanged.
use super::{
    calendar, compatibility_input, public, render_numbers, PeleHostResourcesV3,
    PeleRenderNumbersV9, PeleResult,
};
use pele::compat::{
    self, PresentationBudget, PresentationCapture, PresentationOutcome, PresentationUnavailable,
};
use serde::Serialize;
use std::ptr;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PelePresentationOptionsV11 {
    pub version: u32,
    pub struct_size: u32,
    pub max_elements: u64,
    pub max_text_bytes: u64,
    pub reserved: [u64; 2],
}
pub(crate) unsafe fn budget(
    packet: *const PelePresentationOptionsV11,
) -> Result<PresentationBudget, compat::PeleError> {
    if packet.is_null() {
        return Ok(PresentationBudget::default());
    }
    let header = packet.cast::<u32>();
    if unsafe { ptr::read_unaligned(header) } != 11
        || unsafe { ptr::read_unaligned(header.add(1)) } as usize
            != std::mem::size_of::<PelePresentationOptionsV11>()
    {
        return Err(public::invalid(
            "Native presentation packet version or size is invalid",
        ));
    }
    let packet = unsafe { ptr::read_unaligned(packet) };
    if packet.reserved != [0; 2] {
        return Err(public::invalid(
            "Native presentation packet reserved fields are not zero",
        ));
    }
    Ok(PresentationBudget {
        max_elements: usize::try_from(packet.max_elements).unwrap_or(usize::MAX),
        max_text_bytes: usize::try_from(packet.max_text_bytes).unwrap_or(usize::MAX),
    })
}
#[derive(Serialize)]
pub(crate) struct Presented<'a> {
    #[serde(flatten)]
    pub(crate) rendered: compat::RenderResultTransport<'a>,
    pub(crate) presentation: &'a PresentationOutcome,
}
pub(crate) fn unavailable() -> PresentationOutcome {
    PresentationOutcome::Unavailable(PresentationUnavailable::UnsupportedDiagram)
}

/// # Safety
/// Source/options/packets/resources remain readable for this synchronous call.
#[no_mangle]
pub unsafe extern "C" fn pele_render_presentation_v11(
    source: *const u8,
    source_len: usize,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    packet: *const PelePresentationOptionsV11,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    public::bounded_call(|| {
        render_numbers::check_lengths(len, Some(source_len))?;
        let budget = unsafe { budget(packet) }?;
        let (options, limit) = unsafe { render_numbers::options(bytes, len, numbers, kind, bits) }?;
        let source =
            unsafe { compatibility_input(source, source_len, compat::MAX_PUBLIC_SOURCE_BYTES) }?;
        let (rendered, presentation) = unsafe {
            calendar::with_resources(resources, |host| {
                let mut capture = PresentationCapture::new(host, budget);
                let rendered = compat::Engine::with_builtins().render_source_bounded(
                    source,
                    &options,
                    limit,
                    &mut capture,
                )?;
                Ok::<_, compat::PeleError>((rendered, capture.take().unwrap_or_else(unavailable)))
            })
        }?;
        public::success(Presented {
            rendered: compat::render_result_transport(&rendered),
            presentation: &presentation,
        })
    })
}
/// # Safety
/// Document and all borrowed packets/resources must remain live during the call.
#[no_mangle]
pub unsafe extern "C" fn pele_render_document_presentation_v11(
    document: *const PeleResult,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    packet: *const PelePresentationOptionsV11,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let budget = unsafe { budget(packet) }?;
        let (options, limit) = unsafe { render_numbers::options(bytes, len, numbers, kind, bits) }?;
        let document = unsafe { document.as_ref() }
            .and_then(|value| value.document.as_ref())
            .ok_or_else(|| {
                public::invalid("render() takes diagram text or a model that parse() returned.")
            })?;
        let (rendered, presentation) = unsafe {
            calendar::with_resources(resources, |host| {
                let mut capture = PresentationCapture::new(host, budget);
                let rendered = compat::Engine::empty().render_document_bounded(
                    document,
                    &options,
                    limit,
                    &mut capture,
                )?;
                Ok::<_, compat::PeleError>((rendered, capture.take().unwrap_or_else(unavailable)))
            })
        }?;
        public::success(Presented {
            rendered: compat::render_result_transport(&rendered),
            presentation: &presentation,
        })
    })
}
/// # Safety
/// The mutable document is live, caller-confined and not freed during callbacks.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_presentation_v11(
    handle: *mut public::PelePublicDocument,
    expected_revision: u64,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    packet: *const PelePresentationOptionsV11,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let budget = unsafe { budget(packet) }?;
        let (options, limit) = unsafe { render_numbers::options(bytes, len, numbers, kind, bits) }?;
        unsafe {
            super::host_functions::with_resources(handle, resources, |host| {
                super::detached_render::render_presented(
                    handle,
                    expected_revision,
                    options,
                    limit,
                    budget,
                    host,
                )
            })
        }
    })
}
