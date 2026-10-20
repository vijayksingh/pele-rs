//! Pele diagrams/timeline/{index,db}.ts, 458322a5 (MIT, kepano).
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePeriod {
    pub text: String,
    pub events: Vec<String>,
    pub section: String,
    pub section_index: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
    pub direction: String,
    pub sections: Vec<String>,
    pub periods: Vec<TimelinePeriod>,
    pub current_section: String,
}
#[derive(Default)]
pub struct Timeline;
impl DiagramModule for Timeline {
    type Model = TimelineModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Timeline)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::TimelinePublicState;

impl TimelineModel {
    /// Empty class state shared by parser population and native construction.
    pub(super) fn empty(title: Option<&str>) -> Self {
        Self {
            diagram_type: DiagramType::Timeline,
            title: title.map(str::to_owned),
            acc_title: None,
            acc_descr: None,
            direction: "LR".into(),
            sections: Vec::new(),
            periods: Vec::new(),
            current_section: String::new(),
        }
    }
}
