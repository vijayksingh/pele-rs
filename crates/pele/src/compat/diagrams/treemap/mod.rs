//! Pele treemap family, direct 458322a5 translation (MIT, kepano).
mod format;
mod layout;
mod model;
mod parser;
mod public_model;
pub use public_model::TreemapPublicState;
mod live_phase;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ErrorCode, ModuleDescriptor, Object, ParseContext, PeleError,
    RenderResources, RenderSettings, Rendered,
};
pub(crate) use live_phase::{render_detached, TreemapDetachedInput};
pub use model::{TreemapModel, TreemapNode};
#[derive(Default)]
pub struct Treemap;
impl DiagramModule for Treemap {
    type Model = TreemapModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Treemap)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c)
    }
    fn validate_model_projection(&self, m: &Self::Model) -> Result<(), PeleError> {
        let mut stack: Vec<_> = m.roots.iter().map(|n| (n, 3)).collect();
        while let Some((n, d)) = stack.pop() {
            if d > 128 {
                return Err(PeleError::new(
                    "Treemap model exceeds safe JSON projection depth",
                    ErrorCode::Limit,
                ));
            }
            if let Some(children) = &n.children {
                stack.extend(children.iter().map(|n| (n, d + 2)));
            }
        }
        Ok(())
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
