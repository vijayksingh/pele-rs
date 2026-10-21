//! Railroad family: IR, EBNF, ABNF and PEG, translated from Pele 458322a5 (MIT).
mod lexer;
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, Engine, ModuleDescriptor, Object, ParseContext, ParseOptions,
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub use model::{RailroadModel, RailroadNode, RailroadRule};
#[derive(Default)]
pub struct Railroad;
impl DiagramModule for Railroad {
    type Model = RailroadModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Railroad)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<RailroadModel, PeleError> {
        parser::parse(c)
    }
    fn validate_model_projection(&self, m: &RailroadModel) -> Result<(), PeleError> {
        m.validate_projection()
    }
    fn render(
        &self,
        m: &RailroadModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}
pub fn parse(source: &str, options: &ParseOptions) -> Result<RailroadModel, PeleError> {
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
pub use public_model::RailroadPublicState;

mod live_phase;
mod phase;
pub(crate) use live_phase::{render_detached, RailroadDetachedInput};
