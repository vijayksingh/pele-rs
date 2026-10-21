//! Full Pele GitGraph source/model/render module, pinned 458322a5 (MIT, kepano).
mod live_phase;
mod model;
mod parser;
mod phase;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, Engine, ModuleDescriptor, Object, ParseContext, ParseOptions,
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) use live_phase::{render_detached, GitGraphDetachedInput};
pub use model::{GitCommit, GitModel};
#[derive(Default)]
pub struct GitGraph;
impl DiagramModule for GitGraph {
    type Model = GitModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::GitGraph)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<GitModel, PeleError> {
        model::build(parser::parse(c.source)?, c.config, c.title)
    }
    fn render(
        &self,
        m: &GitModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}
/// Parse through the same preparation, configuration and source limits as Engine.
pub fn parse(source: &str, options: &ParseOptions) -> Result<GitModel, PeleError> {
    let inspected = Engine::empty().inspect(source, &options.preparation)?;
    let mut encoded = inspected.encoded_text;
    encoded.push('\n');
    model::build(
        parser::parse(&encoded)?,
        &inspected.prepared.config,
        inspected.prepared.title.as_deref(),
    )
}

mod public_model;
pub use public_model::GitGraphPublicState;
