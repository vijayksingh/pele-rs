//! Ishikawa source grammar, flat outline model and dedicated fishbone geometry.
//! Translated from Pele ishikawa/*, 458322a5 (MIT, Copyright 2026 kepano).
mod live_phase;
pub(crate) use live_phase::{render_detached, IshikawaDetachedInput};
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{IshikawaModel, IshikawaNode};
#[derive(Default)]
pub struct Ishikawa;
impl DiagramModule for Ishikawa {
    type Model = IshikawaModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Ishikawa)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<IshikawaModel, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn validate_model_projection(&self, m: &IshikawaModel) -> Result<(), PeleError> {
        m.projection()
    }
    fn render(
        &self,
        m: &IshikawaModel,
        _c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, o, r)
    }
}

mod public_model;
pub use public_model::IshikawaPublicState;
