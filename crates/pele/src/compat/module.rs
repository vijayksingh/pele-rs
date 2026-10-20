//! Typed extension boundary. A module owns its model; the engine owns preparation/provenance.
use super::{
    DiagramId, EdgeLimit, Object, OutputLimit, PeleError, PrepareOptions, DEFAULT_MAX_EDGES,
    DEFAULT_OUTPUT_LIMIT,
};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct ModuleDescriptor {
    pub id: DiagramId,
    pub section: Option<String>,
    pub keep_comments: bool,
}
impl ModuleDescriptor {
    pub fn new(id: impl Into<DiagramId>) -> Self {
        Self {
            id: id.into(),
            section: None,
            keep_comments: false,
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParseOptions {
    #[serde(flatten)]
    pub preparation: PrepareOptions,
    pub max_edges: usize,
}
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            preparation: PrepareOptions::default(),
            max_edges: DEFAULT_MAX_EDGES,
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderSettings {
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    pub font_size: Option<f64>,
    pub id_prefix: Option<String>,
    pub responsive: Option<bool>,
    pub max_width: Option<f64>,
    pub auto_direction: Option<bool>,
    pub direction_breakpoint: Option<f64>,
    pub padding: Option<f64>,
    pub output_limit: Option<usize>,
    pub link_schemes: Option<Vec<String>>,
    pub image_schemes: Option<Vec<String>>,
    pub links: Option<bool>,
    pub images: Option<bool>,
    pub link_rel: Option<String>,
    pub now: Option<f64>,
    /// Rust extension: resolve CSS tokens for native SVG decoders.
    pub native_theme: Option<super::NativeTheme>,
}
impl RenderSettings {
    pub fn output_limit(&self) -> usize {
        self.output_limit.unwrap_or(DEFAULT_OUTPUT_LIMIT)
    }
}
/// Combined public-call options; modules receive render-only settings.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderOptions {
    #[serde(flatten)]
    pub parse: ParseOptions,
    #[serde(flatten)]
    pub render: RenderSettings,
}
impl Serialize for ParseOptions {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = s.serialize_map(Some(3))?;
        m.serialize_entry("limit", &self.preparation.limit)?;
        m.serialize_entry("config", &self.preparation.config)?;
        m.serialize_entry("maxEdges", &self.max_edges)?;
        m.end()
    }
}
/// Prepared/encoded source ending in a newline. Source positions remain the module's responsibility.
pub struct ParseContext<'a> {
    pub source: &'a str,
    pub config: &'a Object,
    pub title: Option<&'a str>,
    pub line_offset: usize,
    pub max_edges: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LinkInfo {
    pub id: String,
    pub href: String,
    pub internal: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Rendered {
    pub svg: String,
    #[serde(serialize_with = "js_number")]
    pub width: f64,
    #[serde(serialize_with = "js_number")]
    pub height: f64,
    pub links: Vec<LinkInfo>,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RenderResult {
    #[serde(rename = "type")]
    pub diagram_type: DiagramId,
    #[serde(flatten)]
    pub rendered: Rendered,
}

/// Text style bits match Pele: bold=1, italic=2, mono=4.
pub struct TextRequest<'a> {
    pub text: &'a str,
    pub size: f64,
    pub style: u8,
    pub font_family: Option<&'a str>,
    pub font_family_mono: Option<&'a str>,
}
/// Host-supplied markup is trusted executable-adjacent input. Never construct this from diagram source.
#[derive(Clone, Debug)]
pub struct TrustedSvg(String);
impl TrustedSvg {
    pub fn from_trusted_markup(markup: String) -> Self {
        Self(markup)
    }
    pub fn markup(&self) -> &str {
        &self.0
    }
}
/// A failed generic JavaScript operation, independent of diagram policy.
/// Browser hosts derive engine-native messages; portable hosts retain the
/// established error wire and message without executing any JavaScript.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum JavaScriptFailure {
    ReadonlyArrayPushLength,
    ReadonlyArrayPushIndex(u32),
}
impl JavaScriptFailure {
    pub fn native_error(self) -> super::public::PublicError {
        let key = match self {
            Self::ReadonlyArrayPushLength => "length".to_owned(),
            Self::ReadonlyArrayPushIndex(index) => index.to_string(),
        };
        super::public::PublicError::JavaScript {
            name: super::public::JavaScriptErrorKind::TypeError,
            message: super::public::JsString::from(
                format!("Cannot assign to read only property '{key}' of object '[object Array]'")
                    .as_str(),
            ),
        }
    }
}
/// Per-render host resources. No global resolver, filesystem, network or clock in the core.
pub trait RenderResources {
    /// Resolve a failed intrinsic operation, never replay a caller mutation.
    fn javascript_failure(&mut self, operation: JavaScriptFailure) -> super::public::PublicError {
        operation.native_error()
    }

