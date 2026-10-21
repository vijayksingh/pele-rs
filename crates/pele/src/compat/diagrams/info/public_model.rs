//! Mutable plain Info objects. Geometry remains in the existing Info renderer.
use super::{Info, InfoModel};
use crate::compat::{
    public::{Atom, GraphWriter, ModelGraph, Node, PublicDiagramModule},
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};

pub struct InfoPublicState {
    graph: ModelGraph,
}
fn invalid(message: &str) -> PeleError {
    let mut error = PeleError::new(message, ErrorCode::Semantic);
    error.diagram_type = Some("info".into());
    error
}
fn text(graph: &ModelGraph, value: &Atom) -> Result<String, PeleError> {
    graph
        .text(value)?
        .well_formed()
        .ok_or_else(|| invalid("UTF-16 Info text rendering is not implemented"))
}
fn title(graph: &ModelGraph) -> Result<Option<String>, PeleError> {
    match graph.field("title")? {
        None | Some(Atom::Undefined) => Ok(None),
        Some(value) => text(graph, value).map(Some),
    }
}
fn accessible(graph: &ModelGraph, key: &str) -> Result<Option<String>, PeleError> {
    match graph.field(key)? {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => Ok(None),
        Some(Atom::Number(n)) if *n == 0. || n.is_nan() => Ok(None),
        Some(value) => text(graph, value).map(Some),
    }
}
impl PublicDiagramModule for Info {
    type PublicState = InfoPublicState;
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        Ok(InfoPublicState { graph })
    }
    fn create_public(&self, model: &InfoModel) -> Result<InfoPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let kind = out.string("info");
        let mut fields = vec![out.property("type", Atom::String(kind))];
        // buildInfo returns these keys in this order even when values are undefined.
        for (key, value) in [
            ("title", &model.title),
            ("accTitle", &model.acc_title),
            ("accDescr", &model.acc_descr),
        ] {
            let value = value
                .as_deref()
                .map_or(Atom::Undefined, |v| Atom::String(out.string(v)));
            fields.push(out.property(key, value));
        }
        let root = out.node(Node::Object { properties: fields });
        Ok(InfoPublicState {
            graph: out.finish(Atom::Ref(root))?,
        })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, state: &InfoPublicState) -> Result<ModelGraph, PeleError> {
        Ok(state.graph.clone())
    }
    fn apply_public(
        &self,
        state: &mut InfoPublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        state.graph = graph;
        Ok(())
    }
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::info())
    }
    fn detached_public_render(
        &self,
        _state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::info(
            super::InfoDetachedInput,
        ))
    }
    fn render_public(
        &self,
        state: &mut InfoPublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        // Root type edits never retarget the exact retained handler.
        let model = InfoModel {
            diagram_type: DiagramType::Info,
            title: title(&state.graph)?,
            acc_title: accessible(&state.graph, "accTitle")?,
            acc_descr: accessible(&state.graph, "accDescr")?,
        };
        self.render(&model, config, options, host)
    }
}
