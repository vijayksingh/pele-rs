//! Requirement diagrams, direct Pele requirement/* 458322a5 (MIT, kepano).
mod lexer;
mod model;
mod parser;
mod render;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{Relation, Requirement, RequirementClass, RequirementElement, RequirementModel};
#[derive(Default)]
pub struct RequirementDiagram;
impl DiagramModule for RequirementDiagram {
    type Model = RequirementModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Requirement)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use model::RequirementProvenance;
pub use public_model::RequirementPublicState;

mod live_phase;
mod phase;
pub(crate) use live_phase::{render_detached, RequirementDetachedInput};
