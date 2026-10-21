//! Swimlane shares actual FlowDb identity/fields/builders, Pele 458322a5 (MIT).
//! Only the retained module renderer differs, exactly as swimlane/index.ts.
use super::{Swimlane, SwimlaneModel};
use crate::compat::{
    diagrams::flowchart::{Flowchart, FlowchartPublicState},
    public::*,
    DiagramModule, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub type SwimlanePublicState = FlowchartPublicState;
impl PublicDiagramModule for Swimlane {
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::swimlane())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        Some(DetachedPublicRender::swimlane(state.detached_seed()))
    }

    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        (prototype == PrototypeId(1)).then(|| {
            PublicConstructorFactory::first_value::<Self>(
                prototype,
                crate::compat::diagrams::flowchart::construct_flow_with,
                crate::compat::diagrams::flowchart::FLOW_LIVE_FIELDS,
            )
            .sharing_class::<Flowchart>()
        })
    }
    fn detached_public_invocation(&self, call: MethodCall) -> Option<DetachedPublicInvocation> {
        crate::compat::diagrams::flowchart::detached_flow_method::<Self>(call)
    }

    type PublicState = SwimlanePublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Flowchart.model_schema()
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        crate::compat::diagrams::flowchart::Flowchart.create_public_from_graph(graph)
    }
    fn create_public(&self, model: &SwimlaneModel) -> Result<Self::PublicState, PeleError> {
        Flowchart.create_public(model)
    }
    fn create_public_with_edge_limit(
        &self,
        model: &SwimlaneModel,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        Flowchart.create_public_with_edge_limit(model, limit)
    }
    fn create_public_prepared(
        &self,
        model: &SwimlaneModel,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        crate::compat::public::create_prepared_edge_public(self, model, limit)
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Flowchart.borrow_public_graph(state)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, state: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Flowchart.export_public(state)
    }
    fn apply_public(
        &self,
        state: &mut Self::PublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        Flowchart.apply_public(state, graph)
    }
    fn invoke_public(
        &self,
        state: &mut Self::PublicState,
        call: MethodCall,
        config: &Object,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        Flowchart.invoke_public(state, call, config, host)
    }
    fn render_public(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(state.render_model()?, config, options, host)
    }
    fn render_public_operation(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.render(state.prepared_render_model()?, config, options, host)
            .map_err(Into::into)
    }
}