    /// Resolve an owner-retained callable before borrowing the active arena.
    fn host_function(
        &mut self,
        _reference: u32,
    ) -> Result<std::rc::Rc<dyn super::public::HostFunction>, super::public::PublicError> {
        Err(PeleError::new(
            "Unbound host function reference",
            super::ErrorCode::Semantic,
        )
        .into())
    }

    /// Opt-in presentation capture. Ordinary hosts return None and allocate no frames.
    fn presentation_budget(&self) -> Option<super::PresentationBudget> {
        None
    }
    /// Modules submit provisional data; Engine intercepts and publishes only the
    /// selected successful frame after output policy. Direct module calls have no
    /// engine transaction and must not be used as a committed presentation API.
    fn capture_presentation(&mut self, _outcome: super::PresentationOutcome) {}

    /// Resolve an image only after the core URI policy accepts it. Portable
    /// hosts preserve URI output; native hosts return approved embedded rasters.
    fn image(&mut self, _approved_uri: &str) -> Result<super::ImageResource, PeleError> {
        Ok(super::ImageResource::Passthrough)
    }
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError>;
    fn icon(&mut self, _name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        Ok(None)
    }
    /// An already-established outer core render may supply its inline icon
    /// lookup to a nested raw diagram call. Standalone raw entries have none.
    fn raw_inline_icon(&mut self, _name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        Ok(None)
    }
    /// Direct host resolver: unlike inline-label icons, repeated calls are not cached.
    fn direct_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.icon(name)
    }
    /// Host date-string parser. The portable default provides deterministic UTC
    /// formats; browser adapters delegate to their Date implementation.
    fn parse_date(&mut self, input: &str) -> Result<Option<f64>, PeleError> {
        match super::calendar::date_parser::parse(input) {
            Some(super::calendar::date_parser::ParsedDate::Instant(value)) => Ok(Some(value)),
            Some(super::calendar::date_parser::ParsedDate::Local(civil)) => {
                self.calendar_instant(civil)
            }
            None => Ok(None),
        }
    }
    /// Local calendar supplied by the host. The portable default uses UTC.
    fn calendar_fields(
        &mut self,
        instant_ms: f64,
    ) -> Result<Option<super::CalendarFields>, PeleError> {
        Ok(super::calendar::utc_fields(instant_ms))
    }
    /// Resolve normalized civil time. Hosts follow JavaScript Date: earliest
    /// instant in a fold, and shift forward by the gap for nonexistent time.
    fn calendar_instant(&mut self, civil: super::CivilDateTime) -> Result<Option<f64>, PeleError> {
        Ok(super::calendar::utc_instant(civil))
    }
    /// Direct host clock for an explicit captured public method call. Unlike
    /// render-session time, this is neither option-seeded nor cached.
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        Err(PeleError::new(
            "Host did not supply a clock",
            super::ErrorCode::Semantic,
        ))
    }
}
/// Built-ins and consumer extensions implement the same interface with concrete typed models.
/// Modules are trusted Rust code; registering one grants responsibility for its emitted SVG.
pub trait DiagramModule: Send + Sync + 'static {
    type Model: Serialize + Send + Sync + 'static;
    /// Guard recursive/shared model expansion before JSON introspection. This does
    /// not reject the typed model or prevent rendering without that projection.
    fn validate_model_projection(&self, _model: &Self::Model) -> Result<(), PeleError> {
        Ok(())
    }
    /// Read-only model operation. Existing documents retain their exact handler.
    fn query(
        &self,
        _model: &Self::Model,
        operation: &str,
        _arguments: &super::Value,
    ) -> Result<super::Value, PeleError> {
        Err(PeleError::new(
            format!("Unknown model query: {operation}"),
            super::ErrorCode::Semantic,
        ))
    }
    fn descriptor(&self) -> ModuleDescriptor;
    fn parse(&self, context: ParseContext<'_>) -> Result<Self::Model, PeleError>;
    /// Exact edge policy. Providers that enforce context.max_edges privately
    /// must override this hook; the default retains their previous parse seam.
    fn parse_with_edge_limit(
        &self,
        context: ParseContext<'_>,
        _limit: EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        self.parse(context)
    }
    /// Prepared low-level parsing keeps raw policy presence distinct from core defaults.
    fn parse_prepared(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        self.parse_with_edge_limit(context, limit)
    }
    fn render(
        &self,
        model: &Self::Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError>;
    /// Raw low-level diagram rendering. Core postprocessing is not applied.
    /// Override when a module privately enforces a core output threshold.
    fn render_raw(
        &self,
        model: &Self::Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(model, config, options, resources)
    }
    /// Exact post-render output policy. The engine owns the final comparison.
    /// Providers that privately enforce the legacy integer output_limit must
    /// override this hook to avoid rejecting before the exact comparison.
    fn render_with_output_limit(
        &self,
        model: &Self::Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
        _limit: OutputLimit,
    ) -> Result<Rendered, PeleError> {
        self.render(model, config, options, resources)
    }
}

fn js_number<S: Serializer>(value: &f64, s: S) -> Result<S::Ok, S::Error> {
    super::Value::Number(*value).serialize(s)
}
