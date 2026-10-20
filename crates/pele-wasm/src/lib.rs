#[cfg(feature = "captured-sankey")]
mod captured_sankey;
#[cfg(feature = "captured-sankey")]
mod captured_store;
mod foreign_provider;
mod host_construction;
mod host_er;
mod host_functions;
#[cfg(feature = "native-canonical-sankey")]
mod native_canonical;
mod parse_limits;
mod prepared_entries;
mod public_remaining_er;
mod public_remaining_info;
mod render_numbers;
mod render_packet;
mod source_layout;
mod wasm_registry;
use wasm_bindgen::prelude::*;

/// Full-library source preparation.
#[wasm_bindgen]
pub fn inspect(source: &str, options_json: &str) -> String {
    pele::compat::inspect_envelope(source, options_json).1
}
#[wasm_bindgen]
pub fn capabilities() -> String {
    pele::compat::capabilities_json()
}

#[wasm_bindgen(js_name=ParsedDocument)]
pub struct ParsedDocument {
    inner: pele::compat::ParsedDocument,
}
#[wasm_bindgen]
impl ParsedDocument {
    /// Convert a supported typed model without parsing its source a second time.
    pub fn to_public(&self) -> Result<Option<MutableDocument>, JsValue> {
        use pele::compat::{
            diagrams::{
                agentflow::{AgentFlow, AgentflowModel},
                class_diagram::{Class, ClassModel},
                event_modeling::{EventModeling, EventModelingModel},
                git_graph::{GitGraph, GitModel},
                ishikawa::{Ishikawa, IshikawaModel},
                state::{State, StateModel},
                tree_view::{TreeView, TreeViewModel},
            },
            diagrams::{
                architecture::{Architecture, ArchitectureModel},
                er::{Er, ErModel},
                flowchart::{Flowchart, FlowchartModel},
                info::{Info, InfoModel},
                journey::{Journey, JourneyModel},
                kanban::{Kanban, KanbanModel},
                mindmap::{Mindmap, MindmapModel},
                packet::{Packet, PacketModel},
                pie::{Pie, PieModel},
                quadrant_chart::{QuadrantChart, QuadrantModel},
                radar::{Radar, RadarModel},
                sankey::{Sankey, SankeyModel},
                swimlane::Swimlane,
                timeline::{Timeline, TimelineModel},
                treemap::{Treemap, TreemapModel},
                xychart::{XyChart, XyChartModel},
            },
            public::PublicDocument,
        };
        // Swimlane deliberately uses FlowchartModel but retains its lane renderer.
        let document = if self.inner.diagram_type().as_str() == "swimlane" {
            let model = self
                .inner
                .model::<FlowchartModel>()
                .ok_or_else(|| JsValue::from_str("Swimlane retained an incompatible model"))?;
            PublicDocument::from_module(Swimlane, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<FlowchartModel>() {
            PublicDocument::from_module(Flowchart, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<KanbanModel>() {
            PublicDocument::from_module(Kanban, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<MindmapModel>() {
            PublicDocument::from_module(Mindmap, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<TimelineModel>() {
            PublicDocument::from_module(Timeline, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<ErModel>() {
            PublicDocument::from_module(Er, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<ArchitectureModel>() {
            PublicDocument::from_module(Architecture, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<PieModel>() {
            PublicDocument::from_module(Pie, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<InfoModel>() {
            PublicDocument::from_module(Info, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<SankeyModel>() {
            PublicDocument::from_module(Sankey, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<PacketModel>() {
            PublicDocument::from_module(Packet, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<RadarModel>() {
            PublicDocument::from_module(Radar, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<QuadrantModel>() {
            PublicDocument::from_module(QuadrantChart, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<TreemapModel>() {
            PublicDocument::from_module(Treemap, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<JourneyModel>() {
            PublicDocument::from_module(Journey, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<XyChartModel>() {
            PublicDocument::from_module(XyChart, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<AgentflowModel>() {
            PublicDocument::from_module(AgentFlow, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<ClassModel>() {
            PublicDocument::from_module(Class, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<EventModelingModel>() {
            PublicDocument::from_module(EventModeling, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<GitModel>() {
            PublicDocument::from_module(GitGraph, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<IshikawaModel>() {
            PublicDocument::from_module(Ishikawa, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<StateModel>() {
            PublicDocument::from_module(State, model, self.inner.config().clone())
        } else if let Some(model) = self.inner.model::<TreeViewModel>() {
            PublicDocument::from_module(TreeView, model, self.inner.config().clone())
        } else if let Some(document) = public_remaining_er::convert(&self.inner)
            .or_else(|| public_remaining_info::convert(&self.inner))
        {
            document
        } else {
            return Ok(None);
        };
        document
            .map(|inner| Some(MutableDocument { inner }))
            .map_err(|e| JsValue::from_str(&pele::compat::error_envelope(&e)))
    }
    pub fn query(&self, operation: &str, arguments_json: &str) -> String {
        let run = || -> Result<pele::compat::Value, pele::compat::PeleError> {
            use pele::compat::{ErrorCode, PeleError};
            if operation.len() > 4096 || arguments_json.len() > 1024 * 1024 {
                return Err(PeleError::new(
                    "Model query input exceeds budget",
                    ErrorCode::Limit,
                ));
            }
            let arguments = serde_json::from_str(arguments_json).map_err(|_| {
                PeleError::new("Invalid model query arguments", ErrorCode::Semantic)
            })?;
            self.inner.query(operation, &arguments)
        };
        match run() {
            Ok(value) => pele::compat::success_envelope(&value),
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    pub fn model(&self) -> String {
        match pele::compat::document_envelope(&self.inner) {
            Ok(s) => s,
            Err(e) => pele::compat::error_envelope(&e),
        }
    }
    pub fn render(&self, options_json: &str) -> String {
        match pele::compat::render_options(options_json).and_then(|options| {
            pele::compat::Engine::empty().render_document(
                &self.inner,
                &options,
                &mut pele::compat::text::MetricsResources,
            )
        }) {
            Ok(rendered) => pele::compat::render_result_envelope(&rendered),
            Err(e) => pele::compat::error_envelope(&e),
        }
    }
}

/// Public arenas are flat but can still serialize many properties. Bound bytes
/// while writing, before constructing a transport String or intermediary Value.
fn public_result<T: serde::Serialize>(result: &T) -> String {
    use std::io::Write;
    struct Bounded(Vec<u8>);
    impl Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0.len().saturating_add(bytes.len()) > 64 * 1024 * 1024 {
                return Err(std::io::Error::other("Public JSON result budget"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[derive(serde::Serialize)]
    struct Envelope<'a, T> {
        version: u8,
        ok: bool,
        result: &'a T,
    }
    let mut output = Bounded(Vec::new());
    match serde_json::to_writer(
        &mut output,
        &Envelope {
            version: 2,
            ok: true,
            result,
        },
    ) {
        Ok(()) => String::from_utf8(output.0).expect("serde JSON is UTF-8"),
        Err(_) => pele::compat::error_envelope(&pele::compat::PeleError::new(
            "Public result JSON exceeds 64 MiB transport budget",
            pele::compat::ErrorCode::Limit,
        )),
    }
}
#[wasm_bindgen(js_name=MutableDocument)]
pub struct MutableDocument {
    inner: pele::compat::public::PublicDocument,
}
#[wasm_bindgen]
impl MutableDocument {
    pub fn unchanged_public_graph_is_noop(&self) -> bool {
        self.inner.unchanged_public_graph_is_noop()
    }
    pub fn schema(&self) -> String {
        public_result(&self.inner.schema_view())
    }
    /// Opaque identity of this document's immutable captured schema view.
    /// These are WASM linear-memory offsets, never host process addresses.
    /// Consumers must scope keys to their renderer and initialized WASM module.
    #[cfg(target_arch = "wasm32")]
    pub fn schema_cache_key(&self) -> String {
        let view = self.inner.schema_view();
        format!(
            "1:{:x}:{:x}:{}:{:x}:{}",
            std::ptr::from_ref(view.model) as usize,
            view.shared_functions.as_ptr() as usize,
            view.shared_functions.len(),
            view.function_shapes.as_ptr() as usize,
            view.function_shapes.len(),
        )
    }
    /// Dispatch edits through the retained native module and return post-call
    /// graph even when the operation throws. Revision uses decimal text in JS.
    pub fn invoke(&mut self, revision: &str, call_json: &str) -> String {
        let mut run = || -> Result<_, pele::compat::PeleError> {
            use pele::compat::{public::MethodCall, ErrorCode, PeleError};
            if call_json.len() > 64 * 1024 * 1024 {
                return Err(PeleError::new(
                    "Public method JSON exceeds 64 MiB transport budget",
                    ErrorCode::Limit,
                ));
            }
            let revision = revision.parse::<u64>().map_err(|_| {
                PeleError::new("Invalid public model revision", ErrorCode::Semantic)
            })?;
            let call: MethodCall = serde_json::from_str(call_json).map_err(|_| {
                PeleError::new("Invalid public model method call", ErrorCode::Semantic)
            })?;
            self.inner
                .invoke(revision, call, &mut pele::compat::text::MetricsResources)
        };
        match run() {
            Ok(outcome) => {
                #[derive(serde::Serialize)]
                struct Outcome {
                    revision: String,
                    graph: pele::compat::public::ModelGraph,
                    result: Result<pele::compat::public::Atom, pele::compat::public::PublicError>,
                }
                public_result(&Outcome {
                    revision: outcome.revision.to_string(),
                    graph: outcome.graph,
                    result: outcome.result,
                })
            }
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    pub fn snapshot(&self) -> String {
        match self.inner.snapshot() {
            Ok(graph) => {
                #[derive(serde::Serialize)]
                struct Snapshot {
                    graph: pele::compat::public::ModelGraph,
                    revision: String,
                }
                public_result(&Snapshot {
                    graph,
                    revision: self.inner.revision().to_string(),
                })
            }
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    pub fn apply(&mut self, revision: &str, graph_json: &str) -> String {
        let mut apply = || -> Result<u64, pele::compat::PeleError> {
            use pele::compat::{public::ModelGraph, ErrorCode, PeleError};
            if graph_json.len() > 64 * 1024 * 1024 {
                return Err(PeleError::new(
                    "Public graph JSON exceeds 64 MiB transport budget",
                    ErrorCode::Limit,
                ));
            }
            let revision = revision.parse::<u64>().map_err(|_| {
                PeleError::new("Invalid public model revision", ErrorCode::Semantic)
            })?;
            let graph: ModelGraph = serde_json::from_str(graph_json)
                .map_err(|_| PeleError::new("Invalid public model graph", ErrorCode::Semantic))?;
            self.inner.apply(revision, graph)?;
            Ok(self.inner.revision())
        };
        match apply() {
            Ok(revision) => {
                serde_json::json!({"version":2,"ok":true,"result":revision.to_string()}).to_string()
            }
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    /// Schedules/mutates native state and publishes its post-render graph even
    /// when rendering throws. Revision stays decimal text across JavaScript.
    pub fn render_operation_with_resources(
        &mut self,
        revision: &str,
        options_json: &str,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
    ) -> String {
        let run = || -> Result<_, pele::compat::PeleError> {
            let revision = revision.parse::<u64>().map_err(|_| {
                pele::compat::PeleError::new(
                    "Invalid public model revision",
                    pele::compat::ErrorCode::Semantic,
                )
            })?;
            let options = pele::compat::render_options(options_json)?;
            self.inner
                .render_operation(revision, &options, &mut BrowserResources { width, icons })
        };
        match run() {
            Ok(outcome) => {
                #[derive(serde::Serialize)]
                struct Outcome<'a> {
                    revision: String,
                    graph: &'a pele::compat::public::ModelGraph,
                    result: Result<
                        pele::compat::RenderResultTransport<'a>,
                        &'a pele::compat::public::PublicError,
                    >,
                }
                public_result(&Outcome {
                    revision: outcome.revision.to_string(),
                    graph: &outcome.graph,
                    result: outcome
                        .result
                        .as_ref()
                        .map(pele::compat::render_result_transport),
                })
            }
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    /// Optional owned source-layout publication; legacy providers are not exported.
    pub fn snapshot_source_layout(&self) -> Result<JsValue, JsValue> {
        source_layout::snapshot(&self.inner)
    }
    pub fn snapshot_binary(&self) -> Result<Vec<u8>, JsValue> {
        let run = || -> Result<Vec<u8>, pele::compat::PeleError> {
            let graph = self.inner.snapshot_view()?;
            let metadata =
                serde_json::to_vec(&serde_json::json!({"type":self.inner.diagram_type()}))
                    .expect("snapshot metadata");
            pele::compat::public::encode_binary_exchange(
                self.inner.revision(),
                &graph,
                &metadata,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(|error| JsValue::from_str(&pele::compat::error_envelope(&error)))
    }
    pub fn source_ingress_identity(&mut self) -> Option<String> {
        self.inner
            .source_ingress_identity()
            .map(|id| id.to_string())
    }
    pub fn apply_source_input(&mut self, bytes: &[u8]) -> String {
        match self.inner.apply_source_input(bytes) {
            Ok(()) => public_result(&self.inner.revision().to_string()),
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    pub fn apply_binary(&mut self, bytes: &[u8]) -> String {
        let mut run = || -> Result<(), pele::compat::PeleError> {
            let exchange = pele::compat::public::decode_binary_exchange(
                bytes,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )?;
            if !exchange.metadata.is_empty() {
                return Err(pele::compat::PeleError::new(
                    "Apply binary metadata must be empty",
                    pele::compat::ErrorCode::Semantic,
                ));
            }
            self.inner.apply(exchange.revision, exchange.graph)
        };
        match run() {
            Ok(()) => public_result(&self.inner.revision().to_string()),
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    pub fn invoke_binary(&mut self, revision: &str, call_json: &str) -> Result<Vec<u8>, JsValue> {
        let mut run = || -> Result<Vec<u8>, pele::compat::PeleError> {
            use pele::compat::{ErrorCode, PeleError};
            if call_json.len() > 64 * 1024 * 1024 {
                return Err(PeleError::new(
                    "Public method JSON exceeds 64 MiB transport budget",
                    ErrorCode::Limit,
                ));
            }
            let revision = revision.parse::<u64>().map_err(|_| {
                PeleError::new("Invalid public model revision", ErrorCode::Semantic)
            })?;
            let call = serde_json::from_str(call_json).map_err(|_| {
                PeleError::new("Invalid public model method call", ErrorCode::Semantic)
            })?;
            let outcome = self.inner.invoke(
                revision,
                call,
                &mut BrowserResources {
                    width: None,
                    icons: None,
                },
            )?;
            let metadata = serde_json::to_vec(&outcome.result).expect("operation metadata");
            pele::compat::public::encode_binary_exchange(
                outcome.revision,
                &outcome.graph,
                &metadata,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(|error| JsValue::from_str(&pele::compat::error_envelope(&error)))
    }
    pub fn render_operation_binary(
        &mut self,
        revision: &str,
        options_json: &str,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
    ) -> Result<Vec<u8>, JsValue> {
        let run = || -> Result<Vec<u8>, pele::compat::PeleError> {
            let revision = revision.parse::<u64>().map_err(|_| {
                pele::compat::PeleError::new(
                    "Invalid public model revision",
                    pele::compat::ErrorCode::Semantic,
                )
            })?;
            let options = pele::compat::render_options(options_json)?;
            let outcome = self.inner.render_operation(
                revision,
                &options,
                &mut BrowserResources { width, icons },
            )?;
            let metadata = serde_json::to_vec(
                &outcome
                    .result
                    .as_ref()
                    .map(pele::compat::render_result_transport),
            )
            .expect("render metadata");
            pele::compat::public::encode_binary_exchange(
                outcome.revision,
                &outcome.graph,
                &metadata,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(|error| JsValue::from_str(&pele::compat::error_envelope(&error)))
    }
    pub fn render_with_resources(
        &mut self,
        options_json: &str,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
    ) -> String {
        match pele::compat::render_options(options_json).and_then(|options| {
            self.inner
                .render(&options, &mut BrowserResources { width, icons })
        }) {
            Ok(result) => pele::compat::render_result_envelope(&result),
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
}
#[wasm_bindgen]
pub fn parse_v2(source: &str, options_json: &str) -> Result<ParsedDocument, JsValue> {
    pele::compat::parse_options(options_json)
        .and_then(|options| pele::compat::Engine::with_builtins().parse(source, &options))
        .map(|inner| ParsedDocument { inner })
        .map_err(|e| JsValue::from_str(&pele::compat::error_envelope(&e)))
}
#[wasm_bindgen]
pub fn render_v2(source: &str, options_json: &str) -> String {
    pele::compat::render_envelope(source, options_json).1
}
#[wasm_bindgen]
pub fn builtin_capabilities() -> String {
    pele::compat::builtin_capabilities_json()
}

struct BrowserResources {
    width: Option<js_sys::Function>,
    icons: Option<js_sys::Function>,
}
fn callback_error(error: JsValue) -> pele::compat::PeleError {
    let message = js_sys::Reflect::get(&error, &JsValue::from_str("message"))
        .ok()
        .and_then(|v| v.as_string())
        .or_else(|| error.as_string())
        .unwrap_or_else(|| "Host callback failed".into());
    pele::compat::PeleError::new(message, pele::compat::ErrorCode::Semantic)
}
impl pele::compat::RenderResources for BrowserResources {
    fn text_width(
        &mut self,
        r: pele::compat::TextRequest<'_>,
    ) -> Result<f64, pele::compat::PeleError> {
        if let Some(callback) = &self.width {
            let args = js_sys::Array::new();
            args.push(&r.text.into());
            args.push(&r.size.into());
            args.push(&(r.style as u32).into());
            args.push(&r.font_family.map_or(JsValue::UNDEFINED, JsValue::from_str));
            args.push(
                &r.font_family_mono
                    .map_or(JsValue::UNDEFINED, JsValue::from_str),
            );
            callback
                .apply(&JsValue::UNDEFINED, &args)
                .map_err(callback_error)?
                .as_f64()
                .ok_or_else(|| callback_error("Text measurer must return a number".into()))
        } else {
            Ok(pele::compat::text::metrics_width(r.text, r.size, r.style))
        }
    }
    fn icon(
        &mut self,
        name: &str,
    ) -> Result<Option<pele::compat::TrustedSvg>, pele::compat::PeleError> {
        let Some(callback) = &self.icons else {
            return Ok(None);
        };
        let value = callback
            .call1(&JsValue::UNDEFINED, &name.into())
            .map_err(callback_error)?;
        if value.is_null() || value.is_undefined() {
            return Ok(None);
        }
        value
            .as_string()
            .map(|s| Some(pele::compat::TrustedSvg::from_trusted_markup(s)))
            .ok_or_else(|| {
                callback_error("Icon resolver must return trusted SVG markup or null".into())
            })
    }
    fn parse_date(&mut self, input: &str) -> Result<Option<f64>, pele::compat::PeleError> {
        let instant = js_sys::Date::parse(input);
        Ok(instant.is_finite().then_some(instant))
    }
    fn calendar_fields(
        &mut self,
        instant_ms: f64,
    ) -> Result<Option<pele::compat::CalendarFields>, pele::compat::PeleError> {
        let date = js_sys::Date::new(&instant_ms.into());
        if !date.get_time().is_finite() {
            return Ok(None);
        }
        Ok(Some(pele::compat::CalendarFields {
            civil: pele::compat::CivilDateTime {
                year: date.get_full_year() as i32,
                month: (date.get_month() + 1) as u8,
                day: date.get_date() as u8,
                hour: date.get_hours() as u8,
                minute: date.get_minutes() as u8,
                second: date.get_seconds() as u8,
                millisecond: date.get_milliseconds() as u16,
            },
            weekday: date.get_day() as u8,
            utc_offset_minutes: -date.get_timezone_offset() as i32,
        }))
    }
    fn calendar_instant(
        &mut self,
        c: pele::compat::CivilDateTime,
    ) -> Result<Option<f64>, pele::compat::PeleError> {
        // Construct with a full 400-year cycle for years 0..99, then restore the
        // exact year; Date's historical constructor adjustment belongs to core.
        let year = if (0..100).contains(&c.year) {
            c.year + 400
        } else {
            c.year
        };
        let args = js_sys::Array::new();
        for value in [
            year,
            i32::from(c.month) - 1,
            i32::from(c.day),
            i32::from(c.hour),
            i32::from(c.minute),
            i32::from(c.second),
            i32::from(c.millisecond),
        ] {
            args.push(&JsValue::from_f64(f64::from(value)));
        }
        let constructor = js_sys::Reflect::get(&js_sys::global(), &"Date".into())
            .map_err(callback_error)?
            .unchecked_into::<js_sys::Function>();
        let date = js_sys::Reflect::construct(&constructor, &args)
            .map_err(callback_error)?
            .unchecked_into::<js_sys::Date>();
        if (0..100).contains(&c.year) {
            date.set_full_year(c.year as u32);
        }
        let instant = date.get_time();
        Ok(instant.is_finite().then_some(instant))
    }
    fn now_millis(&mut self) -> Result<f64, pele::compat::PeleError> {
        Ok(js_sys::Date::now())
    }
}
#[wasm_bindgen]
impl ParsedDocument {
    pub fn render_with_resources(
        &self,
        options_json: &str,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
    ) -> String {
        match pele::compat::render_options(options_json).and_then(|options| {
            pele::compat::Engine::empty().render_document(
                &self.inner,
                &options,
                &mut BrowserResources { width, icons },
            )
        }) {
            Ok(result) => pele::compat::render_result_envelope(&result),
            Err(e) => pele::compat::error_envelope(&e),
        }
    }
}
#[wasm_bindgen]
pub fn render_with_resources(
    source: &str,
    options_json: &str,
    width: Option<js_sys::Function>,
    icons: Option<js_sys::Function>,
) -> String {
    match pele::compat::render_options(options_json).and_then(|options| {
        pele::compat::Engine::with_builtins().render_source(
            source,
            &options,
            &mut BrowserResources { width, icons },
        )
    }) {
        Ok(result) => pele::compat::render_result_envelope(&result),
        Err(e) => pele::compat::error_envelope(&e),
    }
}

#[wasm_bindgen]
pub fn measure_text(text: &str, size: f64, style: u8) -> f64 {
    pele::compat::text::metrics_width(text, size, style)
}
