//! Dedicated Block diagram grammar, object arena, grid layout and routing.
//! Translated from Pele block/*, 458322a5 (MIT, Copyright 2026 kepano).
mod arrows;
mod layout;
mod lexer;
mod live_phase;
mod model;
mod parser;
mod phase;
mod render;
pub(crate) use live_phase::{render_detached, BlockDetachedInput};
mod route;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{BlockClassDef, BlockModel, BlockNode};
#[derive(Default)]
pub struct Block;
impl DiagramModule for Block {
    type Model = BlockModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Block)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<BlockModel, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn validate_model_projection(&self, m: &BlockModel) -> Result<(), PeleError> {
        m.projection()
    }
    fn render(
        &self,
        m: &BlockModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::BlockPublicState;
