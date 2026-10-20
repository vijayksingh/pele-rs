use super::graph_read::{GraphReadView, PublicGraphRead, ReadAdapter};
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
    any::{Any, TypeId},
    borrow::Cow,
    sync::{Arc, OnceLock},
};
/// Owned output of a captured parser, with its exact public-construction policy.
/// The engine alone creates this carrier. Model type or schema identity does not
/// certify which module parsed it; delegates must check concrete provenance.
pub struct ParsedPublicModel<M> {
    model: M,
    creator: TypeId,
    prepared: bool,
    edge_limit: Option<EdgeLimit>,
}
impl<M> ParsedPublicModel<M> {
    pub fn model(&self) -> &M {
        &self.model
    }
    pub fn is_prepared(&self) -> bool {
        self.prepared
    }
    pub fn edge_limit(&self) -> Option<EdgeLimit> {
        self.edge_limit
    }
    /// True only for the concrete module paired with the captured parser.
    pub fn is_from_module<D: DiagramModule>(&self) -> bool {
        self.creator == TypeId::of::<D>()
    }
    pub fn into_model(self) -> M {
        self.model
    }
    /// Preserve the existing borrowed construction hooks and their dispatch order.
    pub fn create_legacy<D>(self, module: &D) -> Result<D::PublicState, PeleError>
    where
        D: PublicDiagramModule<Model = M> + ?Sized,
    {
        if self.prepared {
            module.create_public_prepared(&self.model, self.edge_limit.expect("prepared policy"))
        } else if let Some(limit) = self.edge_limit {
            module.create_public_with_edge_limit(&self.model, limit)
        } else {
            module.create_public(&self.model)
        }
    }
}
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
    /// Consume a captured parser result. The default preserves all existing
    /// borrowed hooks. Opt-ins must preserve eager projection admission and
    /// normalization; arbitrary typed adaptation never calls this hook.
    fn create_public_parsed(
        &self,
        parsed: ParsedPublicModel<Self::Model>,
    ) -> Result<Self::PublicState, PeleError> {
        parsed.create_legacy(self)
    }
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
    fn has_source_layout(&self, _state: &Self::PublicState) -> bool {
        false
    }
    /// Additive logical export. Existing providers retain their original export path.
    fn borrow_public_read<'a>(
        &self,
        _state: &'a Self::PublicState,
    ) -> Option<&'a dyn PublicGraphRead> {
        None
    }
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
    #[cfg(feature = "web-host-er")]
    fn is_host_er(&self) -> bool;
    #[cfg(feature = "web-host-sankey")]
    fn is_host_sankey(&self) -> bool;
    fn try_parse_public(
        &self,
        context: ParseContext<'_>,
        edge_limit: Option<EdgeLimit>,
    ) -> Option<Result<State, PeleError>>;
    fn create_owned(
        &self,
        doc: crate::compat::ParsedDocument,
    ) -> Result<(State, Object), PeleError>;
    fn create_graph(&self, graph: ModelGraph) -> Result<State, PeleError>;
    fn graph(&self, state: &State) -> Result<ModelGraph, PeleError>;
    fn borrow_graph<'a>(&self, state: &'a State) -> Option<&'a ModelGraph>;
    fn has_source_layout(&self, state: &State) -> bool;
    fn borrow_read<'a>(&self, state: &'a State) -> Option<&'a dyn PublicGraphRead>;
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
    #[cfg(feature = "web-host-sankey")]
    fn is_host_sankey(&self) -> bool {
        TypeId::of::<D>() == TypeId::of::<crate::compat::diagrams::sankey::Sankey>()
    }

    #[cfg(feature = "web-host-er")]
    fn is_host_er(&self) -> bool {
        std::any::TypeId::of::<D>() == std::any::TypeId::of::<crate::compat::diagrams::er::Er>()
    }
    fn try_parse_public(
        &self,
        context: ParseContext<'_>,
        edge_limit: Option<EdgeLimit>,
    ) -> Option<Result<State, PeleError>> {
        self.0
            .try_parse_public(context, edge_limit)
            .map(|result| result.map(|state| Box::new(state) as State))
    }
    fn create_owned(
        &self,
        doc: crate::compat::ParsedDocument,
    ) -> Result<(State, Object), PeleError> {
        doc.into_public_state::<D::Model, _>(|model, prepared, edge_limit| {
            self.0
                .create_public_parsed(ParsedPublicModel {
                    model,
                    creator: TypeId::of::<D>(),
                    prepared,
                    edge_limit,
                })
                .map(|state| Box::new(state) as State)
        })
    }
    fn create_graph(&self, graph: ModelGraph) -> Result<State, PeleError> {
        Ok(Box::new(self.0.create_public_from_graph(graph)?))
    }
    fn graph(&self, s: &State) -> Result<ModelGraph, PeleError> {
        self.0
            .export_public(s.downcast_ref().expect("paired public state"))
    }
    fn has_source_layout(&self, s: &State) -> bool {
        self.0
            .has_source_layout(s.downcast_ref().expect("paired public state"))
    }
    fn borrow_read<'a>(&self, s: &'a State) -> Option<&'a dyn PublicGraphRead> {
        self.0
            .borrow_public_read(s.downcast_ref().expect("paired public state"))
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
    fn validate_read(&self, graph: &GraphReadView<'_>) -> Result<(), PeleError> {
        match graph {
            GraphReadView::Arena(g) => self.validate_graph(g),
            GraphReadView::Logical(g) => {
                let schema = self.compiled_schema()?;
                let mut read = ReadAdapter(*g);
                super::captured_admission::validate(&mut read)?;
                super::captured_admission::validate_schema_ids(
                    &mut read,
                    schema.methods(),
                    schema.prototypes(),
                )
            }
        }
    }
    fn validate_graph(&self, graph: &ModelGraph) -> Result<(), PeleError> {
        graph.validate_compiled(self.compiled_schema()?)
    }
}
/// Captured builtin-only raw entry policy. Contains no host values or model.
#[cfg(feature = "web-host-er")]
#[doc(hidden)]
pub struct HostErEntrySelection {
    registration: Arc<Registration>,
}
#[cfg(feature = "web-host-er")]
impl HostErEntrySelection {
    pub fn schema(&self) -> ModelSchemaView {
        ModelSchemaView {
            model: self.registration.schema,
            shared_functions: self.registration.shared_functions,
            function_shapes: self.registration.function_shapes,
        }
    }
    /// Called after the adapter's structural admission and config decoding.
    /// No registration lookup or drawable import occurs at this boundary.
    pub fn bind(&self, config: Object) -> Result<crate::compat::host_er::Policy, PeleError> {
        self.registration.compiled_schema()?;
        Ok(crate::compat::host_er::Policy::captured(
            self.registration.descriptor.clone(),
            config,
        ))
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
    /// Facade-only prepared publication. Input is parsed verbatim once; the
    /// low-level prepared API continues to return a public arena document.
    #[cfg(feature = "web-host-er")]
    #[doc(hidden)]
    pub fn parse_prepared_host_er_as<W: crate::compat::host_er::InitialWriter>(
        &self,
        id: &DiagramId,
        context: ParseContext<'_>,
        edge_limit: EdgeLimit,
        make: impl FnOnce(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_er::Parsed<W::Output>, PeleError> {
        let plan = self.engine.prepare_prepared_as(id, context, edge_limit)?;
        let registration = self
            .modules
            .iter()
            .find(|module| module.descriptor.id == *plan.diagram_type())
            .expect("paired public registration")
            .clone();
        let document = plan.parse()?;
        if !registration.codec.is_host_er() {
            return Self::create_document_registered(document, registration)
                .map(crate::compat::host_er::Parsed::Arena);
        }
        let schema = ModelSchemaView {
            model: registration.schema,
            shared_functions: registration.shared_functions,
            function_shapes: registration.function_shapes,
        };
        schema.validate()?;
        let model = document
            .model::<crate::compat::diagrams::er::ErModel>()
            .expect("captured builtin ER model");
        crate::compat::diagrams::er::admit_host_initial(model)?;
        let output = crate::compat::diagrams::er::publish_host_initial(make(schema)?, model)?;
        let policy = crate::compat::host_er::Policy::captured(
            registration.descriptor.clone(),
            document.config().clone(),
        );
        Ok(crate::compat::host_er::Parsed::Host { policy, output })
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
    /// Private browser adoption selects by captured concrete codec identity.
    #[cfg(feature = "web-host-er")]
    #[doc(hidden)]
    pub fn select_host_er_entry(
        &self,
        id: &DiagramId,
    ) -> Result<Option<HostErEntrySelection>, PeleError> {
        let registration = self.graph_registration(id)?;
        Ok(registration
            .codec
            .is_host_er()
            .then_some(HostErEntrySelection { registration }))
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
    /// Captured builtin-only browser construction; fallback consumes the same
    /// prepared plan once. Existing low-level parse methods remain arena-based.
    #[cfg(feature = "web-host-er")]
    #[doc(hidden)]
    pub fn parse_host_er_with_limits<W: crate::compat::host_er::InitialWriter>(
        &self,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
        make: impl FnMut(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_er::Parsed<W::Output>, PeleError> {
        let plan =
            self.engine
                .prepare_parse(source, options, None, ParseAdmission::Limits(limits))?;
        let registration = self
            .modules
            .iter()
            .find(|module| module.descriptor.id == *plan.diagram_type())
            .expect("paired public registration")
            .clone();
        if !registration.codec.is_host_er() {
            return self
                .parse_plan(plan)
                .map(crate::compat::host_er::Parsed::Arena);
        }
        Self::parse_host_er_plan(plan, registration, make)
    }
    #[cfg(feature = "web-host-er")]
    fn parse_host_er_plan<W: super::InitialWriter>(
        plan: ParsePlan,
        registration: Arc<Registration>,
        mut make: impl FnMut(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_er::Parsed<W::Output>, PeleError> {
        let schema = ModelSchemaView {
            model: registration.schema,
            shared_functions: registration.shared_functions,
            function_shapes: registration.function_shapes,
        };
        let writer = make(schema)?;
        match crate::compat::diagrams::er::parse_host_into(plan.context(), writer) {
            Some(result) => {
                let output = result?;
                schema.validate()?;
                let policy = crate::compat::host_er::Policy::captured(
                    registration.descriptor.clone(),
                    plan.into_config(),
                );
                Ok(crate::compat::host_er::Parsed::Host { policy, output })
            }
            None => {
                // Direct construction has discarded its unpublished owner. Keep
                // the captured plan and run the existing grammar exactly once.
                let document = plan.parse()?;
                schema.validate()?;
                let model = document
                    .model::<crate::compat::diagrams::er::ErModel>()
                    .expect("captured builtin ER model");
                // Preserve legacy GraphWriter/facade admission before host
                // primitives can reject a partial large publication earlier.
                crate::compat::diagrams::er::admit_host_initial(model)?;
                let output =
                    crate::compat::diagrams::er::publish_host_initial(make(schema)?, model)?;
                let policy = crate::compat::host_er::Policy::captured(
                    registration.descriptor.clone(),
                    document.config().clone(),
                );
                // The typed model is transient; neither the policy nor output
                // retains it, a public graph, or a later conversion obligation.
                Ok(crate::compat::host_er::Parsed::Host { policy, output })
            }
        }
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
        Self::create_document_registered(document, registration)
    }
    fn create_document_registered(
        document: crate::compat::ParsedDocument,
        registration: Arc<Registration>,
    ) -> Result<PublicDocument, PeleError> {
        ModelSchemaView {
            model: registration.schema,
            shared_functions: registration.shared_functions,
            function_shapes: registration.function_shapes,
        }
        .validate()?;
        let (state, config) = registration.codec.create_owned(document)?;
        Ok(PublicDocument {
            registration,
            state,
            config,
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
/// Validated post-render graph view. The graph stays borrowed only when the
/// captured module supports borrowed export; extension fallback remains owned.
#[derive(Debug)]
pub struct RenderOutcomeView<'a> {
    pub revision: u64,
    pub graph: Cow<'a, ModelGraph>,
    pub result: Result<RenderResult, PublicError>,
}
impl RenderOutcomeView<'_> {
    fn into_owned(self) -> RenderOutcome {
        RenderOutcome {
            revision: self.revision,
            graph: self.graph.into_owned(),
            result: self.result,
        }
    }
}
/// Raw low-level render equivalent of RenderOutcomeView.
#[derive(Debug)]
pub struct RawRenderOutcomeView<'a> {
    pub revision: u64,
    pub graph: Cow<'a, ModelGraph>,
    pub result: Result<Rendered, PublicError>,
}
impl RawRenderOutcomeView<'_> {
    fn into_owned(self) -> RawRenderOutcome {
        RawRenderOutcome {
            revision: self.revision,
            graph: self.graph.into_owned(),
            result: self.result,
        }
    }
}
/// Validated logical post-state. Its borrow is independent of resources.
#[derive(Debug)]
pub struct ReadOutcome<'a, T> {
    pub revision: u64,
    pub graph: GraphReadView<'a>,
    pub result: Result<T, PublicError>,
}
impl<'a> ReadOutcome<'a, RenderResult> {
    fn into_render_view(self) -> Result<RenderOutcomeView<'a>, PeleError> {
        Ok(RenderOutcomeView {
            revision: self.revision,
            graph: self.graph.into_model()?,
            result: self.result,
        })
    }
}
impl<'a> ReadOutcome<'a, Rendered> {
    fn into_raw_view(self) -> Result<RawRenderOutcomeView<'a>, PeleError> {
        Ok(RawRenderOutcomeView {
            revision: self.revision,
            graph: self.graph.into_model()?,
            result: self.result,
        })
    }
}
#[derive(Clone, Copy)]
enum GraphExport {
    Owned,
    Borrowed,
    Logical,
}
impl GraphExport {
    fn finish(self, graph: GraphReadView<'_>) -> Result<GraphReadView<'_>, PeleError> {
        match self {
            Self::Logical => Ok(graph),
            Self::Borrowed => Ok(GraphReadView::Arena(graph.into_model()?)),
            Self::Owned => Ok(GraphReadView::Arena(Cow::Owned(
                graph.into_model()?.into_owned(),
            ))),
        }
    }
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
        self.snapshot_read()?.into_model()
    }
    pub fn has_source_layout(&self) -> bool {
        self.registration.codec.has_source_layout(&self.state)
    }
    fn read_view(&self) -> Result<GraphReadView<'_>, PeleError> {
        Ok(match self.registration.codec.borrow_read(&self.state) {
            Some(g) => GraphReadView::Logical(g),
            None => GraphReadView::Arena(self.graph_view()?),
        })
    }
    pub fn snapshot_read(&self) -> Result<GraphReadView<'_>, PeleError> {
        let graph = self.read_view()?;
        self.registration.validate_read(&graph)?;
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
        let outcome = self.invoke_policy(expected_revision, call, host, GraphExport::Owned)?;
        Ok(InvocationOutcome {
            revision: outcome.revision,
            graph: outcome.graph.into_model()?.into_owned(),
            result: outcome.result,
        })
    }
    pub fn invoke_read(
        &mut self,
        expected_revision: u64,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<ReadOutcome<'_, Atom>, PeleError> {
        self.invoke_policy(expected_revision, call, host, GraphExport::Logical)
    }
    fn invoke_policy(
        &mut self,
        expected_revision: u64,
        call: MethodCall,
        host: &mut dyn RenderResources,
        export: GraphExport,
    ) -> Result<ReadOutcome<'_, Atom>, PeleError> {
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
            let before = self.read_view()?;
            match &before {
                GraphReadView::Arena(graph) => {
                    call.validate_compiled(graph, self.registration.compiled_schema()?)?
                }
                GraphReadView::Logical(_) => {
                    self.registration.validate_read(&before)?;
                    super::schema::validate_call_after_graph(
                        &call,
                        before.reader().extent(),
                        self.registration.compiled_schema()?.methods(),
                    )?;
                }
            }
        }
        let mut result = self
            .registration
            .codec
            .invoke(&mut self.state, call, &self.config, host);
        self.revision = next;
        let graph = export.finish(self.snapshot_read()?)?;
        // A malformed native return is an operation protocol error, while its
        // successful partial mutations still publish in the same outcome.
        if let Ok(atom) = &result {
            let checked =
                super::captured_admission::atom(graph.reader().extent(), atom).and_then(|_| {
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
        Ok(ReadOutcome {
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
        self.render_raw_operation_policy(
            expected_revision,
            config,
            settings,
            host,
            GraphExport::Owned,
        )
        .and_then(ReadOutcome::into_raw_view)
        .map(RawRenderOutcomeView::into_owned)
    }
    /// Borrow the validated post-state without changing dispatch or revision order.
    pub fn render_raw_operation_view(
        &mut self,
        expected_revision: u64,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<RawRenderOutcomeView<'_>, PeleError> {
        self.render_raw_operation_policy(
            expected_revision,
            config,
            settings,
            host,
            GraphExport::Borrowed,
        )
        .and_then(ReadOutcome::into_raw_view)
    }
    pub fn render_raw_operation_read(
        &mut self,
        revision: u64,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<ReadOutcome<'_, Rendered>, PeleError> {
        self.render_raw_operation_policy(revision, config, settings, host, GraphExport::Logical)
    }
    fn render_raw_operation_policy(
        &mut self,
        expected_revision: u64,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
        export: GraphExport,
    ) -> Result<ReadOutcome<'_, Rendered>, PeleError> {
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
        self.snapshot_read()?;
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
        let graph = export.finish(self.snapshot_read()?)?;
        Ok(ReadOutcome {
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
        self.render_operation_policy(
            expected_revision,
            options,
            host,
            None,
            false,
            GraphExport::Owned,
        )
        .and_then(ReadOutcome::into_render_view)
        .map(RenderOutcomeView::into_owned)
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
        self.render_operation_policy(
            expected_revision,
            options,
            host,
            Some(limit),
            true,
            GraphExport::Owned,
        )
        .and_then(ReadOutcome::into_render_view)
        .map(RenderOutcomeView::into_owned)
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
        self.render_operation_policy(
            expected_revision,
            options,
            host,
            output_limit,
            true,
            GraphExport::Owned,
        )
        .and_then(ReadOutcome::into_render_view)
        .map(RenderOutcomeView::into_owned)
    }
    /// Legacy output policy with a validated borrowed post-state.
    pub fn render_operation_view(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
    ) -> Result<RenderOutcomeView<'_>, PeleError> {
        self.render_operation_policy(
            expected_revision,
            options,
            host,
            None,
            false,
            GraphExport::Borrowed,
        )
        .and_then(ReadOutcome::into_render_view)
    }
    /// Exact output policy with a validated borrowed post-state.
    pub fn render_operation_with_output_limit_view(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        limit: OutputLimit,
        host: &mut dyn RenderResources,
    ) -> Result<RenderOutcomeView<'_>, PeleError> {
        self.render_operation_policy(
            expected_revision,
            options,
            host,
            Some(limit),
            true,
            GraphExport::Borrowed,
        )
        .and_then(ReadOutcome::into_render_view)
    }
    /// Bounded rendering with a validated borrowed post-state. Existing owned
    /// APIs still materialize their graph before publishing presentation.
    pub fn render_operation_bounded_view(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        output_limit: Option<OutputLimit>,
        host: &mut dyn RenderResources,
    ) -> Result<RenderOutcomeView<'_>, PeleError> {
        self.render_operation_policy(
            expected_revision,
            options,
            host,
            output_limit,
            true,
            GraphExport::Borrowed,
        )
        .and_then(ReadOutcome::into_render_view)
    }
    pub fn render_operation_read(
        &mut self,
        revision: u64,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
    ) -> Result<ReadOutcome<'_, RenderResult>, PeleError> {
        self.render_operation_policy(revision, options, host, None, false, GraphExport::Logical)
    }
    pub fn render_operation_with_output_limit_read(
        &mut self,
        revision: u64,
        options: &RenderOptions,
        limit: OutputLimit,
        host: &mut dyn RenderResources,
    ) -> Result<ReadOutcome<'_, RenderResult>, PeleError> {
        self.render_operation_policy(
            revision,
            options,
            host,
            Some(limit),
            true,
            GraphExport::Logical,
        )
    }
    pub fn render_operation_bounded_read(
        &mut self,
        revision: u64,
        options: &RenderOptions,
        limit: Option<OutputLimit>,
        host: &mut dyn RenderResources,
    ) -> Result<ReadOutcome<'_, RenderResult>, PeleError> {
        self.render_operation_policy(revision, options, host, limit, true, GraphExport::Logical)
    }
    fn render_operation_policy(
        &mut self,
        expected_revision: u64,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
        limit: Option<OutputLimit>,
        enforce_native_bytes: bool,
        export: GraphExport,
    ) -> Result<ReadOutcome<'_, RenderResult>, PeleError> {
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
        self.snapshot_read()?;
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
        // The owned policy clones here, at the original snapshot stage. Its
        // wrapper only moves that already owned graph after publication.
        let graph = export.finish(self.snapshot_read()?)?;
        presentation.publish();
        Ok(ReadOutcome {
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

#[cfg(feature = "web-host-sankey")]
#[derive(Clone)]
pub(crate) struct CapturedSankeyRegistration(Arc<Registration>);
#[cfg(feature = "web-host-sankey")]
impl CapturedSankeyRegistration {
    pub(crate) fn descriptor(&self) -> &ModuleDescriptor {
        &self.0.descriptor
    }
    pub(crate) fn schema(&self) -> ModelSchemaView {
        ModelSchemaView {
            model: self.0.schema,
            shared_functions: self.0.shared_functions,
            function_shapes: self.0.function_shapes,
        }
    }
    pub(crate) fn validate<S: super::captured_admission::AdmissionRead>(
        &self,
        store: &mut S,
    ) -> Result<(), PeleError> {
        let schema = self.0.compiled_schema()?;
        super::captured_admission::validate(store)?;
        super::captured_admission::validate_schema_ids(store, schema.methods(), schema.prototypes())
    }
    pub(crate) fn validate_uncached<S: super::captured_admission::AdmissionRead>(
        &self,
        store: &mut S,
    ) -> Result<(), PeleError> {
        self.0.schema.validate()?;
        super::captured_admission::validate(store)?;
        let methods = self.0.schema.methods.iter().map(|m| m.id).collect();
        let prototypes = self.0.schema.prototypes.iter().map(|p| p.id).collect();
        super::captured_admission::validate_schema_ids(store, &methods, &prototypes)
    }
    pub(crate) fn validate_call<S: super::captured_admission::AdmissionRead>(
        &self,
        store: &mut S,
        call: &MethodCall,
    ) -> Result<(), PeleError> {
        self.validate(store)?;
        super::schema::validate_call_after_graph(
            call,
            store.extent()?,
            self.0.compiled_schema()?.methods(),
        )
    }
    pub(crate) fn validate_return<S: super::captured_admission::AdmissionRead>(
        &self,
        store: &mut S,
        value: &Atom,
    ) -> Result<(), PeleError> {
        super::captured_admission::atom(store.extent()?, value)?;
        if let Atom::BuiltinFunction(id) = value {
            if self.0.schema.method(super::MethodId(*id)).is_none() {
                return Err(PeleError::new(
                    "Unknown public builtin function",
                    ErrorCode::Semantic,
                ));
            }
        }
        Ok(())
    }
}

/// Captured concrete Sankey policy; registry replacement never reselects it.
#[cfg(feature = "web-host-sankey")]
#[derive(Clone)]
#[doc(hidden)]
pub struct HostSankeyEntrySelection {
    registration: CapturedSankeyRegistration,
}
#[cfg(feature = "web-host-sankey")]
impl HostSankeyEntrySelection {
    pub fn schema(&self) -> ModelSchemaView {
        self.registration.schema()
    }
    pub fn bind<
        S: super::captured_read::CapturedRead + super::captured_admission::AdmissionRead,
    >(
        &self,
        config: Object,
        store: &mut S,
    ) -> Result<crate::compat::host_sankey::Document, PeleError> {
        // Registry create_from_graph admission, then the concrete codec's own
        // validate_schema before its eager (possibly latent-error) projection.
        self.registration.validate(store)?;
        self.registration.validate_uncached(store)?;
        let model = crate::compat::diagrams::sankey::imported_from(store);
        Ok(crate::compat::host_sankey::Document::captured(
            self.registration.clone(),
            config,
            model,
        ))
    }
}

#[cfg(feature = "web-host-sankey")]
impl PublicEngine {
    #[doc(hidden)]
    pub fn select_host_sankey_entry(
        &self,
        id: &DiagramId,
    ) -> Result<Option<HostSankeyEntrySelection>, PeleError> {
        let registration = self.graph_registration(id)?;
        Ok(registration
            .codec
            .is_host_sankey()
            .then(|| HostSankeyEntrySelection {
                registration: CapturedSankeyRegistration(registration),
            }))
    }
    fn create_host_sankey<W>(
        document: crate::compat::ParsedDocument,
        registration: Arc<Registration>,
        make: impl FnOnce(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_sankey::Parsed<W::Output>, PeleError>
    where
        W: super::InitialWriter,
        W::Output: super::captured_read::CapturedRead + super::captured_admission::AdmissionRead,
    {
        let policy = CapturedSankeyRegistration(registration);
        let schema = policy.schema();
        schema.validate()?;
        let ((storage, model), config) = document
            .into_public_state::<crate::compat::diagrams::sankey::SankeyModel, _>(
                |model, prepared, _edge| {
                    let mut storage =
                        crate::compat::diagrams::sankey::initial_into(make(schema)?, &model)?;
                    // Independent native admission at the original GraphWriter.finish
                    // boundary. A sink's finish() is never a validation certificate.
                    super::captured_admission::validate(&mut storage)?;
                    let projection = if prepared {
                        crate::compat::diagrams::sankey::imported_from(&mut storage)
                    } else {
                        crate::compat::diagrams::sankey::normalized_source(model)
                    };
                    Ok((storage, projection))
                },
            )?;
        Ok(crate::compat::host_sankey::Parsed::Captured {
            document: crate::compat::host_sankey::Document::captured(policy, config, model),
            storage,
        })
    }
    /// One source preparation selects the captured ER, captured Sankey, or
    /// ordinary arena route. A declined family never triggers a second parse.
    #[cfg(feature = "web-host-er")]
    #[doc(hidden)]
    pub fn parse_host_storage_with_limits<E, S>(
        &self,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
        make_er: impl FnMut(ModelSchemaView) -> Result<E, PeleError>,
        make_sankey: impl FnOnce(ModelSchemaView) -> Result<S, PeleError>,
    ) -> Result<crate::compat::host_sankey::CombinedParsed<E::Output, S::Output>, PeleError>
    where
        E: super::InitialWriter,
        S: super::InitialWriter,
        S::Output: super::captured_read::CapturedRead + super::captured_admission::AdmissionRead,
    {
        use crate::compat::{host_er, host_sankey};
        let plan =
            self.engine
                .prepare_parse(source, options, None, ParseAdmission::Limits(limits))?;
        let registration = self
            .modules
            .iter()
            .find(|module| module.descriptor.id == *plan.diagram_type())
            .expect("paired public registration")
            .clone();
        if registration.codec.is_host_er() {
            return Self::parse_host_er_plan(plan, registration, make_er).map(
                |parsed| match parsed {
                    host_er::Parsed::Host { policy, output } => {
                        host_sankey::CombinedParsed::Er { policy, output }
                    }
                    host_er::Parsed::Arena(document) => {
                        host_sankey::CombinedParsed::Arena(document)
                    }
                },
            );
        }
        if registration.codec.is_host_sankey() {
            return Self::parse_host_sankey_plan(plan, registration, make_sankey).map(|parsed| {
                match parsed {
                    host_sankey::Parsed::Captured { document, storage } => {
                        host_sankey::CombinedParsed::Sankey { document, storage }
                    }
                    host_sankey::Parsed::Arena(document) => {
                        host_sankey::CombinedParsed::Arena(document)
                    }
                }
            });
        }
        self.parse_plan(plan)
            .map(host_sankey::CombinedParsed::Arena)
    }
    #[doc(hidden)]
    pub fn parse_host_sankey_with_limits<W>(
        &self,
        source: &str,
        options: &ParseOptions,
        limits: ParseLimits,
        make: impl FnOnce(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_sankey::Parsed<W::Output>, PeleError>
    where
        W: super::InitialWriter,
        W::Output: super::captured_read::CapturedRead + super::captured_admission::AdmissionRead,
    {
        let plan =
            self.engine
                .prepare_parse(source, options, None, ParseAdmission::Limits(limits))?;
        let registration = self
            .modules
            .iter()
            .find(|r| r.descriptor.id == *plan.diagram_type())
            .expect("paired public registration")
            .clone();
        if !registration.codec.is_host_sankey() {
            return self
                .parse_plan(plan)
                .map(crate::compat::host_sankey::Parsed::Arena);
        }
        Self::parse_host_sankey_plan(plan, registration, make)
    }
    fn parse_host_sankey_plan<W>(
        plan: ParsePlan,
        registration: Arc<Registration>,
        make: impl FnOnce(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_sankey::Parsed<W::Output>, PeleError>
    where
        W: super::InitialWriter,
        W::Output: super::captured_read::CapturedRead + super::captured_admission::AdmissionRead,
    {
        Self::create_host_sankey(plan.parse()?, registration, make)
    }
    #[doc(hidden)]
    pub fn parse_prepared_host_sankey_as<W>(
        &self,
        id: &DiagramId,
        context: ParseContext<'_>,
        limit: EdgeLimit,
        make: impl FnOnce(ModelSchemaView) -> Result<W, PeleError>,
    ) -> Result<crate::compat::host_sankey::Parsed<W::Output>, PeleError>
    where
        W: super::InitialWriter,
        W::Output: super::captured_read::CapturedRead + super::captured_admission::AdmissionRead,
    {
        let plan = self.engine.prepare_prepared_as(id, context, limit)?;
        let registration = self
            .modules
            .iter()
            .find(|r| r.descriptor.id == *plan.diagram_type())
            .expect("paired public registration")
            .clone();
        let document = plan.parse()?;
        if !registration.codec.is_host_sankey() {
            return Self::create_document_registered(document, registration)
                .map(crate::compat::host_sankey::Parsed::Arena);
        }
        Self::create_host_sankey(document, registration, make)
    }
}
