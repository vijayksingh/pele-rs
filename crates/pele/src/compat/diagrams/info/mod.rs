//! Pele diagrams/info/{index,model,parser,render}.ts, 458322a5 (MIT, kepano).
//! Information diagrams report the pinned compatibility engine version.
mod live_phase;
pub(crate) use live_phase::{render_detached, InfoDetachedInput};
mod parser;
mod public_model;
pub use public_model::InfoPublicState;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
}
#[derive(Default)]
pub struct Info;
impl DiagramModule for Info {
    type Model = InfoModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Info)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<InfoModel, PeleError> {
        parser::parse(context.source, context.title)
    }
    fn render(
        &self,
        model: &InfoModel,
        _config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, options, resources)
    }
}
