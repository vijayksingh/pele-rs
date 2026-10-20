use super::{
    module::*, source, Capability, DiagramId, DiagramType, EdgeLimit, ErrorCode, Inspection,
    Object, OutputLimit, ParseLimits, PeleError, PrepareOptions, SourceLimit, Value, DIAGRAM_TYPES,
};
use serde::Serialize;
use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};
type Model = Box<dyn Any + Send + Sync>;
trait ErasedModule: Send + Sync {
    fn parse(&self, context: ParseContext<'_>) -> Result<Model, PeleError>;
    fn parse_with_edge_limit(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Model, PeleError>;
    fn parse_prepared(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Model, PeleError>;
    fn render(
        &self,
        model: &Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError>;
    fn render_with_output_limit(
        &self,
        model: &Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
        limit: OutputLimit,
    ) -> Result<Rendered, PeleError>;
    fn render_raw(
        &self,
        model: &Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError>;
    fn model_view(&self, model: &Model) -> Result<Value, PeleError>;
    fn query(&self, model: &Model, operation: &str, arguments: &Value) -> Result<Value, PeleError>;
}
// Projection policy belongs to the erased module's concrete type, not its
// stored bytes. Unit builtin modules must remain zero-sized when boxed.
trait ModelProjection: Send + Sync + 'static {
    fn project(model: &impl Serialize) -> Result<Value, PeleError>;
}
struct Legacy;
impl ModelProjection for Legacy {
    fn project(model: &impl Serialize) -> Result<Value, PeleError> {
        project_model(model)
    }
}
// Selected only by private builtin registration. Public providers never inherit
// this policy from a matching descriptor, model type or registration name.
struct NormalizedBuiltin;
impl ModelProjection for NormalizedBuiltin {
    fn project(model: &impl Serialize) -> Result<Value, PeleError> {
        super::normalized_projection::project(model)
    }
}
struct Module<D, P>(D, PhantomData<P>);
struct ModelWriter {
    bytes: Vec<u8>,
    exceeded: bool,
}
impl std::io::Write for ModelWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > super::MAX_MODEL_JSON_BYTES - self.bytes.len() {
            self.exceeded = true;
            return Err(std::io::Error::other("model projection limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn project_model(model: &impl Serialize) -> Result<Value, PeleError> {
    let mut writer = ModelWriter {
        bytes: Vec::new(),
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut writer, model);
    if writer.exceeded {
        return Err(PeleError::new(
            format!(
                "Parsed model JSON exceeds {} bytes",
                super::MAX_MODEL_JSON_BYTES
            ),
            ErrorCode::Limit,
        ));
    }
    result
        .and_then(|_| {
            super::value::parse_model_json(
                std::str::from_utf8(&writer.bytes).expect("JSON serializer emits UTF-8"),
            )
        })
        .map_err(|e| {
            PeleError::new(
                format!("Module model cannot be serialized: {e}"),
                ErrorCode::Semantic,
            )
        })
}
impl<D: DiagramModule, P: ModelProjection> ErasedModule for Module<D, P> {
    fn parse(&self, context: ParseContext<'_>) -> Result<Model, PeleError> {
        Ok(Box::new(self.0.parse(context)?))
    }
    fn parse_with_edge_limit(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Model, PeleError> {
        Ok(Box::new(self.0.parse_with_edge_limit(context, limit)?))
    }
    fn parse_prepared(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Model, PeleError> {
        Ok(Box::new(self.0.parse_prepared(context, limit)?))
    }
    fn render(
        &self,
        model: &Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.0.render(
            model.downcast_ref().expect("private module/model pairing"),
            config,
            options,
            resources,
        )
    }
    fn render_raw(
        &self,
        model: &Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.0.render_raw(
            model.downcast_ref().expect("private module/model pairing"),
            config,
            options,
            resources,
        )
    }
    fn model_view(&self, model: &Model) -> Result<Value, PeleError> {
        let model = model
            .downcast_ref::<D::Model>()
            .expect("private module/model pairing");
        self.0.validate_model_projection(model)?;
        P::project(model)
    }
    fn render_with_output_limit(
        &self,
        model: &Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
        limit: OutputLimit,
    ) -> Result<Rendered, PeleError> {
        self.0.render_with_output_limit(
            model.downcast_ref().expect("private module/model pairing"),
            config,
            options,
            resources,
            limit,
        )
    }
    fn query(&self, model: &Model, operation: &str, arguments: &Value) -> Result<Value, PeleError> {
        self.0.query(
            model.downcast_ref().expect("private module/model pairing"),
            operation,
            arguments,
        )
    }
}
struct Registration {
    descriptor: ModuleDescriptor,
    module: Box<dyn ErasedModule>,
}
/// Shared source preparation. The chosen registration and policy are captured
/// before either the typed parser or an opted-in public parser consumes input.
pub(crate) enum ParseAdmission {
    Legacy,
    Source(SourceLimit),
    Limits(ParseLimits),
}
pub(crate) struct ParsePlan {
    registration: Arc<Registration>,
    source: String,
    config: Object,
    title: Option<String>,
    line_offset: usize,
    max_edges: usize,
    edge_limit: Option<EdgeLimit>,
}
impl ParsePlan {
    pub(crate) fn diagram_type(&self) -> &DiagramId {
        &self.registration.descriptor.id
    }
    pub(crate) fn context(&self) -> ParseContext<'_> {
        ParseContext {
            source: &self.source,
            config: &self.config,
            title: self.title.as_deref(),
            line_offset: self.line_offset,
            max_edges: self.max_edges,
        }
    }
    pub(crate) fn edge_limit(&self) -> Option<EdgeLimit> {
        self.edge_limit
    }
    pub(crate) fn into_config(self) -> Object {
        self.config
    }
    pub(crate) fn parse(self) -> Result<ParsedDocument, PeleError> {
        let model = match self.edge_limit {
            Some(limit) => self
                .registration
                .module
                .parse_with_edge_limit(self.context(), limit),
            None => self.registration.module.parse(self.context()),
        }?;
        Ok(ParsedDocument {
            registration: self.registration,
            model,
            config: self.config,
            edge_limit: self.edge_limit,
            prepared: false,
        })
    }
}
/// Already-prepared input borrows its source unchanged and retains its module.
pub(crate) struct PreparedParsePlan<'a> {
    registration: Arc<Registration>,
    context: ParseContext<'a>,
    config: Object,
    edge_limit: EdgeLimit,
}
impl PreparedParsePlan<'_> {
    #[cfg(feature = "web-host-er")]
    pub(crate) fn diagram_type(&self) -> &DiagramId {
        &self.registration.descriptor.id
    }
    pub(crate) fn parse(self) -> Result<ParsedDocument, PeleError> {
        let model = self
            .registration
            .module
            .parse_prepared(self.context, self.edge_limit)?;
        Ok(ParsedDocument {
            registration: self.registration,
            model,
            config: self.config,
            edge_limit: Some(self.edge_limit),
            prepared: true,
        })
    }
}
/// Immutable document with exact registration/config. Cannot be fabricated from JSON.
pub struct ParsedDocument {
    registration: Arc<Registration>,
    model: Model,
    config: Object,
    edge_limit: Option<EdgeLimit>,
    prepared: bool,
}
impl ParsedDocument {
    /// Exact parse override, if supplied. Exported convenience-model Origin
    /// integer fields retain their legacy projection; this records the override.
    /// Whether this document was parsed from an already-prepared diagram source.
    pub fn is_prepared(&self) -> bool {
        self.prepared
    }
    pub fn edge_limit(&self) -> Option<EdgeLimit> {
        self.edge_limit
    }
    pub fn diagram_type(&self) -> &DiagramId {
        &self.registration.descriptor.id
    }
    pub fn model<T: Any>(&self) -> Option<&T> {
        self.model.downcast_ref()
    }
    pub fn model_view(&self) -> Result<Value, PeleError> {
        self.registration
            .module
            .model_view(&self.model)
            .map_err(|mut error| {
                if error.diagram_type.is_none() {
                    error.diagram_type = Some(self.diagram_type().as_str().to_owned())
                }
                error
            })
    }
    // Consume parser-owned storage only within the paired public codec. The
    // captured module stays alive throughout the construction callback.
    pub(crate) fn into_public_state<T: Any, R>(
        self,
        create: impl FnOnce(T, bool, Option<EdgeLimit>) -> Result<R, PeleError>,
    ) -> Result<(R, Object), PeleError> {
        let Self {
            registration,
            model,
            config,
            edge_limit,
            prepared,
        } = self;
        let model = *model.downcast::<T>().expect("paired public codec");
        let state = create(model, prepared, edge_limit);
        drop(registration);
        state.map(|state| (state, config))
    }
    /// Query the retained typed model without re-parsing or retargeting registration.
    pub fn query(&self, operation: &str, arguments: &Value) -> Result<Value, PeleError> {
        self.registration
            .module
            .query(&self.model, operation, arguments)
            .map_err(|mut error| {
                if error.diagram_type.is_none() {
                    error.diagram_type = Some(self.diagram_type().as_str().to_owned());
                }
                error
            })
    }
    pub fn config(&self) -> &Object {
        &self.config
    }
}
/// Ordered registry. Metadata is captured once; existing documents retain their registration.
#[derive(Default, Clone)]
pub struct Engine {
    modules: Vec<Arc<Registration>>,
}
impl Engine {
    /// Registers implemented built-ins only; consult capabilities for parity status.
    pub fn with_builtins() -> Self {
        let mut engine = Self::empty();
        engine.register(super::diagrams::venn::Venn);
        engine.register(super::diagrams::ishikawa::Ishikawa);
        engine.register(super::diagrams::swimlane::Swimlane);
        engine.register(super::diagrams::wardley::Wardley);
        engine.register(super::diagrams::event_modeling::EventModeling);
        engine.register(super::diagrams::usecase::Usecase);
        engine.register(super::diagrams::cynefin::Cynefin);
        engine.register(super::diagrams::tree_view::TreeView);
        engine.register(super::diagrams::agentflow::AgentFlow);
        engine.register(super::diagrams::requirement::RequirementDiagram);
        engine.register(super::diagrams::block::Block);
        engine.register(super::diagrams::railroad::Railroad);
        engine.register(super::diagrams::treemap::Treemap);
        engine.register(super::diagrams::c4::C4);
        engine.register(super::diagrams::info::Info);
        engine.register(super::diagrams::git_graph::GitGraph);
        engine.register(super::diagrams::architecture::Architecture);
        engine.register(super::diagrams::sankey::Sankey);
        engine.register(super::diagrams::state::State);
        engine.register_projected::<_, NormalizedBuiltin>(super::diagrams::gantt::Gantt);
        engine.register(super::diagrams::sequence::Sequence);
        engine.register(super::diagrams::xychart::XyChart);
        engine.register(super::diagrams::class_diagram::Class);
        engine.register_projected::<_, NormalizedBuiltin>(super::diagrams::er::Er);
        engine.register(super::diagrams::flowchart::Flowchart);
        engine.register(super::diagrams::kanban::Kanban);
        engine.register(super::diagrams::mindmap::Mindmap);
        engine.register(super::diagrams::pie::Pie);
        engine.register(super::diagrams::quadrant_chart::QuadrantChart);
        engine.register(super::diagrams::packet::Packet);
        engine.register(super::diagrams::radar::Radar);
        engine.register(super::diagrams::timeline::Timeline);
        engine.register(super::diagrams::journey::Journey);
        engine
    }
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn register<D: DiagramModule>(&mut self, module: D) {
        self.register_projected::<D, Legacy>(module);
    }
    fn register_projected<D: DiagramModule, P: ModelProjection>(&mut self, module: D) {
        let descriptor = module.descriptor();
        let registration = Arc::new(Registration {
            descriptor,
            module: Box::new(Module::<D, P>(module, PhantomData)),
        });
        if let Some(i) = self
            .modules
            .iter()
            .position(|m| m.descriptor.id == registration.descriptor.id)
        {
            self.modules[i] = registration
        } else {
            self.modules.push(registration)
        }
    }
    pub fn registered(&self, id: impl Into<DiagramId>) -> bool {
        let id = id.into();
        self.modules.iter().any(|m| m.descriptor.id == id)
    }
    pub fn capabilities(&self) -> Vec<Capability> {
        let mut caps: Vec<_> = DIAGRAM_TYPES
            .iter()
            .map(|kind| Capability {
                diagram_type: (*kind).into(),
                known: true,
                registered: self.registered(*kind),
                syntax_parity: false,
                semantic_parity: false,
                render_parity: false,
                host_parity: false,
            })
            .collect();
        for module in &self.modules {
            if let DiagramId::Extension(_) = module.descriptor.id {
                caps.push(Capability {
                    diagram_type: module.descriptor.id.clone(),
                    known: false,
                    registered: true,
                    syntax_parity: false,
                    semantic_parity: false,
                    render_parity: false,
                    host_parity: false,
                })
            }
        }
        caps
    }
    pub fn detect_type(&self, source: &str) -> Result<Option<DiagramType>, PeleError> {
        Ok(source::detect(&source::preprocess(source)?.text))
    }
    pub fn supports(&self, source: &str) -> bool {
        self.detect_type(source)
            .ok()
            .flatten()
            .is_some_and(|kind| self.registered(kind))
    }
    pub fn inspect(&self, source: &str, options: &PrepareOptions) -> Result<Inspection, PeleError> {
        if source.encode_utf16().count() > options.limit {
            return Err(PeleError::new(
                format!(
                    "Diagram source is longer than the limit of {} characters.",
                    options.limit
                ),
                ErrorCode::Limit,
            ));
        }
        self.inspect_prechecked(source, &options.config)
    }
    /// Prepare well-formed source with an exact number-valued source limit.
    /// `limit` overrides only `options.limit`; configuration is retained. The
    /// independent public byte budget remains active for every argument value.
    pub fn inspect_with_source_limit(
        &self,
        source: &str,
        options: &PrepareOptions,
        limit: SourceLimit,
    ) -> Result<Inspection, PeleError> {
        limit.check(source)?;
        self.inspect_prechecked(source, &options.config)
    }
    fn inspect_prechecked(&self, source: &str, extra: &Object) -> Result<Inspection, PeleError> {
        let mut prepared = source::preprocess(source)?;
        let mut config = extra.clone();
        for (k, v) in prepared.config.entries() {
            config.insert(k.into(), v.clone())
        }
        prepared.config = config;
        let kind = source::detect(&prepared.text);
        let encoded_text = source::encode_entities(&prepared.text);
        Ok(Inspection {
            diagram_type: kind,
            registered: kind.is_some_and(|kind| self.registered(kind)),
            prepared,
            encoded_text,
        })
    }
    pub fn parse(&self, source: &str, options: &ParseOptions) -> Result<ParsedDocument, PeleError> {
        let inspection = self.inspect(source, &options.preparation)?;
        self.parse_detected(inspection, options.max_edges)
    }
    /// Parse once with exact source-limit semantics and the existing typed
    /// configuration/edge limit. No preparation is repeated through the legacy
    /// usize source check; all other module inputs remain unchanged.
    pub fn parse_with_source_limit(
        &self,
        source: &str,
        options: &ParseOptions,
        limit: SourceLimit,
    ) -> Result<ParsedDocument, PeleError> {
        let inspection = self.inspect_with_source_limit(source, &options.preparation, limit)?;
        self.parse_detected(inspection, options.max_edges)
    }
    fn parse_detected(
        &self,
        inspection: Inspection,
        max_edges: usize,
    ) -> Result<ParsedDocument, PeleError> {
        let kind = inspection.diagram_type.ok_or_else(|| {
            PeleError::new(
                "No diagram type detected for the given text.",
                ErrorCode::UnsupportedDiagram,
            )
        })?;
        self.parse_prepared(inspection, &kind.into(), max_edges)
    }
    /// Explicit selection lets consumers add new languages without editing the built-in detector.
    pub fn parse_as(
        &self,
        id: &DiagramId,
        source: &str,
        options: &ParseOptions,
    ) -> Result<ParsedDocument, PeleError> {
        self.parse_prepared(
            self.inspect(source, &options.preparation)?,
            id,
            options.max_edges,
        )
    }
    /// Explicit module selection with an exact source limit. The selected
    /// registration, configuration and integer edge limit retain their normal
    /// ownership and behavior.
    pub fn parse_as_with_source_limit(
        &self,
        id: &DiagramId,
        source: &str,
        options: &ParseOptions,
        limit: SourceLimit,
    ) -> Result<ParsedDocument, PeleError> {
        self.parse_prepared(
            self.inspect_with_source_limit(source, &options.preparation, limit)?,
            id,
            options.max_edges,
        )
    }
    /// Composable exact source/edge overrides. Omitted overrides retain typed
    /// options. The native source-byte ceiling applies to this entire new lane.
    pub fn parse_with_limits(
        &self,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
    ) -> Result<ParsedDocument, PeleError> {
        let inspection = self.inspect_with_limits(source, options, limits)?;
        let kind = inspection.diagram_type.ok_or_else(|| {
            PeleError::new(
                "No diagram type detected for the given text.",
                ErrorCode::UnsupportedDiagram,
            )
        })?;
        self.parse_prepared_policy(
            inspection,
            &kind.into(),
            options.max_edges,
            limits.max_edges,
        )
    }
    pub fn parse_as_with_limits(
        &self,
        id: &DiagramId,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
    ) -> Result<ParsedDocument, PeleError> {
        self.parse_prepared_policy(
            self.inspect_with_limits(source, options, limits)?,
            id,
            options.max_edges,
            limits.max_edges,
        )
    }
    pub fn inspect_with_limits(
        &self,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
    ) -> Result<Inspection, PeleError> {
        SourceLimit::check_bytes(source)?;
        match limits.source {
            Some(limit) => self.inspect_with_source_limit(source, &options.preparation, limit),
            None => self.inspect(source, &options.preparation),
        }
    }
    fn parse_prepared(
        &self,
        inspection: Inspection,
        id: &DiagramId,
        max_edges: usize,
    ) -> Result<ParsedDocument, PeleError> {
        self.parse_prepared_policy(inspection, id, max_edges, None)
    }
    fn parse_prepared_policy(
        &self,
        inspection: Inspection,
        id: &DiagramId,
        max_edges: usize,
        edge_limit: Option<EdgeLimit>,
    ) -> Result<ParsedDocument, PeleError> {
        self.prepare_inspected(inspection, Some(id), max_edges, edge_limit)?
            .parse()
    }
    /// Internal preparation seam shared by native typed/public entry points.
    /// Raw prepared parsing retains its existing borrowed-input path.
    pub(crate) fn prepare_parse(
        &self,
        source: &str,
        options: &ParseOptions,
        selected: Option<&DiagramId>,
        admission: ParseAdmission,
    ) -> Result<ParsePlan, PeleError> {
        let (inspection, edge_limit) = match admission {
            ParseAdmission::Legacy => (self.inspect(source, &options.preparation)?, None),
            ParseAdmission::Source(limit) => (
                self.inspect_with_source_limit(source, &options.preparation, limit)?,
                None,
            ),
            ParseAdmission::Limits(limits) => (
                self.inspect_with_limits(source, options, limits)?,
                limits.max_edges,
            ),
        };
        self.prepare_inspected(inspection, selected, options.max_edges, edge_limit)
    }
    fn prepare_inspected(
        &self,
        inspection: Inspection,
        selected: Option<&DiagramId>,
        max_edges: usize,
        edge_limit: Option<EdgeLimit>,
    ) -> Result<ParsePlan, PeleError> {
        let id = match selected {
            Some(id) => id.clone(),
            None => inspection
                .diagram_type
                .ok_or_else(|| {
                    PeleError::new(
                        "No diagram type detected for the given text.",
                        ErrorCode::UnsupportedDiagram,
                    )
                })?
                .into(),
        };
        let registration = self
            .modules
            .iter()
            .find(|m| m.descriptor.id == id)
            .ok_or_else(|| {
                let mut error = PeleError::new(
                    format!("Diagram type \"{id}\" is not supported yet."),
                    ErrorCode::UnsupportedDiagram,
                );
                error.diagram_type = Some(id.to_string());
                error
            })?
            .clone();
        let pre = inspection.prepared;
        let mut source = if registration.descriptor.keep_comments {
            source::encode_entities(&pre.with_comments)
        } else {
            inspection.encoded_text
        };
        source.push('\n');
        Ok(ParsePlan {
            registration,
            source,
            config: pre.config,
            title: pre.title,
            line_offset: pre.line_offset,
            max_edges,
            edge_limit,
        })
    }
    /// Parse an already-prepared diagram source verbatim. No source limit,
    /// preprocessing, detection, entity encoding or newline append is performed.
    /// The independent native source-byte and edge-generation budgets remain.
    /// Undefined here is the raw low-level policy, not core's default 5,000.
    pub fn parse_prepared_as(
        &self,
        id: &DiagramId,
        context: ParseContext<'_>,
        edge_limit: EdgeLimit,
    ) -> Result<ParsedDocument, PeleError> {
        self.prepare_prepared_as(id, context, edge_limit)?.parse()
    }
    pub(crate) fn prepare_prepared_as<'a>(
        &self,
        id: &DiagramId,
        context: ParseContext<'a>,
        edge_limit: EdgeLimit,
    ) -> Result<PreparedParsePlan<'a>, PeleError> {
        SourceLimit::check_bytes(context.source)?;
        if context
            .line_offset
            .checked_add(context.source.len())
            .is_none()
        {
            return Err(PeleError::new(
                "Prepared source position exceeds native range",
                ErrorCode::Limit,
            ));
        }
        let registration = self
            .modules
            .iter()
            .find(|m| &m.descriptor.id == id)
            .cloned()
            .ok_or_else(|| {
                let mut e = PeleError::new(
                    format!("Diagram type \"{id}\" is not supported yet."),
                    ErrorCode::UnsupportedDiagram,
                );
                e.diagram_type = Some(id.to_string());
                e
            })?;
        let config = context.config.clone();
        Ok(PreparedParsePlan {
            registration,
            context,
            config,
            edge_limit,
        })
    }
    /// Invoke only the retained module's raw renderer with the supplied config.
    /// Core responsive/output-limit/theme/link-id handling is deliberately absent.
    pub fn render_document_raw(
        &self,
        document: &ParsedDocument,
        config: &Object,
        settings: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render_raw_registered(
            &document.registration.descriptor,
            settings,
            resources,
            |session| {
                document
                    .registration
                    .module
                    .render_raw(&document.model, config, settings, session)
            },
        )
    }
    pub fn render_source(
        &self,
        source: &str,
        options: &RenderOptions,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        let document = self.parse(source, &options.parse)?;
        self.render_document(&document, options, resources)
    }
    pub fn render_document(
        &self,
        document: &ParsedDocument,
        options: &RenderOptions,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        let registration = &document.registration;
        render_registered(
            &registration.descriptor,
            &document.config,
            options,
            resources,
            |settings, session| {
                registration
                    .module
                    .render(&document.model, &document.config, settings, session)
            },
        )
    }
    /// Render well-formed source with an exact output threshold. Source/edge
    /// limits keep their existing typed contract; only output_limit is overridden.
    pub fn render_source_with_output_limit(
        &self,
        source: &str,
        options: &RenderOptions,
        limit: OutputLimit,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        let document = self.parse(source, &options.parse)?;
        self.render_document_with_output_limit(&document, options, limit, resources)
    }
    /// Use the retained registration and exact post-render UTF-16 comparison.
    /// The independent returned-SVG byte budget remains active for NaN/infinity.
    pub fn render_document_with_output_limit(
        &self,
        document: &ParsedDocument,
        options: &RenderOptions,
        limit: OutputLimit,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        let registration = &document.registration;
        render_registered_with_output_limit(
            &registration.descriptor,
            &document.config,
            options,
            resources,
            limit,
            |settings, session| {
                registration.module.render_with_output_limit(
                    &document.model,
                    &document.config,
                    settings,
                    session,
                    limit,
                )
            },
        )
    }
    /// Render in the bounded native lane. None retains the typed integer output
    /// policy and module dispatch; Some selects the exact post-render policy.
    /// Native source/output byte ceilings cannot be disabled by either policy.
    pub fn render_source_bounded(
        &self,
        source: &str,
        options: &RenderOptions,
        output_limit: Option<OutputLimit>,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        SourceLimit::check_bytes(source)?;
        let document = self.parse(source, &options.parse)?;
        self.render_document_bounded(&document, options, output_limit, resources)
    }
    /// Retain the exact provider/config while independently bounding returned SVG
    /// bytes before and after native theme resolution. This does not bound every
    /// provider's peak allocation. Existing legacy methods keep their contract.
    pub fn render_document_bounded(
        &self,
        document: &ParsedDocument,
        options: &RenderOptions,
        output_limit: Option<OutputLimit>,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        let registration = &document.registration;
        render_registered_policy(
            &registration.descriptor,
            &document.config,
            options,
            resources,
            output_limit,
            true,
            |settings, session| match output_limit {
                Some(limit) => registration.module.render_with_output_limit(
                    &document.model,
                    &document.config,
                    settings,
                    session,
                    limit,
                ),
                None => {
                    registration
                        .module
                        .render(&document.model, &document.config, settings, session)
                }
            },
        )
    }
}
pub(crate) fn render_registered(
    descriptor: &ModuleDescriptor,
    config: &Object,
    options: &RenderOptions,
    resources: &mut dyn RenderResources,
    render: impl FnOnce(&RenderSettings, &mut dyn RenderResources) -> Result<Rendered, PeleError>,
) -> Result<RenderResult, PeleError> {
    render_registered_policy(descriptor, config, options, resources, None, false, render)
}
pub(crate) fn render_registered_with_output_limit(
    descriptor: &ModuleDescriptor,
    config: &Object,
    options: &RenderOptions,
    resources: &mut dyn RenderResources,
    limit: OutputLimit,
    render: impl FnOnce(&RenderSettings, &mut dyn RenderResources) -> Result<Rendered, PeleError>,
) -> Result<RenderResult, PeleError> {
    render_registered_policy(
        descriptor,
        config,
        options,
        resources,
        Some(limit),
        true,
        render,
    )
}
pub(crate) fn render_registered_policy(
    descriptor: &ModuleDescriptor,
    config: &Object,
    options: &RenderOptions,
    resources: &mut dyn RenderResources,
    limit: Option<OutputLimit>,
    enforce_native_bytes: bool,
    render: impl FnOnce(&RenderSettings, &mut dyn RenderResources) -> Result<Rendered, PeleError>,
) -> Result<RenderResult, PeleError> {
    let mut settings = options.render.clone();
    let section = descriptor
        .section
        .as_deref()
        .unwrap_or(descriptor.id.as_str());
    let fixed = config
        .get(section)
        .and_then(Value::as_object)
        .and_then(|s| s.get("useMaxWidth"))
        == Some(&Value::Bool(false));
    if settings.responsive.is_none() {
        settings.responsive = Some(!fixed)
    }
    let presentation_budget = resources.presentation_budget();
    let mut session = RenderSession {
        presentation_budget,
        presentation: None,
        host: resources,
        icons: HashMap::new(),
        clock: settings.now.filter(|value| value.is_finite()).map(Ok),
    };
    let mut rendered = render(&settings, &mut session)?;
    let native_bytes = enforce_native_bytes || limit.is_some();
    if native_bytes {
        OutputLimit::check_bytes(&rendered.svg, descriptor.id.as_str())?;
    }
    if let Some(theme) = &settings.native_theme {
        rendered.svg = super::svg::resolved_svg(&rendered.svg, theme)?;
    }
    if native_bytes {
        OutputLimit::check_bytes(&rendered.svg, descriptor.id.as_str())?;
    }
    if let Some(limit) = limit {
        limit.check(&rendered.svg, descriptor.id.as_str())?;
    } else if rendered.svg.encode_utf16().count() > settings.output_limit() {
        let mut e = PeleError::new(
            format!(
                "Diagram output is longer than the limit of {} characters.",
                settings.output_limit()
            ),
            ErrorCode::Limit,
        );
        e.diagram_type = Some(descriptor.id.to_string());
        return Err(e);
    }
    for link in &mut rendered.links {
        link.id = source::decode_entities(&link.id)?
    }
    if session.presentation_budget.is_some() {
        session
            .host
            .capture_presentation(session.presentation.take().unwrap_or(
                super::PresentationOutcome::Unavailable(
                    super::PresentationUnavailable::UnsupportedDiagram,
                ),
            ));
    }
    Ok(RenderResult {
        diagram_type: descriptor.id.clone(),
        rendered,
    })
}

pub(crate) fn render_raw_registered(
    descriptor: &ModuleDescriptor,
    settings: &RenderSettings,
    host: &mut dyn RenderResources,
    render: impl FnOnce(&mut dyn RenderResources) -> Result<Rendered, PeleError>,
) -> Result<Rendered, PeleError> {
    let mut resources = RawRenderSession {
        session: RenderSession {
            presentation_budget: None,
            presentation: None,
            host,
            icons: HashMap::new(),
            clock: settings.now.filter(|n| n.is_finite()).map(Ok),
        },
    };
    let rendered = render(&mut resources)?;
    OutputLimit::check_bytes(&rendered.svg, descriptor.id.as_str())?;
    Ok(rendered)
}
struct RawRenderSession<'a> {
    session: RenderSession<'a>,
}
impl RenderResources for RawRenderSession<'_> {
    fn host_function(
        &mut self,
        reference: u32,
    ) -> Result<std::rc::Rc<dyn super::public::HostFunction>, super::public::PublicError> {
        self.session.host_function(reference)
    }

    fn image(&mut self, uri: &str) -> Result<super::ImageResource, PeleError> {
        self.session.host.image(uri)
    }
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.session.text_width(r)
    }
    // The raw entry does not establish core's global inline icon resolver.
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.session.host.raw_inline_icon(name)
    }
    fn direct_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.session.host.icon(name)
    }
    fn parse_date(&mut self, s: &str) -> Result<Option<f64>, PeleError> {
        self.session.parse_date(s)
    }
    fn calendar_fields(&mut self, n: f64) -> Result<Option<super::CalendarFields>, PeleError> {
        self.session.calendar_fields(n)
    }
    fn calendar_instant(&mut self, c: super::CivilDateTime) -> Result<Option<f64>, PeleError> {
        self.session.calendar_instant(c)
    }
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.session.direct_now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        self.session.now_millis()
    }
}
struct RenderSession<'a> {
    presentation_budget: Option<super::PresentationBudget>,
    presentation: Option<super::PresentationOutcome>,
    host: &'a mut dyn RenderResources,
    icons: HashMap<String, Option<TrustedSvg>>,
    clock: Option<Result<f64, PeleError>>,
}
impl RenderResources for RenderSession<'_> {
    fn host_function(
        &mut self,
        reference: u32,
    ) -> Result<std::rc::Rc<dyn super::public::HostFunction>, super::public::PublicError> {
        self.host.host_function(reference)
    }

    fn presentation_budget(&self) -> Option<super::PresentationBudget> {
        self.presentation_budget
    }
    fn capture_presentation(&mut self, outcome: super::PresentationOutcome) {
        if self.presentation_budget.is_some() {
            self.presentation = Some(outcome);
        }
    }
    fn raw_inline_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.raw_inline_icon(name)
    }
    fn image(&mut self, uri: &str) -> Result<super::ImageResource, PeleError> {
        self.host.image(uri)
    }
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        self.host.text_width(request)
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        if let Some(result) = self.icons.get(name) {
            return Ok(result.clone());
        }
        let result = self.host.icon(&source::decode_entities(name)?)?;
        self.icons.insert(name.into(), result.clone());
        Ok(result)
    }
    fn direct_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.icon(&source::decode_entities(name)?)
    }
    fn parse_date(&mut self, input: &str) -> Result<Option<f64>, PeleError> {
        Ok(self
            .host
            .parse_date(input)?
            .filter(|value| value.is_finite() && value.abs() <= 8.64e15)
            .map(f64::trunc))
    }
    fn calendar_fields(
        &mut self,
        instant_ms: f64,
    ) -> Result<Option<super::CalendarFields>, PeleError> {
        if !instant_ms.is_finite() || instant_ms.abs() > 8.64e15 {
            return Ok(None);
        }
        let fields = self.host.calendar_fields(instant_ms.trunc())?;
        if fields.is_some_and(|fields| {
            let c = fields.civil;
            i64::from(c.year).abs() > 300_000
                || !(1..=12).contains(&c.month)
                || !(1..=31).contains(&c.day)
                || c.hour > 23
                || c.minute > 59
                || c.second > 59
                || c.millisecond > 999
                || fields.weekday > 6
                || i64::from(fields.utc_offset_minutes).abs() > 1440
        }) {
            return Err(PeleError::new(
                "Host calendar returned invalid fields",
                ErrorCode::Semantic,
            ));
        }
        Ok(fields)
    }
    fn calendar_instant(&mut self, civil: super::CivilDateTime) -> Result<Option<f64>, PeleError> {
        Ok(self
            .host
            .calendar_instant(civil)?
            .filter(|value| value.is_finite() && value.abs() <= 8.64e15)
            .map(f64::trunc))
    }
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.direct_now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        if self.clock.is_none() {
            self.clock = Some(self.host.now_millis())
        }
        self.clock.as_ref().unwrap().clone()
    }
}
impl Serialize for ParsedDocument {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.model_view()
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
}
