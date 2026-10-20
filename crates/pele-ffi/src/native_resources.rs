//! Explicit, bounded native raster resolution. No network or filesystem implementation.
use super::presentation::{budget, unavailable, Presented};
use super::{
    calendar, compatibility_input, public, render_numbers, PeleBorrowedBytes, PeleHostResourcesV3,
    PelePresentationOptionsV11, PeleRenderNumbersV9, PeleResult,
};
use pele::compat::{
    self, EmbeddedImage, ErrorCode, ImageResource, PeleError, PresentationCapture, RenderResources,
    TextRequest, TrustedSvg,
};
use std::{collections::HashMap, ffi::c_void, ptr, slice};
pub type PeleImageLoaderV14 =
    unsafe extern "C" fn(*mut c_void, *const u8, usize, *mut PeleBorrowedBytes) -> u32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PeleNativeResourcesV14 {
    pub version: u32,
    pub struct_size: u32,
    pub context: *mut c_void,
    pub image_loader: Option<PeleImageLoaderV14>,
    pub max_image_bytes: u64,
    pub max_image_pixels: u64,
    pub max_total_bytes: u64,
    pub reserved: [u64; 2],
}
unsafe fn native_packet(
    packet: *const PeleNativeResourcesV14,
) -> Result<PeleNativeResourcesV14, PeleError> {
    if packet.is_null() {
        return Ok(PeleNativeResourcesV14 {
            version: 14,
            struct_size: 64,
            context: ptr::null_mut(),
            image_loader: None,
            max_image_bytes: compat::MAX_NATIVE_IMAGE_BYTES as u64,
            max_image_pixels: compat::MAX_NATIVE_IMAGE_PIXELS,
            max_total_bytes: compat::MAX_NATIVE_IMAGE_TOTAL_BYTES as u64,
            reserved: [0; 2],
        });
    }
    let h = packet.cast::<u32>();
    if unsafe { ptr::read_unaligned(h) } != 14
        || unsafe { ptr::read_unaligned(h.add(1)) } as usize
            != std::mem::size_of::<PeleNativeResourcesV14>()
    {
        return Err(public::invalid(
            "Native resource packet version or size is invalid",
        ));
    }
    let p = unsafe { ptr::read_unaligned(packet) };
    if p.reserved != [0; 2] {
        return Err(public::invalid(
            "Native resource packet reserved fields are not zero",
        ));
    }
    Ok(p)
}
struct Resources<'a> {
    host: &'a mut dyn RenderResources,
    packet: PeleNativeResourcesV14,
    images: HashMap<String, EmbeddedImage>,
    bytes: usize,
}
impl<'a> Resources<'a> {
    fn new(host: &'a mut dyn RenderResources, packet: PeleNativeResourcesV14) -> Self {
        Self {
            host,
            packet,
            images: HashMap::new(),
            bytes: 0,
        }
    }
}
impl RenderResources for Resources<'_> {
    fn host_function(
        &mut self,
        reference: u32,
    ) -> Result<std::rc::Rc<dyn compat::public::HostFunction>, compat::public::PublicError> {
        self.host.host_function(reference)
    }
    fn presentation_budget(&self) -> Option<compat::PresentationBudget> {
        self.host.presentation_budget()
    }
    fn capture_presentation(&mut self, outcome: compat::PresentationOutcome) {
        self.host.capture_presentation(outcome)
    }
    fn image(&mut self, uri: &str) -> Result<ImageResource, PeleError> {
        if let Some(image) = self.images.get(uri) {
            let total = self
                .packet
                .max_total_bytes
                .min(compat::MAX_NATIVE_IMAGE_TOTAL_BYTES as u64) as usize;
            if image.encoded_len() > total.saturating_sub(self.bytes) {
                return Err(PeleError::new(
                    "Native image exceeds encoded byte budget",
                    ErrorCode::Limit,
                ));
            }
            self.bytes += image.encoded_len();
            return Ok(ImageResource::Embedded(image.clone()));
        }
        if uri.len() > 4096 || self.images.len() >= compat::MAX_NATIVE_IMAGE_COUNT {
            return Err(PeleError::new(
                "Native image reference exceeds resource budget",
                ErrorCode::Limit,
            ));
        }
        let loader = self.packet.image_loader.ok_or_else(|| {
            public::invalid("Native image requires an approved host image loader")
        })?;
        let mut output = PeleBorrowedBytes {
            data: ptr::null(),
            len: 0,
        };
        if unsafe { loader(self.packet.context, uri.as_ptr(), uri.len(), &mut output) } != 0 {
            return Err(public::invalid("Native image loader failed"));
        }
        if output.data.is_null() {
            return Err(public::invalid(
                "Native image loader did not approve the image",
            ));
        }
        let cap = self
            .packet
            .max_image_bytes
            .min(compat::MAX_NATIVE_IMAGE_BYTES as u64) as usize;
        let total = self
            .packet
            .max_total_bytes
            .min(compat::MAX_NATIVE_IMAGE_TOTAL_BYTES as u64) as usize;
        if output.len > cap || output.len > total.saturating_sub(self.bytes) {
            return Err(PeleError::new(
                "Native image exceeds encoded byte budget",
                ErrorCode::Limit,
            ));
        }
        let image = EmbeddedImage::from_encoded(
            unsafe { slice::from_raw_parts(output.data, output.len) },
            cap,
            self.packet.max_image_pixels,
        )?;
        self.bytes += output.len;
        self.images.insert(uri.into(), image.clone());
        Ok(ImageResource::Embedded(image))
    }
    fn raw_inline_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.raw_inline_icon(name)
    }
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.host.text_width(r)
    }
    fn icon(&mut self, n: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.icon(n)
    }
    fn direct_icon(&mut self, n: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.direct_icon(n)
    }
    fn parse_date(&mut self, n: &str) -> Result<Option<f64>, PeleError> {
        self.host.parse_date(n)
    }
    fn calendar_fields(&mut self, n: f64) -> Result<Option<compat::CalendarFields>, PeleError> {
        self.host.calendar_fields(n)
    }
    fn calendar_instant(&mut self, n: compat::CivilDateTime) -> Result<Option<f64>, PeleError> {
        self.host.calendar_instant(n)
    }
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.direct_now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.now_millis()
    }
}
/// # Safety
/// Source/options/packets/resources remain readable for this synchronous call.
#[no_mangle]
pub unsafe extern "C" fn pele_render_native_v14(
    source: *const u8,
    source_len: usize,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    packet: *const PelePresentationOptionsV11,
    resources: *const PeleHostResourcesV3,
    native: *const PeleNativeResourcesV14,
) -> *mut PeleResult {
    public::bounded_call(|| {
        render_numbers::check_lengths(len, Some(source_len))?;
        let budget = unsafe { budget(packet) }?;
        let native = unsafe { native_packet(native) }?;
        let (options, limit) = unsafe { render_numbers::options(bytes, len, numbers, kind, bits) }?;
        let source =
            unsafe { compatibility_input(source, source_len, compat::MAX_PUBLIC_SOURCE_BYTES) }?;
        let (rendered, presentation) = unsafe {
            calendar::with_resources(resources, |host| {
                let mut images = Resources::new(host, native);
                let host: &mut dyn compat::RenderResources = &mut images;
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
pub unsafe extern "C" fn pele_render_document_native_v14(
    document: *const PeleResult,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    packet: *const PelePresentationOptionsV11,
    resources: *const PeleHostResourcesV3,
    native: *const PeleNativeResourcesV14,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let budget = unsafe { budget(packet) }?;
        let native = unsafe { native_packet(native) }?;
        let (options, limit) = unsafe { render_numbers::options(bytes, len, numbers, kind, bits) }?;
        let document = unsafe { document.as_ref() }
            .and_then(|value| value.document.as_ref())
            .ok_or_else(|| {
                public::invalid("render() takes diagram text or a model that parse() returned.")
            })?;
        let (rendered, presentation) = unsafe {
            calendar::with_resources(resources, |host| {
                let mut images = Resources::new(host, native);
                let host: &mut dyn compat::RenderResources = &mut images;
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
pub unsafe extern "C" fn pele_public_render_native_v14(
    handle: *mut public::PelePublicDocument,
    expected_revision: u64,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    packet: *const PelePresentationOptionsV11,
    resources: *const PeleHostResourcesV3,
    native: *const PeleNativeResourcesV14,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let budget = unsafe { budget(packet) }?;
        let native = unsafe { native_packet(native) }?;
        let (options, limit) = unsafe { render_numbers::options(bytes, len, numbers, kind, bits) }?;
        let mut document = unsafe { public::document(handle) }?;
        let (outcome, presentation) = unsafe {
            super::host_functions::with_resources(handle, resources, |host| {
                let mut images = Resources::new(host, native);
                let host: &mut dyn compat::RenderResources = &mut images;
                let mut capture = PresentationCapture::new(host, budget);
                let outcome = document.render_operation_bounded_read(
                    expected_revision,
                    &options,
                    limit,
                    &mut capture,
                )?;
                Ok::<_, compat::PeleError>((outcome, capture.take().unwrap_or_else(unavailable)))
            })
        }?;
        let wire = outcome.result.as_ref().map(|rendered| Presented {
            rendered: compat::render_result_transport(rendered),
            presentation: &presentation,
        });
        public::binary_metadata(outcome.revision, &outcome.graph, &wire)
    })
}
