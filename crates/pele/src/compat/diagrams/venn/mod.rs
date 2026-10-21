//! Full Venn parser/DB/circle geometry/renderer, Pele 458322a5 (MIT).
mod layout;
mod lexer;
mod math;
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, Engine, ModuleDescriptor, Object, ParseContext, ParseOptions,
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub use model::{VennModel, VennStyle, VennSubset, VennText};
#[derive(Default)]
pub struct Venn;
impl DiagramModule for Venn {
    type Model = VennModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Venn)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<VennModel, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn render(
        &self,
        m: &VennModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}
pub fn parse(source: &str, options: &ParseOptions) -> Result<VennModel, PeleError> {
    let inspected = Engine::empty().inspect(source, &options.preparation)?;
    let mut source = inspected.encoded_text;
    source.push('\n');
    parser::parse(&source, inspected.prepared.title.as_deref())
}

mod public_model;
pub use public_model::VennPublicState;

mod live_phase;
pub(crate) use live_phase::{render_detached, VennDetachedInput};
