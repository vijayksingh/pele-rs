//! Complete XY chart family, Pele xychart/* 458322a5 (MIT, kepano).
mod lexer;
mod model;
mod parser;
mod public_model;
pub use public_model::XyChartPublicState;
mod live_phase;
mod phase;
mod render;
pub(crate) use live_phase::{render_detached, XYChartDetachedInput};
mod ticks;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{LinearAxis, Orientation, Plot, PlotType, XAxis, XyChartModel};
#[derive(Default)]
pub struct XyChart;
impl DiagramModule for XyChart {
    type Model = XyChartModel;
    fn descriptor(&self) -> ModuleDescriptor {
        let mut d = ModuleDescriptor::new(DiagramType::Xychart);
        d.section = Some("xyChart".into());
        d
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<XyChartModel, PeleError> {
        parser::parse(context)
    }
    fn render(
        &self,
        model: &XyChartModel,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}
