//! Full Cynefin source/model/seeded boundaries/render module, Pele 458322a5 (MIT).
mod boundaries;
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, Engine, ModuleDescriptor, Object, ParseContext, ParseOptions,
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub use model::{CynefinDomain, CynefinItem, CynefinModel, CynefinTransition, DomainName};
#[derive(Default)]
pub struct Cynefin;
impl DiagramModule for Cynefin {
    type Model = CynefinModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Cynefin)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<CynefinModel, PeleError> {
        parser::parse(c)
    }
    fn render(
        &self,
        m: &CynefinModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}
pub fn parse(source: &str, options: &ParseOptions) -> Result<CynefinModel, PeleError> {
    let inspected = Engine::empty().inspect(source, &options.preparation)?;
    let mut source = inspected.encoded_text;
    source.push('\n');
    parser::parse(ParseContext {
        source: &source,
        config: &inspected.prepared.config,
        title: inspected.prepared.title.as_deref(),
        line_offset: inspected.prepared.line_offset,
        max_edges: options.max_edges,
    })
}

mod public_model;
pub use public_model::CynefinPublicState;

mod live_phase;
pub(crate) use live_phase::{render_detached, CynefinDetachedInput};
