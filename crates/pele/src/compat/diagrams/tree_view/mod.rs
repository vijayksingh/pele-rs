//! TreeView parser, box-drawing conversion and native row renderer, Pele 458322a5 (MIT).
mod box_drawing;
mod icons;
mod live_phase;
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub(crate) use live_phase::{render_detached, TreeViewDetachedInput};
pub use model::{NodeType, TreeNode, TreeViewModel};
#[derive(Default)]
pub struct TreeView;
impl DiagramModule for TreeView {
    type Model = TreeViewModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::TreeView)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<TreeViewModel, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn validate_model_projection(&self, m: &TreeViewModel) -> Result<(), PeleError> {
        m.projection()
    }
    fn render(
        &self,
        m: &TreeViewModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::TreeViewPublicState;
