//! Flowchart graph capture and live style/metadata phases, Pele 458322a5 (MIT, kepano).
use super::{
    graph::{build_flow_graph, FlowGraph, GraphNode},
    model::FlowchartModel,
};
use crate::compat::{PeleError, RenderResources};
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    fn begin_draw(
        &mut self,
        curve: Option<&str>,
        resources: &mut dyn RenderResources,
    ) -> Result<FlowGraph, Self::Error>;
    fn resolve_styles(&mut self, node: &mut GraphNode) -> Result<(), Self::Error>;
    fn direction(&mut self) -> Result<Option<String>, Self::Error>;
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error>;
}
pub(super) struct Typed<'a>(pub &'a FlowchartModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    fn begin_draw(
        &mut self,
        curve: Option<&str>,
        _resources: &mut dyn RenderResources,
    ) -> Result<FlowGraph, PeleError> {
        Ok(build_flow_graph(self.0, curve))
    }
    fn resolve_styles(&mut self, _node: &mut GraphNode) -> Result<(), PeleError> {
        Ok(())
    }
    fn direction(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.direction.clone())
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PeleError> {
        Ok(match key {
            "title" => self.0.title.clone(),
            "accTitle" => self.0.acc_title.clone(),
            "accDescr" => self.0.acc_descr.clone(),
            _ => None,
        })
    }
}
