use super::{
    schema::CompiledSchema, Atom, FunctionShape, InvocationOutcome, MethodCall, ModelGraph,
    ModelSchema, ModelSchemaView, PublicError, RenderOutcome, SharedFunction, OPAQUE_MODEL_SCHEMA,
};
use crate::compat::engine::{ParseAdmission, ParsePlan};
use crate::compat::{
    DiagramId, DiagramModule, EdgeLimit, Engine, ErrorCode, ModuleDescriptor, Object, OutputLimit,
    ParseContext, ParseLimits, ParseOptions, PeleError, RenderOptions, RenderResources,
    RenderResult, RenderSettings, Rendered, SourceLimit,
};
use std::{
    any::Any,
    borrow::Cow,
    sync::{Arc, OnceLock},
};
/// Typed codec opt-in. Parsing, geometry and rendering remain in the native module.
pub trait PublicDiagramModule: DiagramModule {
    type PublicState: Send + 'static;
    fn model_schema(&self) -> &'static ModelSchema {
        &OPAQUE_MODEL_SCHEMA
    }
    fn shared_functions(&self) -> &'static [SharedFunction] {
        &[]
    }
    fn function_shapes(&self) -> &'static [FunctionShape] {
        &[]
    }
    /// Method IDs, receiver identity and arguments come from this document's
    /// current arena. The module owns native behavior, including partial edits.
    fn invoke_public(
        &self,
        _state: &mut Self::PublicState,
        _call: MethodCall,
        _config: &Object,
        _host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        Err(PeleError::new("Public model methods are unavailable", ErrorCode::Semantic).into())
    }
    /// Optional eager parser for fresh public state. Returning None leaves the
    /// established typed-parse/create sequence intact. An implementation must
    /// preserve all parse errors, admission limits and final public behavior;
    /// it may not publish state or invoke host code before returning None.
    /// Only ordinary textual parse calls use this hook; raw prepared input and
    /// adaptation of existing typed models retain their existing interfaces.
    fn try_parse_public(
        &self,
        _context: ParseContext<'_>,
        _edge_limit: Option<EdgeLimit>,
    ) -> Option<Result<Self::PublicState, PeleError>> {
        None
    }
    fn create_public(&self, model: &Self::Model) -> Result<Self::PublicState, PeleError>;
    /// Construct this module's real mutable state directly from a caller arena.
    /// No source parsing, seeded model or missing-property defaults are implied.
    fn create_public_from_graph(&self, _graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        Err(PeleError::new(
            "Public model construction is unavailable",
            ErrorCode::Semantic,
        ))
    }
    /// Seed exact parse-time edge options before deriving public state. Once
    /// created, the mutable graph owns its options; provenance is not reapplied.
    fn create_public_with_edge_limit(
        &self,
        model: &Self::Model,
        _limit: EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        self.create_public(model)
    }
    /// Prepared parsing preserves raw low-level policy presence. Core's
    /// undefined default remains unchanged in create_public_with_edge_limit.
    fn create_public_prepared(
        &self,
        model: &Self::Model,
        limit: EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        self.create_public_with_edge_limit(model, limit)
    }
    fn export_public(&self, state: &Self::PublicState) -> Result<ModelGraph, PeleError>;
    /// Optional allocation-free export of the same graph as `export_public`.
    /// Opt in only when export has no additional observable work or failures;
    /// document snapshots still validate this graph against the retained schema.
    fn borrow_public_graph<'a>(&self, _state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        None
    }
    /// Whether a host may omit reapplying its exactly acknowledged graph.
    /// Applying identical data must have no semantic effects beyond refreshing
    /// graph-derived caches. This never changes `PublicDocument::apply` itself
    /// or its revision advancement; extensions conservatively remain opted out.
    fn unchanged_public_graph_is_noop(&self) -> bool {
        false
    }
    fn apply_public(
        &self,
        state: &mut Self::PublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError>;
    /// Stateful render with native JavaScript/lossless error categories. Existing
    /// modules keep their render implementation through this default adapter.
    fn render_public_operation(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.render_public(state, config, options, host)
            .map_err(Into::into)
    }
    fn render_public_raw_operation(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.render_public_operation(state, config, settings, host)
    }
    /// Exact post-render output policy, retaining native JavaScript/host errors.
    /// Override when the provider privately enforces legacy integer output limits.
    fn render_public_operation_with_output_limit(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
        _limit: OutputLimit,
    ) -> Result<Rendered, PublicError> {
        self.render_public_operation(state, config, options, host)
    }
    fn render_public(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError>;
}
// Opted-in edge builders share the same ordinary options.maxEdges own field.
pub(crate) fn create_prepared_edge_public<D: PublicDiagramModule>(
    module: &D,
    model: &D::Model,
    limit: EdgeLimit,
) -> Result<D::PublicState, PeleError> {
    let mut state = module.create_public_with_edge_limit(model, limit)?;
    if matches!(limit, EdgeLimit::Undefined) {
        let mut graph = module.export_public(&state)?;
        let Atom::Ref(root) = graph.root else {
            return Err(PeleError::new(
                "Prepared edge model has no root",
                ErrorCode::Semantic,
            ));
        };
        let options = graph.nodes[root as usize]
            .properties()
            .iter()
            .find(|p| graph.strings[p.key as usize].is("options"))
            .and_then(|p| {
                if let Atom::Ref(id) = p.value {
                    Some(id)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                PeleError::new("Prepared edge options are missing", ErrorCode::Semantic)
            })?;
        let position = graph.nodes[options as usize]
            .properties()
            .iter()
            .position(|p| graph.strings[p.key as usize].is("maxEdges"))
            .ok_or_else(|| {
                PeleError::new("Prepared edge policy is missing", ErrorCode::Semantic)
            })?;
        graph.nodes[options as usize].properties_mut()[position].value = Atom::Undefined;
        module.apply_public(&mut state, graph)?;
    }
    Ok(state)
}
struct Shared<D> {
    module: Arc<D>,
    descriptor: ModuleDescriptor,
}
impl<D: DiagramModule> DiagramModule for Shared<D> {
    type Model = D::Model;
    fn descriptor(&self) -> ModuleDescriptor {
        self.descriptor.clone()
    }
    fn query(
        &self,
        model: &Self::Model,
        operation: &str,
        arguments: &crate::compat::Value,
    ) -> Result<crate::compat::Value, PeleError> {
        self.module.query(model, operation, arguments)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        self.module.parse(c)
    }
    fn parse_with_edge_limit(
        &self,
        c: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        self.module.parse_with_edge_limit(c, limit)
    }
    fn parse_prepared(
        &self,
        c: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        self.module.parse_prepared(c, limit)
    }
    fn validate_model_projection(&self, m: &Self::Model) -> Result<(), PeleError> {
        self.module.validate_model_projection(m)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.module.render(m, c, o, h)
    }
    fn render_raw(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.module.render_raw(m, c, o, h)
    }
    fn render_with_output_limit(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
        limit: OutputLimit,
    ) -> Result<Rendered, PeleError> {
        self.module.render_with_output_limit(m, c, o, h, limit)
    }
}
type State = Box<dyn Any + Send>;
trait Erased: Send + Sync {
    fn try_parse_public(
        &self,
        context: ParseContext<'_>,
        edge_limit: Option<EdgeLimit>,
    ) -> Option<Result<State, PeleError>>;
    fn create(&self, doc: &crate::compat::ParsedDocument) -> Result<State, PeleError>;
    fn create_graph(&self, graph: ModelGraph) -> Result<State, PeleError>;
    fn graph(&self, state: &State) -> Result<ModelGraph, PeleError>;
    fn borrow_graph<'a>(&self, state: &'a State) -> Option<&'a ModelGraph>;
    fn unchanged_public_graph_is_noop(&self) -> bool;
    fn apply(&self, state: &mut State, graph: ModelGraph) -> Result<(), PeleError>;
    fn invoke(
        &self,
        state: &mut State,
        call: MethodCall,
        config: &Object,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError>;
    fn render_operation(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError>;
    fn render_raw_operation(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError>;
    fn render_operation_with_output_limit(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
        limit: OutputLimit,
    ) -> Result<Rendered, PublicError>;
    fn render(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError>;
}
struct Codec<D>(Arc<D>);
impl<D: PublicDiagramModule> Erased for Codec<D> {
    fn try_parse_public(
        &self,
        context: ParseContext<'_>,
        edge_limit: Option<EdgeLimit>,
    ) -> Option<Result<State, PeleError>> {
        self.0
            .try_parse_public(context, edge_limit)
            .map(|result| result.map(|state| Box::new(state) as State))
    }
    fn create(&self, doc: &crate::compat::ParsedDocument) -> Result<State, PeleError> {
        let model = doc.model::<D::Model>().expect("paired public codec");
        if doc.is_prepared() {
            return Ok(Box::new(self.0.create_public_prepared(
                model,
                doc.edge_limit().expect("prepared policy"),
            )?));
        }
        Ok(Box::new(match doc.edge_limit() {
            Some(limit) => self.0.create_public_with_edge_limit(model, limit)?,
            None => self.0.create_public(model)?,
        }))
    }
    fn create_graph(&self, graph: ModelGraph) -> Result<State, PeleError> {
        Ok(Box::new(self.0.create_public_from_graph(graph)?))
    }
    fn graph(&self, s: &State) -> Result<ModelGraph, PeleError> {
        self.0
            .export_public(s.downcast_ref().expect("paired public state"))
    }
    fn borrow_graph<'a>(&self, s: &'a State) -> Option<&'a ModelGraph> {
        self.0
            .borrow_public_graph(s.downcast_ref().expect("paired public state"))
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        self.0.unchanged_public_graph_is_noop()
    }
    fn apply(&self, s: &mut State, g: ModelGraph) -> Result<(), PeleError> {
        self.0
            .apply_public(s.downcast_mut().expect("paired public state"), g)
    }
    fn invoke(
        &self,
        s: &mut State,
        call: MethodCall,
        c: &Object,
        h: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        self.0
            .invoke_public(s.downcast_mut().expect("paired public state"), call, c, h)
    }
    fn render_operation(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.0.render_public_operation(
            state.downcast_mut().expect("paired public state"),
            config,
            options,
            host,
        )
    }
    fn render_raw_operation(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.0.render_public_raw_operation(
            state.downcast_mut().expect("paired public state"),
            config,
            options,
            host,
        )
    }
    fn render_operation_with_output_limit(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
        limit: OutputLimit,
    ) -> Result<Rendered, PublicError> {
        self.0.render_public_operation_with_output_limit(
            state.downcast_mut().expect("paired public state"),
            config,
            options,
            host,
            limit,
        )
    }
    fn render(
        &self,
        s: &mut State,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.0
            .render_public(s.downcast_mut().expect("paired public state"), c, o, h)
    }
}
struct Registration {
    descriptor: ModuleDescriptor,
    schema: &'static ModelSchema,
    compiled_schema: OnceLock<Result<CompiledSchema, PeleError>>,
    shared_functions: &'static [SharedFunction],
    function_shapes: &'static [FunctionShape],
    codec: Box<dyn Erased>,
}
impl Registration {
    fn compiled_schema(&self) -> Result<&CompiledSchema, PeleError> {
        // Register stays infallible. Cache validation (including its error) only
        // where this captured schema would already have been checked.
        self.compiled_schema
            .get_or_init(|| CompiledSchema::new(self.schema))
            .as_ref()
            .map_err(Clone::clone)
    }
    fn validate_graph(&self, graph: &ModelGraph) -> Result<(), PeleError> {
        graph.validate_compiled(self.compiled_schema()?)
    }
}
#[derive(Default)]
pub struct PublicEngine {
    engine: Engine,
    modules: Vec<Arc<Registration>>,
}
impl PublicEngine {
    /// All built-in mutable codecs. Register extensions/replacements through the
    /// same typed interface; registration alone does not certify runtime parity.
    pub fn with_builtins() -> Self {
        let mut engine = Self::empty();
        for kind in crate::compat::DIAGRAM_TYPES {
            engine.register_builtin(kind);
        }
        engine
    }
    /// Install one canonical built-in through the same typed module interface
    /// used by extensions. Repeated registration replaces later parses only.
    pub fn register_builtin(&mut self, kind: crate::compat::DiagramType) {
        use crate::compat::{diagrams as d, DiagramType as T};
        match kind {
            T::Flowchart => self.register(d::flowchart::Flowchart),
            T::Pie => self.register(d::pie::Pie),
            T::Er => self.register(d::er::Er),
            T::Mindmap => self.register(d::mindmap::Mindmap),
            T::Kanban => self.register(d::kanban::Kanban),
            T::Timeline => self.register(d::timeline::Timeline),
            T::Journey => self.register(d::journey::Journey),
            T::Class => self.register(d::class_diagram::Class),
            T::QuadrantChart => self.register(d::quadrant_chart::QuadrantChart),
            T::Xychart => self.register(d::xychart::XyChart),
            T::State => self.register(d::state::State),
            T::Gantt => self.register(d::gantt::Gantt),
            T::GitGraph => self.register(d::git_graph::GitGraph),
            T::Architecture => self.register(d::architecture::Architecture),
            T::Requirement => self.register(d::requirement::RequirementDiagram),
            T::Sankey => self.register(d::sankey::Sankey),
            T::Packet => self.register(d::packet::Packet),
            T::Radar => self.register(d::radar::Radar),
            T::Treemap => self.register(d::treemap::Treemap),
            T::Info => self.register(d::info::Info),
            T::Sequence => self.register(d::sequence::Sequence),
            T::C4 => self.register(d::c4::C4),
            T::Block => self.register(d::block::Block),
            T::Railroad => self.register(d::railroad::Railroad),
            T::Agentflow => self.register(d::agentflow::AgentFlow),
            T::TreeView => self.register(d::tree_view::TreeView),
            T::Cynefin => self.register(d::cynefin::Cynefin),
            T::Wardley => self.register(d::wardley::Wardley),
            T::Eventmodeling => self.register(d::event_modeling::EventModeling),
            T::Usecase => self.register(d::usecase::Usecase),
            T::Venn => self.register(d::venn::Venn),
            T::Ishikawa => self.register(d::ishikawa::Ishikawa),
            T::Swimlane => self.register(d::swimlane::Swimlane),
        }
    }
    pub fn registered(&self, id: impl Into<DiagramId>) -> bool {
        self.engine.registered(id)
    }
    pub fn capabilities(&self) -> Vec<crate::compat::Capability> {
        self.engine.capabilities()
    }
    /// Discovery preprocesses without applying parse limits or grammar checks.
    pub fn detect_type(
        &self,
        source: &str,
    ) -> Result<Option<crate::compat::DiagramType>, PeleError> {
        self.engine.detect_type(source)
    }
    /// Preprocessing failure is unsupported; parsing remains a separate step.
    pub fn supports(&self, source: &str) -> bool {
        self.engine.supports(source)
    }
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn register<D: PublicDiagramModule>(&mut self, module: D) {
        let descriptor = module.descriptor();
        let schema = module.model_schema();
        let shared_functions = module.shared_functions();
        let function_shapes = module.function_shapes();
        let shared = Arc::new(module);
        self.engine.register(Shared {
            module: shared.clone(),
            descriptor: descriptor.clone(),
        });
        let registration = Arc::new(Registration {
            descriptor,
            schema,
            compiled_schema: OnceLock::new(),
            shared_functions,
            function_shapes,
            codec: Box::new(Codec(shared)),
        });
        if let Some(slot) = self
            .modules
            .iter_mut()
            .find(|m| m.descriptor.id == registration.descriptor.id)
        {
            *slot = registration
        } else {
            self.modules.push(registration)
        }
    }
    pub fn parse(&self, source: &str, options: &ParseOptions) -> Result<PublicDocument, PeleError> {
        self.parse_plan(
            self.engine
                .prepare_parse(source, options, None, ParseAdmission::Legacy)?,
        )
    }
    /// Parse through the retained native codec with an exact number-valued
    /// source limit. Only the legacy source limit is overridden.
    pub fn parse_with_source_limit(
        &self,
        source: &str,
        options: &ParseOptions,
        limit: SourceLimit,
    ) -> Result<PublicDocument, PeleError> {
        self.parse_plan(self.engine.prepare_parse(
            source,
            options,
            None,
            ParseAdmission::Source(limit),
        )?)
    }
    pub fn parse_with_limits(
        &self,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
    ) -> Result<PublicDocument, PeleError> {
        self.parse_plan(self.engine.prepare_parse(
            source,
            options,
            None,
            ParseAdmission::Limits(limits),
        )?)
    }
    pub fn parse_prepared_as(
        &self,
        id: &DiagramId,
        context: ParseContext<'_>,
        edge_limit: EdgeLimit,
    ) -> Result<PublicDocument, PeleError> {
        self.create_document(self.engine.parse_prepared_as(id, context, edge_limit)?)
    }
    /// Render source through this registry with exact parse limits, without
    /// exporting or importing a public model graph solely to produce SVG.
    pub fn render_source_bounded_with_limits(
        &self,
        source: &str,
        options: &RenderOptions,
        limits: ParseLimits,
        output_limit: Option<OutputLimit>,
        host: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        SourceLimit::check_bytes(source)?;
        let document = self
            .engine
            .parse_with_limits(source, &options.parse, limits)?;
        self.engine
            .render_document_bounded(&document, options, output_limit, host)
    }
    pub fn parse_as_with_limits(
        &self,
        id: &DiagramId,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
    ) -> Result<PublicDocument, PeleError> {
        self.parse_plan(self.engine.prepare_parse(
            source,
            options,
            Some(id),
            ParseAdmission::Limits(limits),
        )?)
    }
    /// Select an extension language without changing the built-in source detector.
    pub fn parse_as(
        &self,
        id: &DiagramId,
        source: &str,
        options: &ParseOptions,
    ) -> Result<PublicDocument, PeleError> {
        self.parse_plan(self.engine.prepare_parse(
            source,
            options,
            Some(id),
            ParseAdmission::Legacy,
        )?)
    }
    /// Select an extension while keeping the public byte budget and exact
    /// source comparison, without changing the extension's ParseContext shape.
    pub fn parse_as_with_source_limit(
        &self,
        id: &DiagramId,
        source: &str,
        options: &ParseOptions,
        limit: SourceLimit,
    ) -> Result<PublicDocument, PeleError> {
        self.parse_plan(self.engine.prepare_parse(
            source,
            options,
            Some(id),
            ParseAdmission::Source(limit),
        )?)
    }
    /// Schema for an explicitly selected registered provider, without parsing.
    pub fn schema_for(&self, id: &DiagramId) -> Result<ModelSchemaView, PeleError> {
        let registration = self.graph_registration(id)?;
        Ok(ModelSchemaView {
            model: registration.schema,
            shared_functions: registration.shared_functions,
            function_shapes: registration.function_shapes,
        })
    }
    fn graph_registration(&self, id: &DiagramId) -> Result<Arc<Registration>, PeleError> {
        self.modules
            .iter()
            .find(|m| m.descriptor.id == *id)
            .cloned()
            .ok_or_else(|| {
                let mut error = PeleError::new(
                    format!("Diagram type \"{id}\" is not supported yet."),
                    ErrorCode::UnsupportedDiagram,
                );
                error.diagram_type = Some(id.to_string());
                error
            })
    }
    /// Construct an owned native model from supplied graph data. Registration
    /// selection is explicit and remains retained across replacement or drop.
    pub fn create_from_graph_as(
        &self,
        id: &DiagramId,
        graph: ModelGraph,
        config: Object,
    ) -> Result<PublicDocument, PeleError> {
        let registration = self.graph_registration(id)?;
        registration.validate_graph(&graph)?;
        let state = registration.codec.create_graph(graph)?;
        Ok(PublicDocument {
            registration,
            state,
            config,
            revision: 0,
        })
    }
    fn parse_plan(&self, plan: ParsePlan) -> Result<PublicDocument, PeleError> {
        let registration = self
            .modules
            .iter()
            .find(|module| module.descriptor.id == *plan.diagram_type())
            .expect("paired public registration")
            .clone();
        match registration
            .codec
            .try_parse_public(plan.context(), plan.edge_limit())
        {
            Some(result) => {
                let state = result?;
                ModelSchemaView {
                    model: registration.schema,
                    shared_functions: registration.shared_functions,
                    function_shapes: registration.function_shapes,
                }
                .validate()?;
                Ok(PublicDocument {
                    registration,
                    state,
                    config: plan.into_config(),
                    revision: 0,
                })
            }
            None => self.create_document(plan.parse()?),
        }
    }
    fn create_document(
        &self,
        document: crate::compat::ParsedDocument,
    ) -> Result<PublicDocument, PeleError> {
        let registration = self
            .modules
            .iter()
            .find(|m| m.descriptor.id == *document.diagram_type())
            .expect("paired public registration")
            .clone();
        ModelSchemaView {
            model: registration.schema,
            shared_functions: registration.shared_functions,
            function_shapes: registration.function_shapes,
        }
        .validate()?;
        let state = registration.codec.create(&document)?;
        Ok(PublicDocument {
            registration,
            state,
            config: document.config().clone(),
            revision: 0,
        })
    }
}
/// Raw diagram render outcome; dispatched partial mutations are always published.
pub struct RawRenderOutcome {
    pub revision: u64,
    pub graph: ModelGraph,
    pub result: Result<Rendered, PublicError>,
}
/// One serialized mutable operation stream. This type is deliberately not Sync.
pub struct PublicDocument {
    registration: Arc<Registration>,
    state: State,
    config: Object,
    revision: u64,
}
impl PublicDocument {
    /// Adapt an existing typed model without parsing its source again.
    /// The document owns the exact module and effective config supplied here.
    pub fn from_module<D: PublicDiagramModule>(
        module: D,
        model: &D::Model,
        config: Object,
    ) -> Result<Self, PeleError> {
        let descriptor = module.descriptor();
        let schema = module.model_schema();
        let shared_functions = module.shared_functions();
        let function_shapes = module.function_shapes();
        ModelSchemaView {
            model: schema,
            shared_functions,
            function_shapes,
        }
        .validate()?;
        let state: State = Box::new(module.create_public(model)?);
        Ok(Self {
            registration: Arc::new(Registration {
                descriptor,
                schema,
                compiled_schema: OnceLock::new(),
                shared_functions,
                function_shapes,
                codec: Box::new(Codec(Arc::new(module))),
            }),
            state,
            config,
            revision: 0,
        })
    }
    /// Construct a concrete provider directly from an arena, without parsing.
    pub fn from_graph<D: PublicDiagramModule>(
        module: D,
        graph: ModelGraph,
        config: Object,
    ) -> Result<Self, PeleError> {
        let descriptor = module.descriptor();
        let schema = module.model_schema();
        let shared_functions = module.shared_functions();
        let function_shapes = module.function_shapes();
        ModelSchemaView {
            model: schema,
            shared_functions,
            function_shapes,
        }
        .validate()?;
        graph.validate_schema(schema)?;
        let state: State = Box::new(module.create_public_from_graph(graph)?);
        Ok(Self {
            registration: Arc::new(Registration {
                descriptor,
                schema,
                compiled_schema: OnceLock::new(),
                shared_functions,
                function_shapes,
                codec: Box::new(Codec(Arc::new(module))),
            }),
            state,
            config,
            revision: 0,
        })
    }
    /// Effective parse configuration owned by this retained document.
    pub fn retained_config(&self) -> &Object {
        &self.config
    }
    pub fn diagram_type(&self) -> &DiagramId {
        &self.registration.descriptor.id
    }
    pub fn model_schema(&self) -> &'static ModelSchema {
        self.registration.schema
    }
    pub fn schema_view(&self) -> ModelSchemaView {
        ModelSchemaView {
            model: self.registration.schema,
            shared_functions: self.registration.shared_functions,
            function_shapes: self.registration.function_shapes,
        }
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn unchanged_public_graph_is_noop(&self) -> bool {
        self.registration.codec.unchanged_public_graph_is_noop()
    }
    fn graph_view(&self) -> Result<Cow<'_, ModelGraph>, PeleError> {
        match self.registration.codec.borrow_graph(&self.state) {
            Some(graph) => Ok(Cow::Borrowed(graph)),
            None => self.registration.codec.graph(&self.state).map(Cow::Owned),
        }
    }
    /// A validated view, borrowed when the retained module supports it.
    /// The document cannot mutate while this view is retained. Extensions that
    /// only implement owned export retain that export and its original errors.
    pub fn snapshot_view(&self) -> Result<Cow<'_, ModelGraph>, PeleError> {
        let graph = self.graph_view()?;
        self.registration.validate_graph(&graph)?;
        Ok(graph)
    }
    pub fn snapshot(&self) -> Result<ModelGraph, PeleError> {
        self.snapshot_view().map(Cow::into_owned)
    }
    pub fn apply(&mut self, expected_revision: u64, graph: ModelGraph) -> Result<(), PeleError> {
        if expected_revision != self.revision {
            return Err(PeleError::new(
                "Stale public model revision",
                ErrorCode::Semantic,
            ));
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| PeleError::new("Public model revision exhausted", ErrorCode::Limit))?;
        self.registration.validate_graph(&graph)?;
        self.registration.codec.apply(&mut self.state, graph)?;
        self.revision = next_revision;
        Ok(())
    }
    /// Dispatch through the retained module, then publish every prior mutation
    /// even if the operation returns a native, JavaScript or host exception.
    /// Stale/malformed calls never reach module code. A codec that exports an
    /// invalid graph is a protocol error; its state and advanced revision remain.
    pub fn invoke(
        &mut self,
        expected_revision: u64,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<InvocationOutcome, PeleError> {
        if expected_revision != self.revision {
            return Err(PeleError::new(
                "Stale public model revision",
                ErrorCode::Semantic,
            ));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or_else(|| PeleError::new("Public model revision exhausted", ErrorCode::Limit))?;
        {
            let before = self.graph_view()?;
            call.validate_compiled(&before, self.registration.compiled_schema()?)?;
        }
        let mut result = self
            .registration
            .codec
            .invoke(&mut self.state, call, &self.config, host);
        self.revision = next;
        let graph = self.snapshot()?;
        // A malformed native return is an operation protocol error, while its
        // successful partial mutations still publish in the same outcome.
        if let Ok(atom) = &result {
            let checked = graph.validate_atom(atom).and_then(|_| {
                if let Atom::BuiltinFunction(id) = atom {
                    if self
                        .registration
                        .schema
                        .method(super::MethodId(*id))
                        .is_none()
                    {
                        return Err(PeleError::new(
                            "Unknown public builtin function",
                            ErrorCode::Semantic,
                        ));
                    }
                }
                Ok(())
            });
            if let Err(e) = checked {
                result = Err(e.into());
            }
        }
        Ok(InvocationOutcome {
            revision: next,
            graph,
            result,
        })
    }
    /// Render with the retained low-level handler and supplied config, without
    /// core defaults or semantic output policy. Independent byte bounds remain.
    pub fn render_raw_operation(
        &mut self,
        expected_revision: u64,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<RawRenderOutcome, PeleError> {
        if expected_revision != self.revision {
            return Err(PeleError::new(
                "Stale public model revision",
                ErrorCode::Semantic,
            ));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or_else(|| PeleError::new("Public model revision exhausted", ErrorCode::Limit))?;
        self.snapshot_view()?;
        let mut operation_error = None;
        let result = crate::compat::engine::render_raw_registered(
            &self.registration.descriptor,
            settings,
            host,
            |session| {
                self.registration
                    .codec
                    .render_raw_operation(&mut self.state, config, settings, session)
                    .map_err(|error| {
                        operation_error = Some(error);
                        PeleError::new("Public raw render operation failed", ErrorCode::Semantic)
                    })
            },
        )
        .map_err(|error| operation_error.unwrap_or_else(|| error.into()));
        self.revision = next;
        let graph = self.snapshot()?;
        Ok(RawRenderOutcome {
            revision: next,
            graph,
            result,
        })
    }
    /// Render through the retained handler, publishing post-render state on
    /// success or operation error. Stale revisions reject before dispatch.
    pub fn render_operation(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
    ) -> Result<RenderOutcome, PeleError> {
        self.render_operation_policy(expected_revision, options, host, None, false)
    }
    /// Exact output comparison after rendering. Dispatched mutations/revision
    /// publish on limit and host errors, under the existing graph export budgets.
    pub fn render_operation_with_output_limit(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        limit: OutputLimit,
        host: &mut dyn RenderResources,
    ) -> Result<RenderOutcome, PeleError> {
        self.render_operation_policy(expected_revision, options, host, Some(limit), true)
    }
    /// Additive native lane: bound returned SVG bytes even when the caller keeps
    /// legacy integer output semantics. Dispatched errors still publish state.
    pub fn render_operation_bounded(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        output_limit: Option<OutputLimit>,
        host: &mut dyn RenderResources,
    ) -> Result<RenderOutcome, PeleError> {
        self.render_operation_policy(expected_revision, options, host, output_limit, true)
    }
    fn render_operation_policy(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
        limit: Option<OutputLimit>,
        enforce_native_bytes: bool,
    ) -> Result<RenderOutcome, PeleError> {
        if expected_revision != self.revision {
            return Err(PeleError::new(
                "Stale public model revision",
                ErrorCode::Semantic,
            ));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or_else(|| PeleError::new("Public model revision exhausted", ErrorCode::Limit))?;
        self.snapshot_view()?;
        let mut operation_error = None;
        let mut presentation = crate::compat::PresentationCapture::deferred(host);
        let result = crate::compat::engine::render_registered_policy(
            &self.registration.descriptor,
            &self.config,
            options,
            &mut presentation,
            limit,
            enforce_native_bytes,
            |settings, session| {
                let codec = &self.registration.codec;
                let result = match limit {
                    Some(limit) => codec.render_operation_with_output_limit(
                        &mut self.state,
                        &self.config,
                        settings,
                        session,
                        limit,
                    ),
                    None => {
                        codec.render_operation(&mut self.state, &self.config, settings, session)
                    }
                };
                result.map_err(|error| {
                    operation_error = Some(error);
                    // The orchestration layer only needs to stop. Preserve
                    // the real typed error outside its legacy error lane.
                    PeleError::new("Public render operation failed", ErrorCode::Semantic)
                })
            },
        )
        .map_err(|error| operation_error.unwrap_or_else(|| error.into()));
        self.revision = next;
        let graph = self.snapshot()?;
        presentation.publish();
        Ok(RenderOutcome {
            revision: next,
            graph,
            result,
        })
    }
    pub fn render(
        &mut self,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        crate::compat::engine::render_registered(
            &self.registration.descriptor,
            &self.config,
            options,
            host,
            |settings, session| {
                self.registration
                    .codec
                    .render(&mut self.state, &self.config, settings, session)
            },
        )
    }
}
