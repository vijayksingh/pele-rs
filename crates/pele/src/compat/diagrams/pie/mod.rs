//! Pele diagrams/pie/{parser,model,index}.ts, 458322a5 (MIT, kepano).
mod parser;
mod public_model;
pub use public_model::PiePublicState;
mod live_phase;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered, Value,
};
pub(crate) use live_phase::{render_detached, PieDetachedInput};
use serde::{Serialize, Serializer};
#[derive(Clone, Debug)]
pub struct PieSection {
    pub label: String,
    pub value: f64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
    pub show_data: bool,
    #[serde(serialize_with = "sections")]
    pub sections: Vec<PieSection>,
}
fn sections<S: Serializer>(sections: &[PieSection], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut out = s.serialize_seq(Some(sections.len()))?;
    for section in sections {
        out.serialize_element(&(&section.label, Value::Number(section.value)))?
    }
    out.end()
}
#[derive(Default)]
pub struct Pie;
impl DiagramModule for Pie {
    type Model = PieModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Pie)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<PieModel, PeleError> {
        parser::parse(context.source, context.title)
    }
    fn render(
        &self,
        model: &PieModel,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}
