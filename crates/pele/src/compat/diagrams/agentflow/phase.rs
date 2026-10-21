//! Agentflow cached geometry and resource-live inputs, Pele458322a5 (MIT,kepano).
use super::{graph::GraphNode, model::AgentflowModel};
use crate::compat::PeleError;
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    fn resolve_styles(&mut self, index: usize, node: &mut GraphNode) -> Result<(), Self::Error>;
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error>;
}
pub(super) struct Typed<'a>(pub &'a AgentflowModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    fn resolve_styles(&mut self, _: usize, _: &mut GraphNode) -> Result<(), PeleError> {
        Ok(())
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PeleError> {
        Ok(match key {
            "direction" => self.0.direction.clone(),
            "title" => self.0.title.clone(),
            "accTitle" => self.0.acc_title.clone(),
            "accDescr" => self.0.acc_descr.clone(),
            _ => None,
        })
    }
}
