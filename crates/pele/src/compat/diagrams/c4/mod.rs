//! Pele C4 family: all five header forms share a typed arena, parser and renderer.
mod lexer;
mod model;
mod parser;
mod tokens;
use crate::compat::{Engine, ParseContext, ParseOptions, PeleError};
pub use model::{C4Attr, C4Boundary, C4Element, C4Model, C4Named, C4Rel, C4Shape, ElementRef};
/// Parse through the same source preparation/configuration limits used by Engine.
pub fn parse(source: &str, options: &ParseOptions) -> Result<C4Model, PeleError> {
    let inspected = Engine::empty().inspect(source, &options.preparation)?;
    let mut encoded = inspected.encoded_text;
    encoded.push('\n');
    parser::parse(ParseContext {
        source: &encoded,
        config: &inspected.prepared.config,
        title: inspected.prepared.title.as_deref(),
        line_offset: inspected.prepared.line_offset,
        max_edges: options.max_edges,
    })
}
mod live_phase;
mod phase;
pub(crate) use live_phase::{render_detached, C4DetachedInput};
mod layout;
mod render;
mod route;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, RenderResources, RenderSettings, Rendered,
};
#[derive(Default)]
pub struct C4;
impl DiagramModule for C4 {
    type Model = C4Model;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::C4)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<C4Model, PeleError> {
        parser::parse(c)
    }
    fn validate_model_projection(&self, m: &C4Model) -> Result<(), PeleError> {
        m.validate_projection()
    }
    fn render(
        &self,
        m: &C4Model,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::C4PublicState;
