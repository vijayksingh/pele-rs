//! Pele diagrams/radar/{index,model}.ts, 458322a5 (MIT, kepano).
mod live_phase;
pub(crate) use live_phase::{render_detached, RadarDetachedInput};
mod parser;
mod public_model;
pub use public_model::RadarPublicState;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub struct RadarAxis {
    pub name: String,
    pub label: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct RadarCurve {
    pub name: String,
    pub label: String,
    pub entries: Vec<f64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarOptions {
    pub show_legend: bool,
    pub ticks: f64,
    pub max: Option<f64>,
    pub min: f64,
    pub graticule: String,
}
impl Default for RadarOptions {
    fn default() -> Self {
        Self {
            show_legend: true,
            ticks: 5.,
            max: None,
            min: 0.,
            graticule: "circle".into(),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
    pub axes: Vec<RadarAxis>,
    pub curves: Vec<RadarCurve>,
    pub options: RadarOptions,
}
#[derive(Default)]
pub struct Radar;
impl DiagramModule for Radar {
    type Model = RadarModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Radar)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<RadarModel, PeleError> {
        parser::parse(context.source, context.title)
    }
    fn render(
        &self,
        model: &RadarModel,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}
