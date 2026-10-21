//! Full Wardley source, coordinate model and map renderer, Pele 458322a5 (MIT).
mod lexer;
mod model;
mod parser;
mod render;
mod phase;
mod live_phase;
pub(crate) use live_phase::{render_detached,WardleyDetachedInput};
use crate::compat::{
    DiagramModule, DiagramType, Engine, ModuleDescriptor, Object, ParseContext, ParseOptions,
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub use model::{
    WardleyAnnotation, WardleyAxes, WardleyForce, WardleyLink, WardleyModel, WardleyNode,
    WardleyNote, WardleyPipeline, WardleyPoint, WardleySize, WardleyTrend,
};
#[derive(Default)]
pub struct Wardley;
impl DiagramModule for Wardley {
    type Model = WardleyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        let mut d = ModuleDescriptor::new(DiagramType::Wardley);
        d.section = Some("wardley-beta".into());
        d
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<WardleyModel, PeleError> {
        model::build(parser::parse(c.source)?, c.title)
    }
    fn render(
        &self,
        m: &WardleyModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}
pub fn parse(source: &str, options: &ParseOptions) -> Result<WardleyModel, PeleError> {
    let inspected = Engine::empty().inspect(source, &options.preparation)?;
    let mut source = inspected.encoded_text;
    source.push('\n');
    model::build(parser::parse(&source)?, inspected.prepared.title.as_deref())
}

mod public_model;
pub use public_model::WardleyPublicState;
