//! Full Pele compatibility interface, under construction. Legacy v1 remains separate.
//! Detection recognizes all pinned families; capabilities never imply an absent renderer.
mod calendar;
pub use calendar::{fields_at_offset, instant_in_zone, CalendarFields, CivilDateTime};
pub mod diagrams;
mod edge_limit;
mod engine;
mod normalized_projection;
pub use edge_limit::{EdgeLimit, ParseLimits, MAX_PUBLIC_EDGES};
mod error;
mod id;
pub(crate) mod layout;
mod svg;
pub use svg::NativeTheme;
pub mod text;
pub use id::DiagramId;
mod native_image;
pub use native_image::*;
mod module;
mod presentation;
pub use presentation::*;
mod number_math;
mod provider;
pub use provider::{render_provider_bounded, ProviderInput};
mod output_limit;
pub use output_limit::{OutputLimit, MAX_PUBLIC_OUTPUT_BYTES};
pub mod public;
pub use engine::{Engine, ParsedDocument};
pub use module::*;
pub mod source;
mod source_limit;
pub use source_limit::{SourceLimit, MAX_PUBLIC_SOURCE_BYTES};
mod value;
pub use error::{ErrorCode, PeleError};
use serde::{Deserialize, Serialize};
pub use source::{DiagramType, PreparedSource, DIAGRAM_TYPES};
pub use value::{Object, Value};
pub const UPSTREAM_REVISION: &str = "458322a5b8abb1ae57ddf127c32017ab9fb14a92";
pub const DEFAULT_LIMIT: usize = 50_000;
pub const DEFAULT_OUTPUT_LIMIT: usize = 4_000_000;
pub const DEFAULT_MAX_EDGES: usize = 5_000;
/// A model's shared data can expand during JSON projection. Bound transport allocation.
pub const MAX_MODEL_JSON_BYTES: usize = 16 * 1024 * 1024;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareOptions {
    pub limit: usize,
    pub config: Object,
}
impl Default for PrepareOptions {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            config: Object::default(),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    #[serde(rename = "type")]
    pub diagram_type: Option<DiagramType>,
    pub registered: bool,
    pub prepared: PreparedSource,
    pub encoded_text: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    #[serde(rename = "type")]
    pub diagram_type: DiagramId,
    pub known: bool,
    pub registered: bool,
    pub syntax_parity: bool,
    pub semantic_parity: bool,
    pub render_parity: bool,
    pub host_parity: bool,
}
/// Version 2 preparation contract, shared by C and WASM. Does not promise rendering parity.
pub fn inspect_envelope(source: &str, options_json: &str) -> (u32, String) {
    let result = if options_json.trim().is_empty() {
        Ok(PrepareOptions::default())
    } else {
        serde_json::from_str::<PrepareOptions>(options_json).map_err(|e| {
            PeleError::new(
                format!("Invalid preparation options: {e}"),
                ErrorCode::Semantic,
            )
        })
    }
    .and_then(|options| Engine::empty().inspect(source, &options));
    match result {
        Ok(result) => (0, success_envelope(&result)),
        Err(error) => (
            if error.code == ErrorCode::Limit { 2 } else { 1 },
            error_envelope(&error),
        ),
    }
}
#[derive(Serialize)]
struct SuccessEnvelope<'a, T> {
    version: u32,
    ok: bool,
    result: &'a T,
}
#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    version: u32,
    ok: bool,
    error: &'a PeleError,
}
pub fn success_envelope<T: Serialize>(result: &T) -> String {
    serde_json::to_string(&SuccessEnvelope {
        version: 2,
        ok: true,
        result,
    })
    .expect("serializable engine value")
}
/// V2 JSON projections use null for non-finite numbers. Render transport adds
/// exact IEEE bits only where JSON loses a dimension; host adapters remove this metadata.
/// Borrowed nested render transport; JSON-inexpressible dimensions retain bits.
#[derive(Serialize)]
pub struct RenderResultTransport<'a> {
    #[serde(flatten)]
    result: &'a RenderResult,
    #[serde(
        rename = "$numbers",
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    numbers: std::collections::BTreeMap<&'static str, String>,
}
pub fn render_result_transport(result: &RenderResult) -> RenderResultTransport<'_> {
    let mut numbers = std::collections::BTreeMap::new();
    for (key, value) in [
        ("width", result.rendered.width),
        ("height", result.rendered.height),
    ] {
        if !value.is_finite() || (value == 0.0 && value.is_sign_negative()) {
            numbers.insert(key, format!("{:016x}", value.to_bits()));
        }
    }
    RenderResultTransport { result, numbers }
}
pub fn render_result_envelope(result: &RenderResult) -> String {
    success_envelope(&render_result_transport(result))
}
pub fn error_envelope(error: &PeleError) -> String {
    serde_json::to_string(&ErrorEnvelope {
        version: 2,
        ok: false,
        error,
    })
    .expect("serializable error")
}
pub fn capabilities_json() -> String {
    success_envelope(&Engine::empty().capabilities())
}

pub fn parse_options(json: &str) -> Result<ParseOptions, PeleError> {
    if json.len() > 1024 * 1024 {
        return Err(PeleError::new(
            "Transport options exceed 1 MiB",
            ErrorCode::Limit,
        ));
    }
    if json.trim().is_empty() {
        return Ok(ParseOptions::default());
    }
    serde_json::from_str(json)
        .map_err(|e| PeleError::new(format!("Invalid parse options: {e}"), ErrorCode::Semantic))
}
pub fn render_options(json: &str) -> Result<RenderOptions, PeleError> {
    if json.len() > 1024 * 1024 {
        return Err(PeleError::new(
            "Transport options exceed 1 MiB",
            ErrorCode::Limit,
        ));
    }
    if json.trim().is_empty() {
        return Ok(RenderOptions::default());
    }
    serde_json::from_str(json)
        .map_err(|e| PeleError::new(format!("Invalid render options: {e}"), ErrorCode::Semantic))
}
pub fn document_envelope(document: &ParsedDocument) -> Result<String, PeleError> {
    #[derive(Serialize)]
    struct Projection<'a> {
        #[serde(rename = "type")]
        kind: &'a DiagramId,
        model: Value,
    }
    Ok(success_envelope(&Projection {
        kind: document.diagram_type(),
        model: document.model_view()?,
    }))
}
pub fn render_envelope(source: &str, options_json: &str) -> (u32, String) {
    let result = render_options(options_json).and_then(|options| {
        Engine::with_builtins().render_source(source, &options, &mut text::MetricsResources)
    });
    match result {
        Ok(result) => (0, render_result_envelope(&result)),
        Err(error) => (
            if error.code == ErrorCode::Limit { 2 } else { 1 },
            error_envelope(&error),
        ),
    }
}
pub fn builtin_capabilities_json() -> String {
    success_envelope(&Engine::with_builtins().capabilities())
}

/// Exact legacy envelope admission for adapters handing off an already normalized
/// projection. Counts the same serializer without retaining a second JSON buffer.
#[doc(hidden)]
pub fn success_envelope_len<T: Serialize>(result: &T) -> usize {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .expect("bounded projection size");
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0);
    serde_json::to_writer(
        &mut count,
        &SuccessEnvelope {
            version: 2,
            ok: true,
            result,
        },
    )
    .expect("serializable engine value");
    count.0
}

#[cfg(feature = "web-host-er")]
#[doc(hidden)]
pub mod host_er;

#[cfg(feature = "web-host-sankey")]
#[doc(hidden)]
pub mod host_sankey;
