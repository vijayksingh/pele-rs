//! Pele architecture module, 458322a5 (MIT, kepano).
mod layout;
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{ArchEdge, ArchGroup, ArchLayoutHint, ArchNode, ArchitectureModel};
#[derive(Default)]
pub struct Architecture;
impl DiagramModule for Architecture {
    type Model = ArchitectureModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Architecture)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<ArchitectureModel, PeleError> {
        model::populate(parser::parse(c.source)?, c.title)
    }
    fn render(
        &self,
        m: &ArchitectureModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::ArchitecturePublicState;

mod live_phase;
pub(crate) use live_phase::{render_detached, ArchitectureDetachedInput};
